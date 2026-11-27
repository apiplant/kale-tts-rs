use std::{io::IsTerminal, path::PathBuf, sync::mpsc};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ntex::web;
use rust_embed::RustEmbed;
use serde::Deserialize;
use kale_tts_rs::{Config, KaleTts, download::{ModelId, cache_model, default_cache_dir, http_agent}};

fn default_device() -> &'static str {
    if cfg!(feature = "cuda") { "auto" } else { "cpu" }
}

#[derive(Parser)]
#[command(name = "kale-tts", version, about = "Native Candle text-to-speech using Kale models")]
struct Args {
    #[arg(long, global = true, env = "KALE_TTS_MODEL", default_value = "nano")]
    model: String,
    #[arg(long, global = true, env = "KALE_TTS_DEVICE", default_value_t = default_device().to_string())]
    device: String,
    #[arg(long, global = true, env = "KALE_TTS_CACHE", default_value_os_t = default_cache_dir())]
    cache_dir: PathBuf,
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Render speech to a 24 kHz mono WAV file.
    Synthesize { text: String, #[arg(long, default_value = "")] voice: String, output: PathBuf },
    /// List voices provided by the selected model.
    Voices,
    /// Download model graph and metadata into the local cache.
    Fetch,
    /// Start the ntex HTTP API and bundled SolidJS interface.
    Serve { #[arg(long, env = "KALE_TTS_ADDR", default_value = "127.0.0.1:8080")] addr: String },
}

#[derive(Deserialize)]
struct SpeechRequest { input: String, #[serde(default = "default_voice")] voice: String }
fn default_voice() -> String { String::new() }

struct Job { request: SpeechRequest, reply: futures_channel::oneshot::Sender<Result<Vec<u8>, String>> }

#[derive(RustEmbed)]
#[folder = "studio/dist/"]
struct Ui;

#[ntex::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let model = ModelId::parse(&args.model)?;
    match args.command {
        Action::Fetch => {
            let path = cache_model(&http_agent(), model, &args.cache_dir)?;
            eprintln!("Model files ready in {}", path.display());
        }
        Action::Synthesize { text, voice, output } => {
            let mut tts = KaleTts::load(Config { model, device: args.device, cache_dir: args.cache_dir })?;
            let samples = tts.synthesize(&text, &voice)?;
            std::fs::write(&output, tts.wav_bytes(&samples)?).with_context(|| format!("writing {}", output.display()))?;
        }
        Action::Voices => {
            let tts = KaleTts::load(Config { model, device: args.device, cache_dir: args.cache_dir })?;
            for (id, name, accent) in tts.voices() { println!("{id}\t{name}\t{accent}"); }
        }
        Action::Serve { addr } => serve(Config { model, device: args.device, cache_dir: args.cache_dir }, &addr).await?,
    }
    Ok(())
}

async fn serve(config: Config, addr: &str) -> Result<()> {
    if Ui::get("index.html").is_none() {
        anyhow::bail!("bundled SolidJS UI is missing; build it with `pnpm --dir studio build` and rebuild this binary");
    }
    eprintln!("loading {} on {}", config.model.slug(), config.device);
    let (tx, rx) = mpsc::sync_channel::<Job>(8);
    let (ready_tx, ready_rx) = mpsc::sync_channel::<std::result::Result<serde_json::Value, String>>(1);
    std::thread::Builder::new().name("kale-inference".into()).spawn(move || {
        let mut tts = match KaleTts::load(config) {
            Ok(tts) => tts,
            Err(error) => { let _ = ready_tx.send(Err(format!("{error:#}"))); return; }
        };
        let info = serde_json::json!({"model": tts.model_name(), "sample_rate": 24000, "voices": tts.voices().iter().map(|(id,name,accent)| serde_json::json!({"id":id,"name":name,"accent":accent})).collect::<Vec<_>>()});
        let _ = ready_tx.send(Ok(info));
        for job in rx {
            let result = tts.synthesize(&job.request.input, &job.request.voice).and_then(|samples| tts.wav_bytes(&samples)).map_err(|e| format!("{e:#}"));
            let _ = job.reply.send(result);
        }
    })?;
    let model_info = ready_rx.recv().context("model worker stopped during startup")?.map_err(anyhow::Error::msg)?;
    let server = web::HttpServer::new(async move |_| {
        let worker = tx.clone();
        let model_info = model_info.clone();
        web::App::new()
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/health").route(web::get().to(async || "ready")))
            .service(web::resource("/voices").route(web::get().to(move || { let value = model_info.clone(); async move { let mut response = web::HttpResponse::Ok(); response.json(&value) } })))
            .service(web::resource("/v1/audio/speech").route(web::post().to(move |body: web::types::Payload| speech(body, worker.clone()))))
            .service(web::resource("/assets/{file}").route(web::get().to(bundle_asset)))
            .service(web::resource("/mark.svg").route(web::get().to(|| async { embedded_asset("mark.svg") })))
    }).bind(addr, ntex::SharedCfg::default())?;
    let host = addr.replace("0.0.0.0", "localhost").replace("[::]", "localhost");
    let color = std::io::stderr().is_terminal();
    let accent = |s: &str| if color { format!("\x1b[38;5;215m{s}\x1b[0m") } else { s.to_owned() };
    eprintln!("\n  {}  http://{host}/\n  {}  POST /v1/audio/speech\n", accent("Kale TTS"), accent("ntex API"));
    server.run().await?;
    Ok(())
}

async fn index() -> web::HttpResponse {
    match Ui::get("index.html") {
        Some(file) => web::HttpResponse::Ok().content_type("text/html; charset=utf-8").body(file.data.into_owned()),
        None => web::HttpResponse::InternalServerError().body("UI bundle is missing"),
    }
}

async fn bundle_asset(path: web::types::Path<String>) -> web::HttpResponse {
    embedded_asset(&format!("assets/{}", path.into_inner()))
}

fn embedded_asset(path: &str) -> web::HttpResponse {
    match Ui::get(path) {
        Some(file) => {
            let content_type = mime_guess::from_path(path).first_or_octet_stream().to_string();
            let mut response = web::HttpResponse::Ok();
            response.content_type(content_type);
            response.body(file.data.into_owned())
        }
        None => web::HttpResponse::NotFound().body("not found"),
    }
}

async fn speech(mut body: web::types::Payload, worker: mpsc::SyncSender<Job>) -> web::HttpResponse {
    let mut bytes = Vec::new();
    while let Some(chunk) = body.recv().await {
        match chunk { Ok(chunk) => bytes.extend_from_slice(&chunk), Err(_) => return web::HttpResponse::BadRequest().body("invalid request body") }
        if bytes.len() > 1_048_576 { return web::HttpResponse::PayloadTooLarge().body("request body is too large"); }
    }
    let request: SpeechRequest = match serde_json::from_slice(&bytes) { Ok(value) => value, Err(_) => return web::HttpResponse::BadRequest().body("expected JSON with input and optional voice") };
    let (reply, rx) = futures_channel::oneshot::channel();
    if worker.try_send(Job { request, reply }).is_err() {
        let mut response = web::HttpResponse::ServiceUnavailable(); response.set_header("Retry-After", "2");
        return response.body("inference queue is full; retry shortly");
    }
    match rx.await {
        Ok(Ok(wav)) => web::HttpResponse::Ok().content_type("audio/wav").body(wav),
        Ok(Err(error)) => web::HttpResponse::BadRequest().body(error),
        Err(_) => web::HttpResponse::InternalServerError().body("inference worker failed"),
    }
}

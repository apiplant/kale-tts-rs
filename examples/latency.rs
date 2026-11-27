use std::{hint::black_box, path::PathBuf, time::Instant};

use kale_tts_rs::{
    Config, KaleTts,
    download::{ModelId, default_cache_dir},
};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let model = ModelId::parse(&args.next().expect("nano|tiny|base"))?;
    let default_device = if cfg!(feature = "cuda") {
        "auto"
    } else {
        "cpu"
    };
    let device = args.next().unwrap_or_else(|| default_device.into());
    let runs = args.next().map_or(Ok(3), |runs| runs.parse::<usize>())?;
    anyhow::ensure!(runs > 0, "number of timed calls must be positive");
    let voice = if model == ModelId::Base {
        "kenzie"
    } else {
        "s057b"
    };
    let text = "Hello there.";

    let cache_dir = std::env::var_os("KALE_TTS_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(default_cache_dir);
    let start = Instant::now();
    let mut tts = KaleTts::load(Config {
        model,
        device: device.clone(),
        cache_dir,
    })?;
    println!(
        "model={} device={device} cold_load_ms={:.3}",
        model.slug(),
        millis(start)
    );
    let start = Instant::now();
    let warmup = tts.synthesize(text, voice)?;
    println!(
        "first_synthesis_ms={:.3} audio_samples={}",
        millis(start),
        warmup.len()
    );
    for _ in 0..2 {
        black_box(tts.synthesize(text, voice)?);
    }

    let mut synthesis = Vec::with_capacity(runs);
    let mut wav = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        let samples = tts.synthesize(text, voice)?;
        synthesis.push(millis(start));
        black_box(tts.wav_bytes(&samples)?);
        wav.push(millis(start));
    }
    print_summary("warm_synthesis", synthesis);
    print_summary("warm_with_pcm16_wav", wav);
    Ok(())
}

fn millis(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn print_summary(label: &str, mut times: Vec<f64>) {
    times.sort_unstable_by(f64::total_cmp);
    let n = times.len();
    let p50 = (times[(n - 1) / 2] + times[n / 2]) / 2.0;
    let p95 = times[(n as f64 * 0.95).ceil() as usize - 1];
    println!(
        "{label}_ms n={n} p50={p50:.3} p95={p95:.3} max={:.3}",
        times[n - 1]
    );
}

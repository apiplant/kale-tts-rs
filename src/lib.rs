//! Candle based Kale TTS inference. Nano and tiny use the published ONNX graphs.

pub mod download;
pub mod kokoro;
pub mod model;
pub use candle_onnx::onnx;

use std::{collections::HashMap, path::{Path, PathBuf}};

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use candle_core::{DType, Device, Tensor};
use candle_onnx::onnx::ModelProto;
use rquickjs::{Context as JsContext, Runtime as JsRuntime};
use serde::Deserialize;

use crate::download::{ModelId, cache_model, default_cache_dir, download_voice, http_agent};

const FE_RATE: u32 = 24_000;

#[derive(Clone, Debug)]
pub struct Config {
    pub model: ModelId,
    pub device: String,
    pub cache_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let device = if cfg!(feature = "cuda") { "auto" } else { "cpu" };
        Self { model: ModelId::Nano, device: device.into(), cache_dir: default_cache_dir() }
    }
}

#[derive(Deserialize)]
struct VoicesFile {
    #[serde(default)]
    voice_inputs: Vec<VoiceInput>,
    #[serde(default)]
    voices: Vec<VoiceMeta>,
}
#[derive(Deserialize)]
struct VoiceInput { name: String, dims: Vec<usize> }
#[derive(Clone, Deserialize)]
struct VoiceMeta {
    #[serde(default)] id: String,
    name: Option<String>,
    accent: Option<String>,
    #[serde(default)] file: Option<String>,
}

struct FrontEnd { _runtime: JsRuntime, context: JsContext }
impl FrontEnd {
    fn load(ettin: Option<&str>) -> Result<Self> {
        let runtime = JsRuntime::new().context("starting bundled text front end")?;
        let context = JsContext::full(&runtime).context("creating text front-end context")?;
        context.with(|ctx| -> rquickjs::Result<()> {
            ctx.eval::<(), _>(include_str!("../vendor/kale_frontend.js"))?;
            let globals = ctx.globals();
            globals.set("feLexUs", include_str!("../vendor/frontend/words/lexicon.tsv"))?;
            globals.set("feOverrideUs", include_str!("../vendor/frontend/words/override.public.tsv"))?;
            globals.set("feTermsUs", include_str!("../vendor/frontend/words/terms.tsv"))?;
            globals.set("feG2pUs", BASE64.encode(include_bytes!("../vendor/frontend/words/g2p.bin")))?;
            ctx.eval::<bool, _>("kaleFrontEnd.init(feLexUs, feOverrideUs, feTermsUs, feG2pUs)")?;
            globals.set("feLexGb", include_str!("../vendor/frontend/words/en-gb/lexicon.tsv"))?;
            globals.set("feOverrideGb", include_str!("../vendor/frontend/words/en-gb/override.public.tsv"))?;
            globals.set("feTermsGb", include_str!("../vendor/frontend/words/terms.tsv"))?;
            globals.set("feG2pGb", BASE64.encode(include_bytes!("../vendor/frontend/words/en-gb/g2p.bin")))?;
            ctx.eval::<bool, _>("kaleFrontEnd.initGb(feLexGb, feOverrideGb, feTermsGb, feG2pGb)")?;
            if let Some(tokenizer) = ettin { globals.set("feEttin", tokenizer)?; ctx.eval::<bool, _>("kaleFrontEnd.initEttin(feEttin)")?; }
            Ok(())
        }).context("initializing bundled text front end")?;
        Ok(Self { _runtime: runtime, context })
    }

    fn sentences(&self, text: &str, accent: &str) -> Result<Vec<Sentence>> {
        let json = self.context.with(|ctx| -> rquickjs::Result<String> {
            let globals = ctx.globals(); globals.set("feText", text)?; globals.set("feAccent", accent)?;
            ctx.eval("kaleFrontEnd.run(feText, feAccent)")
        }).context("running bundled pronunciation front end")?;
        Ok(serde_json::from_str(&json)?)
    }

    fn word_ipa(&self, text: &str, accent: &str) -> Result<Vec<String>> {
        let json = self.context.with(|ctx| -> rquickjs::Result<String> {
            let globals = ctx.globals(); globals.set("feText", text)?; globals.set("feAccent", accent)?;
            ctx.eval("kaleFrontEnd.ipaWords(feText, feAccent)")
        }).context("getting word pronunciations")?;
        Ok(serde_json::from_str(&json)?)
    }

    fn ettin(&self, text: &str, previous: &str, ids: &[i64], word_ids: &[Vec<i64>]) -> Result<EttinInputs> {
        let ids = serde_json::to_string(ids)?;
        let word_ids = serde_json::to_string(word_ids)?;
        let json = self.context.with(|ctx| -> rquickjs::Result<String> {
            let globals = ctx.globals(); globals.set("feText", text)?; globals.set("fePrevious", previous)?;
            globals.set("feSoundIds", ids)?; globals.set("feWordIds", word_ids)?;
            ctx.eval("kaleFrontEnd.ettin(feText, fePrevious, feSoundIds, feWordIds)")
        }).context("building tiny-model Ettin inputs")?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[derive(Deserialize)]
struct Sentence { text: String, ipa: String }
#[derive(Deserialize)]
struct EttinInputs { ids: Vec<i64>, wavg: Vec<f32>, widx: Vec<i64>, #[serde(rename = "W")] words: usize, #[serde(rename = "S")] sequence: usize }

pub struct KaleTts {
    id: ModelId,
    device: Device,
    model: Option<ModelProto>,
    base_model: Option<kokoro::Kokoro>,
    init: HashMap<String, Tensor>,
    host_integer_constants: HashMap<String, Vec<i64>>,
    host_shape_plan: Option<model::onnx_eval::HostShapePlan>,
    #[cfg(feature = "cuda")]
    nano_graph: Option<model::nano_graph::NanoGraph>,
    model_dir: PathBuf,
    voices: VoicesFile,
    vocab: HashMap<String, i64>,
    frontend: FrontEnd,
    agent: ureq::Agent,
}

impl KaleTts {
    pub fn load(config: Config) -> Result<Self> {
        let agent = http_agent();
        let model_dir = cache_model(&agent, config.model, &config.cache_dir)?;
        let new_cuda = || {
            if config.model == ModelId::Nano {
                Device::new_cuda_with_stream(0)
            } else {
                Device::new_cuda(0)
            }
        };
        let device = match config.device.as_str() {
            "cpu" => Device::Cpu,
            "auto" => new_cuda().unwrap_or(Device::Cpu),
            "cuda" | "cuda:0" => new_cuda().context("initializing CUDA device 0")?,
            other => bail!("unsupported device '{other}'; use cpu, auto, or cuda:0"),
        };
        if device.is_cuda() && !cfg!(feature = "cuda") { bail!("this binary was built without the cuda feature"); }
        #[cfg(feature = "cuda")]
        if config.model == ModelId::Nano && device.is_cuda()
            && std::env::var_os("KALE_TTS_DISABLE_SINGLE_STREAM").is_none()
        {
            // This model owns its Candle stream. BLAS, RNG, copies, and custom
            // launches enqueue there; no buffers are exported to other streams.
            unsafe { device.as_cuda_device()?.disable_event_tracking(); }
        }
        if let Some(seed) = std::env::var_os("KALE_TTS_CUDA_SEED") {
            if !device.is_cuda() {
                bail!("KALE_TTS_CUDA_SEED requires a CUDA device");
            }
            let seed = seed.to_string_lossy().parse::<u64>()
                .context("KALE_TTS_CUDA_SEED must be an unsigned 64-bit integer")?;
            device.set_seed(seed).context("setting CUDA seed")?;
        }
        let (model, base_model, init, host_integer_constants) = if config.model == ModelId::Base {
            let base = kokoro::Kokoro::load(&model_dir, &device).map_err(anyhow::Error::from)
                .context("loading the converted Kale base checkpoint with the native Candle Kokoro graph")?;
            (None, Some(base), HashMap::new(), HashMap::new())
        } else {
            let path = model_dir.join("model.candle.onnx");
            let mut model = candle_onnx::read_file(&path).with_context(|| format!("reading Candle graph {}", path.display()))?;
            let graph = model.graph.as_ref().context("ONNX graph is empty")?;
            let mut init = graph.initializer.iter().map(|t| Ok((t.name.clone(), model::onnx_eval::get_tensor(t, &t.name, &device)?))).collect::<Result<HashMap<_, _>>>()?;
            // Constant nodes have static tensor payloads too. Upload them once with the
            // initializers instead of rebuilding and transferring hundreds of tiny tensors
            // for every request.
            for node in graph.node.iter().filter(|node| node.op_type == "Constant") {
                let output = node.output.first().context("Constant node has no output")?;
                let tensor = node.attribute.iter().find(|attr| attr.name == "value")
                    .and_then(|attr| attr.t.as_ref())
                    .with_context(|| format!("Constant node {} has no tensor value", node.name))?;
                init.insert(output.clone(), model::onnx_eval::get_tensor(tensor, &node.name, &device)?);
            }
            model::onnx_eval::prepare_constant_casts(&model, &mut init)?;
            model::onnx_eval::prepare_quantized_weights(&model, &mut init)?;
            model::onnx_eval::prepare_lstm_weights(&model, &mut init, &device)?;
            let mut host_integer_constants = model::onnx_eval::prepare_host_integer_constants(&model)?;
            model::onnx_eval::prepare_static_nodes(&model, &mut init, &mut host_integer_constants, &device)?;
            if config.model == ModelId::Nano {
                model::onnx_eval::compact_prepared_graph(&mut model, &mut init, &mut host_integer_constants);
            }
            (Some(model), None, init, host_integer_constants)
        };
        let host_shape_plan = match &model {
            Some(model) if config.model == ModelId::Nano && device.is_cuda()
                && std::env::var_os("KALE_TTS_DISABLE_HOST_SHAPES").is_none() =>
            {
                Some(model::onnx_eval::prepare_host_shape_plan(model, &init)?)
            }
            _ => None,
        };
        let voices_raw: serde_json::Value = serde_json::from_slice(&std::fs::read(model_dir.join("voices.json"))?)?;
        let voices = if config.model == ModelId::Base {
            let rows: Vec<VoiceMeta> = serde_json::from_value(voices_raw)?;
            VoicesFile { voice_inputs: Vec::new(), voices: rows.into_iter().map(|mut v| {
                if v.id.is_empty() {
                    v.id = v.file.as_deref().and_then(|path| Path::new(path).file_stem()).and_then(|stem| stem.to_str()).unwrap_or_default().to_owned();
                }
                v
            }).collect() }
        } else { serde_json::from_value(voices_raw)? };
        let vocab: HashMap<String, i64> = if config.model == ModelId::Base {
            let config_json: serde_json::Value = serde_json::from_slice(&std::fs::read(model_dir.join("config.json"))?)?;
            let vocab: HashMap<String, usize> = serde_json::from_value(config_json["vocab"].clone()).context("loading base phoneme vocabulary")?;
            vocab.into_iter().map(|(key, value)| (key, value as i64)).collect()
        } else {
            let tokenizer: serde_json::Value = serde_json::from_slice(&std::fs::read(model_dir.join("tokenizer.json"))?)?;
            serde_json::from_value(tokenizer["model"]["vocab"].clone()).context("loading phoneme vocabulary")?
        };
        let ettin = if config.model == ModelId::Tiny { Some(std::fs::read_to_string(model_dir.join("ettin-tokenizer.json"))?) } else { None };
        let frontend = FrontEnd::load(ettin.as_deref())?;
        #[cfg(feature = "cuda")]
        let nano_graph = if std::env::var_os("KALE_TTS_DISABLE_GRAPHS").is_none()
            && std::env::var_os("KALE_TTS_DISABLE_SINGLE_STREAM").is_none()
            && std::env::var_os("KALE_TTS_PROFILE_OPS").is_none()
            && std::env::var_os("KALE_TTS_TRACE_OPS").is_none()
        {
            model.as_ref().zip(host_shape_plan.as_ref())
                .and_then(|(model, plan)| model::nano_graph::NanoGraph::new(model, plan, &init))
        } else { None };
        Ok(Self {
            id: config.model, device, model, base_model, init, host_integer_constants, host_shape_plan,
            #[cfg(feature = "cuda")]
            nano_graph,
            model_dir, voices, vocab, frontend, agent,
        })
    }

    pub fn voices(&self) -> Vec<(String, String, String)> {
        self.voices.voices.iter().map(|v| (v.id.clone(), v.name.clone().unwrap_or_else(|| v.id.clone()), v.accent.clone().unwrap_or_else(|| "en-us".into()))).collect()
    }
    pub fn model_name(&self) -> &'static str { self.id.slug() }

    /// Encode samples as WAV, using the Kale CUDA PCM export kernel for CUDA runs.
    pub fn wav_bytes(&self, samples: &[f32]) -> Result<Vec<u8>> {
        wav_bytes_on_device(samples, &self.device)
    }

    /// Synthesize speech into 24 kHz mono PCM float samples.
    pub fn synthesize(&mut self, text: &str, requested_voice: &str) -> Result<Vec<f32>> {
        let fallback = if self.id == ModelId::Base { "kenzie" } else { "s057b" };
        let voice = self.resolve_voice(if requested_voice.is_empty() { fallback } else { requested_voice })?;
        download_voice(&self.agent, self.id, &self.model_dir, &voice.id)?;
        let voice_id = voice.id.clone();
        let accent = voice.accent.clone();
        let accent = accent.as_deref().unwrap_or("en-us");
        let sentences = self.frontend.sentences(text, accent)?;
        if sentences.is_empty() { bail!("text did not produce any speech"); }
        let mut audio = Vec::new();
        let mut previous = String::new();
        for sentence in sentences {
            let phone_ids = self.phone_ids(&sentence.ipa, accent)?;
            if phone_ids.is_empty() { continue; }
            if phone_ids.len() > 508 { bail!("sentence has {} phoneme tokens; split input below 508", phone_ids.len()); }
            let ettin = if self.id == ModelId::Tiny {
                let words = self.frontend.word_ipa(&sentence.text, accent)?;
                let word_ids = words.iter().map(|ipa| self.phone_ids(ipa, accent)).collect::<Result<Vec<_>>>()?;
                Some(self.frontend.ettin(&sentence.text, &previous, &phone_ids, &word_ids)?)
            } else { None };
            let wave = self.infer_sentence(&phone_ids, &voice_id, ettin.as_ref())
                .with_context(|| format!("Candle inference failed for {}/{}", self.id.slug(), voice_id))?;
            if !audio.is_empty() { audio.extend(std::iter::repeat_n(0.0f32, FE_RATE as usize / 8)); }
            audio.extend(wave);
            previous = sentence.text;
        }
        if audio.is_empty() { bail!("pronunciation front end produced no supported phoneme ids"); }
        Ok(audio)
    }

    fn resolve_voice(&self, name: &str) -> Result<&VoiceMeta> {
        self.voices.voices.iter().find(|v| v.id.eq_ignore_ascii_case(name) || v.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(name)))
            .with_context(|| format!("unknown voice '{name}'; available ids: {}", self.voices.voices.iter().map(|v| v.id.as_str()).collect::<Vec<_>>().join(", ")))
    }

    fn phone_ids(&self, ipa: &str, accent: &str) -> Result<Vec<i64>> {
        let chars = ipa.chars().collect::<Vec<_>>();
        let mut converted = Vec::new(); let mut i = 0;
        while i < chars.len() {
            let two = if i + 1 < chars.len() { chars[i..i+2].iter().collect::<String>() } else { String::new() };
            let pair = match two.as_str() {
                "eɪ" => Some('A'), "aɪ" => Some('I'), "aʊ" => Some('W'), "oʊ" if accent != "en-gb" => Some('O'),
                "əʊ" if accent == "en-gb" => Some('Q'), "ɔɪ" => Some('Y'), "tʃ" => Some('ʧ'), "dʒ" => Some('ʤ'), _ => None,
            };
            if let Some(c) = pair { converted.push(c); i += 2; continue; }
            let c = chars[i]; i += 1;
            match c { 'ː' => {}, 'ɚ' if accent != "en-gb" => converted.extend(['ə','ɹ']), 'ɝ' if accent != "en-gb" => converted.extend(['ɜ','ɹ']), 'ɾ' if accent != "en-gb" => converted.push('T'), _ => converted.push(c) }
        }
        for k in 1..converted.len().saturating_sub(1) {
            if converted[k] == 'ə' && "lnm".contains(converted[k + 1]) && (k + 2 == converted.len() || " .,!?;:".contains(converted[k + 2]))
                && !"aeiouɑɐɒæəɘɚɛɜɝɞɪɨʊʉʌɔøœɤɯʏyAIWOYᵊᵻˈˌ ".contains(converted[k - 1]) { converted[k] = 'ᵊ'; }
        }
        Ok(converted.iter().filter_map(|c| self.vocab.get(&c.to_string()).copied()).collect())
    }

    fn infer_sentence(&mut self, ids: &[i64], voice: &str, ettin: Option<&EttinInputs>) -> Result<Vec<f32>> {
        if let Some(model) = &self.base_model {
            let path = self.model_dir.join("voices").join(format!("{voice}.safetensors"));
            let style = kokoro::Kokoro::load_voice(&path, ids.len(), &self.device)?;
            let waveform = model.forward_ids(ids, &style, 1.0).context("native Candle Kokoro forward")?;
            let waveform = if waveform.rank() == 2 { waveform.squeeze(0)? } else { waveform };
            return waveform.to_dtype(DType::F32)?.to_vec1::<f32>().context("reading base waveform");
        }
        let style_bytes = std::fs::read(self.model_dir.join("voices").join(format!("{voice}.bin")))?;
        if style_bytes.len() % 4 != 0 { bail!("invalid style file for voice {voice}"); }
        let style = f32_bytes(&style_bytes);
        let rows = style.len() / 256;
        let row = ids.len().min(rows).saturating_sub(1);
        let extra = f32_bytes(&std::fs::read(self.model_dir.join("voices").join(format!("{voice}.extra.bin")))?);
        let mut values = self.init.clone();
        let mut input_ids = Vec::with_capacity(ids.len() + 2); input_ids.push(0i64); input_ids.extend_from_slice(ids); input_ids.push(0);
        let input_shape = (1, input_ids.len());
        let mut feed = |name: &str, tensor: Tensor| { values.insert(name.to_string(), tensor); };
        feed("input_ids", Tensor::from_vec(input_ids, input_shape, &self.device)?);
        feed("speed", Tensor::ones((1,), DType::F32, &self.device)?);
        feed("style", Tensor::from_vec(style[row * 256..(row + 1) * 256].to_vec(), (1, 256), &self.device)?);
        let mut offset = 0usize;
        for spec in &self.voices.voice_inputs {
            let len = spec.dims.iter().product::<usize>();
            let end = offset.checked_add(len).context("voice input offset overflow")?;
            if end > extra.len() { bail!("voice extra input file is too short for {}", spec.name); }
            feed(&spec.name, Tensor::from_vec(extra[offset..end].to_vec(), spec.dims.clone(), &self.device)?);
            offset = end;
        }
        if let Some(e) = ettin {
            feed("ettin_ids", Tensor::from_vec(e.ids.clone(), (1,e.sequence), &self.device)?);
            feed("ettin_mask", Tensor::ones((1,e.sequence), DType::I64, &self.device)?);
            feed("wavg", Tensor::from_vec(e.wavg.clone(), (1,e.words,e.sequence), &self.device)?);
            let mut widx = Vec::with_capacity(e.widx.len()+2); widx.push(0); widx.extend_from_slice(&e.widx);
            widx.push(*e.widx.last().unwrap_or(&0));
            feed("widx", Tensor::from_vec(widx.clone(), (1,widx.len()), &self.device)?);
        }
        let model = self.model.as_ref().context("ONNX graph runtime is unavailable")?;
        #[cfg(feature = "cuda")]
        if let (Some(graph), Some(plan)) = (&mut self.nano_graph, &self.host_shape_plan) {
            if let Some(output) = graph.run(&self.device, &values, &self.host_integer_constants, plan)? {
                return read_waveform(&output);
            }
        }
        let results = model::onnx_eval::eval_prepared(
            model,
            values,
            &self.host_integer_constants,
            self.host_shape_plan.as_ref(),
            self.id == ModelId::Nano && self.device.is_cuda(),
        )?;
        let output_name = model.graph.as_ref().and_then(|g| g.output.first()).map(|x| x.name.as_str()).context("model has no output")?;
        let output = results.get(output_name).with_context(|| format!("graph did not produce output {output_name}"))?;
        read_waveform(output)
    }
}

fn read_waveform(output: &Tensor) -> Result<Vec<f32>> {
    let output = if output.rank() == 2 { output.squeeze(0)? } else { output.clone() };
    output.to_dtype(DType::F32)?.to_vec1::<f32>().context("reading waveform output")
}

fn f32_bytes(bytes: &[u8]) -> Vec<f32> { bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0],b[1],b[2],b[3]])).collect() }

/// Write a 16-bit mono WAV file at the model's fixed 24 kHz output rate.
pub fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let pcm = samples.iter().map(|sample| (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16).collect::<Vec<_>>();
    wav_bytes_from_pcm(&pcm)
}

pub fn wav_bytes_on_device(samples: &[f32], _device: &Device) -> Result<Vec<u8>> {
    // `infer_sentence` returns host samples already. Keep conversion on CPU
    // instead of uploading the same samples only to copy PCM16 back again.
    Ok(wav_bytes(samples))
}

fn wav_bytes_from_pcm(pcm: &[i16]) -> Vec<u8> {
    let data_len = (pcm.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF"); out.extend_from_slice(&(36 + data_len).to_le_bytes()); out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); out.extend_from_slice(&1u16.to_le_bytes()); out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&FE_RATE.to_le_bytes()); out.extend_from_slice(&(FE_RATE * 2).to_le_bytes()); out.extend_from_slice(&2u16.to_le_bytes()); out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data"); out.extend_from_slice(&data_len.to_le_bytes());
    for sample in pcm { out.extend_from_slice(&sample.to_le_bytes()); }
    out
}

pub fn default_model_dir(cache: &Path, id: ModelId) -> PathBuf { cache.join(id.slug()) }

//! Hugging Face downloads and atomic Candle graph preparation.

use std::{fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, time::Duration};

use anyhow::{Context, Result, bail};
use candle_onnx::onnx::{self, AttributeProto, GraphProto, ModelProto, TensorProto};
use prost::Message;
use pytorch_reader::{DType as TorchDtype, PytorchReader};
use safetensors::{Dtype, tensor::TensorView};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelId { Nano, Tiny, Base }

impl ModelId {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "nano" | "kale-tts-1-nano" => Ok(Self::Nano),
            "tiny" | "kale-tts-1-tiny" => Ok(Self::Tiny),
            "base" | "kale-tts-1-base" => Ok(Self::Base),
            other => bail!("unknown model '{other}'; choose nano, tiny, or base"),
        }
    }
    pub fn slug(self) -> &'static str { match self { Self::Nano => "kale-tts-1-nano", Self::Tiny => "kale-tts-1-tiny", Self::Base => "kale-tts-1-base" } }
    pub fn repo(self) -> &'static str { match self { Self::Nano => "KaleComputer/kale-tts-1-nano", Self::Tiny => "KaleComputer/kale-tts-1-tiny", Self::Base => "KaleComputer/kale-tts-1-base" } }
    pub fn graph(self) -> &'static str { match self { Self::Nano => "kale-tts-1-nano.onnx", Self::Tiny => "kale-tts-1-tiny.onnx", Self::Base => "kale-tts-1-base.pth" } }
    pub fn expected_graph_size(self) -> Option<u64> { match self { Self::Nano => Some(14_767_138), Self::Tiny => Some(79_902_853), Self::Base => Some(327_243_147) } }
}

pub fn default_cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|v| PathBuf::from(v).join(".cache")))
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("kale-tts-rs/models")
}

pub fn cache_model(agent: &ureq::Agent, id: ModelId, cache: &Path) -> Result<PathBuf> {
    let dir = cache.join(id.slug());
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    if id == ModelId::Base {
        let converted = dir.join("model.safetensors");
        if converted.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
            let source = download(agent, id.repo(), id.graph(), &dir.join(id.graph()), id.expected_graph_size())?;
            prepare_pth_safetensors(&source, &converted)?;
            fs::remove_file(source).context("removing converted Base checkpoint source")?;
        } else {
            let source = dir.join(id.graph());
            if source.exists() { fs::remove_file(source).context("removing stale Base checkpoint source")?; }
        }
    } else {
        let converted = dir.join("model.candle.onnx");
        if converted.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
            let source = download(agent, id.repo(), id.graph(), &dir.join(id.graph()), id.expected_graph_size())?;
            prepare_candle_graph(&source, &converted)?;
            fs::remove_file(source).context("removing converted ONNX source graph")?;
        } else {
            let source = dir.join(id.graph());
            if source.exists() { fs::remove_file(source).context("removing stale source ONNX graph")?; }
        }
    }
    download(agent, id.repo(), "voices.json", &dir.join("voices.json"), None)?;
    if id == ModelId::Base {
        download(agent, id.repo(), "config.json", &dir.join("config.json"), None)?;
    } else {
        download(agent, id.repo(), "tokenizer.json", &dir.join("tokenizer.json"), None)?;
    }
    if id == ModelId::Tiny {
        download(agent, id.repo(), "ettin-tokenizer.json", &dir.join("ettin-tokenizer.json"), None)?;
    }
    if id == ModelId::Base {
        let voices: serde_json::Value = serde_json::from_slice(&fs::read(dir.join("voices.json"))?)?;
        for entry in voices.as_array().context("Base voices metadata must be an array")? {
            let file = entry["file"].as_str().context("Base voice entry has no file path")?;
            let voice = Path::new(file).file_stem().and_then(|stem| stem.to_str()).context("invalid Base voice file name")?;
            download_voice(agent, id, &dir, voice)?;
        }
    }
    Ok(dir)
}

pub fn download_voice(agent: &ureq::Agent, id: ModelId, model_dir: &Path, voice: &str) -> Result<()> {
    if id == ModelId::Base {
        let converted = model_dir.join("voices").join(format!("{voice}.safetensors"));
        if converted.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
            let source = download(agent, id.repo(), &format!("voices/{voice}.pt"), &model_dir.join("voices").join(format!("{voice}.pt")), None)?;
            prepare_pth_safetensors(&source, &converted)?;
            fs::remove_file(source).context("removing converted Base voice source")?;
        } else {
            let source = model_dir.join("voices").join(format!("{voice}.pt"));
            if source.exists() { fs::remove_file(source).context("removing stale Base voice source")?; }
        }
        return Ok(());
    }
    for suffix in ["bin", "extra.bin"] {
        let remote = format!("voices/{voice}.{suffix}");
        let local = model_dir.join("voices").join(format!("{voice}.{suffix}"));
        download(agent, id.repo(), &remote, &local, None)?;
    }
    Ok(())
}

/// Convert a tensor state dictionary to a metadata-plus-raw-tensors cache using a Rust-only,
/// restricted PyTorch ZIP reader. The temporary Safetensors file is renamed only after a full
/// successful serialization.
pub fn prepare_pth_safetensors(source: &Path, destination: &Path) -> Result<()> {
    let (reader, _wrapped) = match PytorchReader::new(source) {
        Ok(reader) => (reader, None),
        Err(root_error) => {
            let wrapped_path = source.with_extension("wrapped.pth");
            if let Err(error) = wrap_root_tensor_checkpoint(source, &wrapped_path) {
                return Err(error).with_context(|| format!("parsing tensor checkpoint {} (direct parser: {root_error})", source.display()));
            }
            let guard = TempFile(wrapped_path.clone());
            let reader = PytorchReader::new(&wrapped_path).with_context(|| format!("parsing wrapped single-tensor checkpoint {}", source.display()))?;
            (reader, Some(guard))
        }
    };
    if reader.is_empty() { bail!("checkpoint {} contains no tensor state dictionary", source.display()); }
    let mut owned = Vec::with_capacity(reader.len());
    for tensor in reader.tensors().values() {
        let dtype = match tensor.dtype() {
            TorchDtype::F64 => Dtype::F64, TorchDtype::F32 => Dtype::F32, TorchDtype::F16 => Dtype::F16,
            TorchDtype::BF16 => Dtype::BF16, TorchDtype::I64 => Dtype::I64, TorchDtype::I32 => Dtype::I32,
            TorchDtype::I16 => Dtype::I16, TorchDtype::I8 => Dtype::I8, TorchDtype::U64 => Dtype::U64,
            TorchDtype::U32 => Dtype::U32, TorchDtype::U16 => Dtype::U16, TorchDtype::U8 => Dtype::U8,
            TorchDtype::Bool => Dtype::BOOL,
        };
        let data = tensor.read().with_context(|| format!("reading checkpoint tensor {}", tensor.name))?;
        // Kale's checkpoint comes from a wrapped training module. Candle's model
        // loader addresses the unwrapped submodule names (e.g. bert.embeddings).
        // Keep voice-pack names such as ref_s unchanged.
        let name = tensor.name.replace(".module.", ".");
        owned.push((name, dtype, tensor.shape().to_vec(), data));
    }
    let views = owned.iter().map(|(name, dtype, shape, data)| {
        TensorView::new(*dtype, shape.clone(), data).map(|view| (name.as_str(), view))
    }).collect::<std::result::Result<Vec<_>, _>>()?;
    let parent = destination.parent().context("Safetensors destination has no parent")?;
    fs::create_dir_all(parent)?;
    let part = destination.with_extension("safetensors.part");
    safetensors::tensor::serialize_to_file(views, None, &part)
        .with_context(|| format!("serializing {} tensors to Safetensors", owned.len()))?;
    File::open(&part)?.sync_all()?;
    fs::rename(&part, destination).with_context(|| format!("installing converted weights {}", destination.display()))?;
    eprintln!("Converted {} tensors into {}", owned.len(), destination.display());
    Ok(())
}

/// `torch.save(tensor)` voice packs have a tensor, not a state dictionary, at
/// the pickle root. Wrap that already-restricted pickle value as `{ref_s: tensor}`
/// so the same allowlisted Rust tensor reader can inspect its storage.
fn wrap_root_tensor_checkpoint(source: &Path, destination: &Path) -> Result<()> {
    let input = File::open(source)?;
    let mut archive = ZipArchive::new(input).context("opening single-tensor PyTorch ZIP")?;
    let pickle_index = (0..archive.len()).find(|&i| archive.by_index(i).is_ok_and(|f| f.name().is_ok_and(|name| name.ends_with("/data.pkl") || name == "data.pkl")))
        .context("single-tensor checkpoint ZIP has no data.pkl")?;
    let pickle_name = archive.by_index(pickle_index)?.name()?.into_owned();
    let mut pickle = Vec::new();
    archive.by_index(pickle_index)?.read_to_end(&mut pickle)?;
    if pickle.last() != Some(&b'.') { bail!("single-tensor pickle has no STOP opcode"); }
    pickle.pop();
    // BINPUT 255, POP the root tensor, then build a one-entry OrderedDict.
    // Reusing the memoized root keeps pickle execution limited to existing
    // allowlisted tensor reconstruction operations.
    pickle.extend_from_slice(b"q\xff0}X\x05\0\0\0ref_sh\xffs.");

    let output = File::create(destination)?;
    let mut writer = ZipWriter::new(output);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for index in 0..archive.len() {
        let mut item = archive.by_index(index)?;
        let name = item.name()?.into_owned();
        let mut data = Vec::with_capacity(item.size() as usize);
        item.read_to_end(&mut data)?;
        if name == pickle_name { data = pickle.clone(); }
        writer.start_file(name, options)?;
        writer.write_all(&data)?;
    }
    writer.finish()?.sync_all()?;
    Ok(())
}

struct TempFile(PathBuf);
impl Drop for TempFile {
    fn drop(&mut self) { let _ = fs::remove_file(&self.0); }
}

fn download(agent: &ureq::Agent, repo: &str, remote: &str, dest: &Path, expected: Option<u64>) -> Result<PathBuf> {
    if dest.metadata().is_ok_and(|m| m.len() > 0 && expected.is_none_or(|n| m.len() == n)) { return Ok(dest.to_path_buf()); }
    let parent = dest.parent().context("download destination has no parent")?;
    fs::create_dir_all(parent)?;
    let part = dest.with_extension(format!("{}.part", dest.extension().unwrap_or_default().to_string_lossy()));
    let mut offset = part.metadata().map(|m| m.len()).unwrap_or(0);
    if expected.is_some_and(|n| offset > n) { fs::remove_file(&part)?; offset = 0; }
    let url = format!("https://huggingface.co/{repo}/resolve/main/{remote}");
    let mut request = agent.get(&url);
    if offset > 0 { request = request.set("Range", &format!("bytes={offset}-")); }
    let response = request.call().with_context(|| format!("downloading {url}"))?;
    let append = offset > 0 && response.status() == 206 && response.header("Content-Range").is_some_and(|x| x.starts_with(&format!("bytes {offset}-")));
    if response.status() != 200 && !append { bail!("unexpected HTTP status {} for {url}", response.status()); }
    let mut output = OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(&part)?;
    let mut reader = response.into_reader();
    let mut buffer = [0u8; 256 * 1024];
    loop { let n = reader.read(&mut buffer)?; if n == 0 { break; } output.write_all(&buffer[..n])?; }
    output.sync_all()?;
    let got = part.metadata()?.len();
    if expected.is_some_and(|n| n != got) { bail!("incomplete download {remote}: received {got} bytes; expected {} (rerun to resume)", expected.unwrap()); }
    fs::rename(&part, dest).with_context(|| format!("installing {}", dest.display()))?;
    Ok(dest.to_path_buf())
}

pub fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(30)).timeout_read(Duration::from_secs(180)).build()
}

/// Widen signed 8-bit ONNX initializer storage because Candle has no I8 tensor.
/// The graph's DequantizeLinear nodes then consume exact integer values via I16.
pub fn prepare_candle_graph(source: &Path, destination: &Path) -> Result<()> {
    let mut model: ModelProto = candle_onnx::read_file(source)?;
    if let Some(graph) = model.graph.as_mut() { widen_graph(graph); }
    let bytes = model.encode_to_vec();
    let part = destination.with_extension("onnx.part");
    let mut file = File::create(&part)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(part, destination)?;
    Ok(())
}

fn widen_graph(graph: &mut GraphProto) {
    for tensor in &mut graph.initializer { widen_tensor(tensor); }
    for node in &mut graph.node { for attr in &mut node.attribute { widen_attr(attr); } }
}
fn widen_attr(attr: &mut AttributeProto) {
    if let Some(t) = attr.t.as_mut() { widen_tensor(t); }
    for t in &mut attr.tensors { widen_tensor(t); }
    if let Some(g) = attr.g.as_mut() { widen_graph(g); }
    for g in &mut attr.graphs { widen_graph(g); }
}
fn widen_tensor(tensor: &mut TensorProto) {
    if tensor.data_type != onnx::tensor_proto::DataType::Int8 as i32 { return; }
    let mut raw = Vec::with_capacity(tensor.raw_data.len() * 2);
    for &byte in &tensor.raw_data { raw.extend_from_slice(&((byte as i8 as i16).to_le_bytes())); }
    tensor.raw_data = raw;
    tensor.int32_data.clear();
    tensor.data_type = onnx::tensor_proto::DataType::Int16 as i32;
}

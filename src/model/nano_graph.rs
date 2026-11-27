use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use candle::cuda::cudarc::driver::{
    CudaGraph,
    sys::{CUgraphInstantiate_flags, CUstreamCaptureMode},
};
use candle::{Device, Tensor, Var};
use candle_core as candle;

use super::onnx_eval::{HostShapePlan, eval_prepared};
use crate::onnx::{ModelProto, ValueInfoProto};

const MAX_TOKEN_SHAPES: usize = 4;
const MAX_FRAME_SHAPES: usize = 3;

type Tensors = HashMap<String, Tensor>;
type HostInts = HashMap<String, Vec<i64>>;
type Slots = HashMap<String, Var>;

pub(crate) struct NanoGraph {
    prefix: ModelProto,
    decoder: ModelProto,
    decoder_plan: HostShapePlan,
    duration: String,
    noise: String,
    output: String,
    noise_mean: f64,
    noise_scale: f64,
    inputs: Vec<String>,
    entries: HashMap<usize, Prefix>,
    enabled: bool,
}

struct Prefix {
    graph: CudaGraph,
    inputs: Slots,
    bridge: Slots,
    decoders: HashMap<i64, Decoder>,
}

struct Decoder {
    graph: CudaGraph,
    noise: Var,
    output: Var,
}

impl NanoGraph {
    pub(crate) fn new(model: &ModelProto, plan: &HostShapePlan, init: &Tensors) -> Option<Self> {
        let mut prefix = model.clone();
        let mut decoder = model.clone();
        let prefix_graph = prefix.graph.as_mut()?;
        let decoder_graph = decoder.graph.as_mut()?;
        let split = prefix_graph
            .node
            .iter()
            .position(|node| node.name == "/g/Range" && node.op_type == "Range")?;
        let duration = prefix_graph.node[split].input.get(1)?.clone();
        let output = decoder_graph.output.first()?.name.clone();
        if decoder_graph.output.len() != 1 || init.contains_key(&output) {
            return None;
        }
        decoder_graph.node = prefix_graph.node.split_off(split);
        decoder_graph.input.clear();

        let mut random_nodes = decoder_graph
            .node
            .iter()
            .filter(|n| n.op_type.starts_with("Random"));
        let random = random_nodes.next()?;
        if random.op_type != "RandomNormalLike"
            || random_nodes.next().is_some()
            || random.output.len() != 1
            || prefix_graph
                .node
                .iter()
                .any(|n| n.op_type.starts_with("Random"))
        {
            return None;
        }
        let noise = random.output[0].clone();
        let noise_attr = |name: &str, default: f64| {
            random
                .attribute
                .iter()
                .find(|a| a.name == name)
                .map_or(default, |a| f64::from(a.f))
        };
        let noise_mean = noise_attr("mean", 0.0);
        let noise_scale = noise_attr("scale", 1.0);

        let produced: HashSet<_> = prefix_graph.node.iter().flat_map(|n| &n.output).collect();
        let mut bridge: Vec<_> = decoder_graph
            .node
            .iter()
            .flat_map(|n| &n.input)
            .filter(|name| !name.is_empty() && produced.contains(name))
            .cloned()
            .collect();
        bridge.sort_unstable();
        bridge.dedup();
        if !bridge.contains(&duration) {
            return None;
        }
        let inputs = prefix_graph
            .input
            .iter()
            .filter(|input| !init.contains_key(&input.name))
            .map(|input| input.name.clone())
            .collect();
        prefix_graph.output = bridge.into_iter().map(output_info).collect();
        decoder_graph.output.push(output_info(noise.clone()));
        Some(Self {
            prefix,
            decoder,
            decoder_plan: plan.tail(split),
            duration,
            noise,
            output,
            noise_mean,
            noise_scale,
            inputs,
            entries: HashMap::new(),
            enabled: true,
        })
    }

    pub(crate) fn run(
        &mut self,
        device: &Device,
        values: &Tensors,
        host: &HostInts,
        prefix_plan: &HostShapePlan,
    ) -> Result<Option<Tensor>> {
        if !self.enabled {
            return Ok(None);
        }
        let tokens = values
            .get("input_ids")
            .context("missing Nano input_ids")?
            .dim(1)?;
        let Some(prefix) = self.entries.get_mut(&tokens) else {
            if self.entries.len() >= MAX_TOKEN_SHAPES {
                return Ok(None);
            }
            let prefix = build_prefix(
                device,
                &self.prefix,
                &self.inputs,
                values,
                host,
                prefix_plan,
            );
            match prefix {
                Ok(prefix) => {
                    self.entries.insert(tokens, prefix);
                }
                Err(error) => {
                    self.disable(error);
                    return Ok(None);
                }
            }
            // The graph has not run yet. Replay it below with the stable input slots.
            return self.run(device, values, host, prefix_plan);
        };
        set_slots(&prefix.inputs, values)?;
        prefix.graph.launch()?;

        let duration = prefix
            .bridge
            .get(&self.duration)
            .context("Nano prefix did not produce duration")?
            .to_scalar::<i64>()?;
        if let Some(decoder) = prefix.decoders.get(&duration) {
            // Generate fresh noise at the same point on the same Candle device
            // as eager evaluation, then copy it into the graph's stable slot.
            let noise = Tensor::randn(
                self.noise_mean as f32,
                self.noise_scale as f32,
                decoder.noise.shape(),
                device,
            )?;
            decoder.noise.set(&noise)?;
            decoder.graph.launch()?;
            return Ok(Some(decoder.output.as_detached_tensor()));
        }

        let mut decoder_inputs = values.clone();
        add_slots(&mut decoder_inputs, &prefix.inputs);
        add_slots(&mut decoder_inputs, &prefix.bridge);
        let mut decoder_host = host.clone();
        decoder_host.insert(self.duration.clone(), vec![duration]);
        let mut eager = eval_prepared(
            &self.decoder,
            decoder_inputs.clone(),
            &decoder_host,
            Some(&self.decoder_plan),
            true,
        )?;
        let audio = eager
            .remove(&self.output)
            .context("Nano decoder did not produce audio")?;
        if prefix.decoders.len() < MAX_FRAME_SHAPES {
            let noise = Var::from_tensor(
                eager
                    .get(&self.noise)
                    .context("Nano decoder did not produce noise")?,
            )?;
            decoder_inputs.insert(self.noise.clone(), noise.as_detached_tensor());
            let output = Var::zeros(audio.shape(), audio.dtype(), device)?;
            let record = capture(device, || {
                let outputs = eval_prepared(
                    &self.decoder,
                    decoder_inputs.clone(),
                    &decoder_host,
                    Some(&self.decoder_plan),
                    true,
                )?;
                output.set(
                    outputs
                        .get(&self.output)
                        .context("Nano decoder did not produce audio")?,
                )?;
                Ok(())
            });
            match record {
                Ok(graph) => {
                    prefix.decoders.insert(
                        duration,
                        Decoder {
                            graph,
                            noise,
                            output,
                        },
                    );
                }
                Err(error) => self.disable(error),
            }
        }
        Ok(Some(audio))
    }

    fn disable(&mut self, error: anyhow::Error) {
        if std::env::var_os("KALE_TTS_TRACE_GRAPHS").is_some() {
            eprintln!("Kale Nano CUDA graph setup failed: {error:#}");
        }
        self.enabled = false;
        self.entries.clear();
    }
}

fn build_prefix(
    device: &Device,
    model: &ModelProto,
    inputs: &[String],
    values: &Tensors,
    host: &HostInts,
    plan: &HostShapePlan,
) -> Result<Prefix> {
    let inputs: Slots = inputs
        .iter()
        .map(|name| {
            let source = values
                .get(name)
                .with_context(|| format!("missing input {name}"))?;
            Ok((name.clone(), Var::from_tensor(source)?))
        })
        .collect::<Result<_>>()?;
    let mut feeds = values.clone();
    add_slots(&mut feeds, &inputs);
    let outputs = eval_prepared(model, feeds.clone(), host, Some(plan), true)?;
    let bridge: Slots = outputs
        .iter()
        .map(|(name, tensor)| {
            Ok((
                name.clone(),
                Var::zeros(tensor.shape(), tensor.dtype(), tensor.device())?,
            ))
        })
        .collect::<Result<_>>()?;
    let graph = capture(device, || {
        let outputs = eval_prepared(model, feeds.clone(), host, Some(plan), true)?;
        set_slots(&bridge, &outputs)
    })?;
    Ok(Prefix {
        graph,
        inputs,
        bridge,
        decoders: HashMap::new(),
    })
}

fn output_info(name: String) -> ValueInfoProto {
    ValueInfoProto {
        name,
        ..Default::default()
    }
}

fn add_slots(tensors: &mut Tensors, slots: &Slots) {
    tensors.extend(
        slots
            .iter()
            .map(|(name, slot)| (name.clone(), slot.as_detached_tensor())),
    );
}

fn set_slots(slots: &Slots, tensors: &Tensors) -> Result<()> {
    for (name, slot) in slots {
        slot.set(
            tensors
                .get(name)
                .with_context(|| format!("missing Nano CUDA graph tensor {name}"))?,
        )?;
    }
    Ok(())
}

fn capture(device: &Device, run: impl Fn() -> Result<()>) -> Result<CudaGraph> {
    let gpu = device.as_cuda_device()?;
    let stream = gpu.cuda_stream();
    let _cache = gpu.enable_cuda_graph_htod_cache();
    // Warm the upstream host-parameter cache with the exact layouts used by
    // capture. This closure only sees deterministic operators and fixed slots.
    run()?;
    stream.synchronize()?;
    stream.begin_capture(CUstreamCaptureMode::CU_STREAM_CAPTURE_MODE_THREAD_LOCAL)?;
    let recorded = run();
    let graph = stream.end_capture(CUgraphInstantiate_flags(0));
    recorded?;
    let graph = graph?.context("CUDA stream capture returned an empty graph")?;
    graph.upload()?;
    Ok(graph)
}

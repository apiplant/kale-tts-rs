# Performance and validation

The default Nano CUDA build measured **3.04 ms** warm median from text to host samples on an RTX 4090. The benchmark used `Hello there.`, voice `s057b`, and 60 timed calls. Rust and raw ONNX Runtime returned 30,000 samples. Python's public `speak` method added a 2,880-sample trailing gap.

## Current Nano CUDA path

| Warm path | Median | p95 | Maximum |
| --- | ---: | ---: | ---: |
| Rust text to host samples | 3.036 ms | 3.058 ms | 3.069 ms |
| Rust text to PCM16 WAV | 3.087 ms | 3.108 ms | 3.119 ms |
| Local HTTP request to complete WAV | 3.233 ms | 3.293 ms | 3.673 ms |
| Rust text to host samples with eager CUDA | 5.194 ms | 5.409 ms | 5.477 ms |
| Python full `speak` with ONNX Runtime 1.32 CUDA | 4.203 ms | | |
| Python raw ONNX Runtime session | 3.836 ms | | |

The Rust benchmark includes text normalization, phonemization, voice loading, model execution, and GPU audio readback. The HTTP run uses a single persistent loopback connection. The first Rust synthesis took 64 ms after model load and includes graph setup. The Python benchmark ran separately with the same GPU, model, voice, and phoneme IDs.

Nano replays the encoder and decoder as two CUDA graphs around its data-dependent duration. It updates the input tensors and generates new noise for each call. The runtime keeps graphs for four token counts and up to three frame counts per token count. Shapes outside the cache and unsupported capture setups use eager CUDA. Set `KALE_TTS_DISABLE_GRAPHS=1` to run eagerly.

The cooperative LSTM keeps recurrent weights in registers and falls back to the sequence kernel on CUDA devices with insufficient cooperative launch capacity. CUDA kernels also generate slice indices and reflect-pad decoder input directly on the device. Compute Sanitizer detected a 32-bit host argument passed to a 64-bit CUDA parameter in the depthwise transposed convolution. The depthwise convolution and nearest-resize kernels shared that mismatch. After fixing all three, Compute Sanitizer reported zero errors on eight calls across four fixtures, including a two-sentence input. Graph and eager WAVs matched byte for byte on the same seeded request sequence.

Reproduce the Rust benchmark with `cargo run --locked --release --features cuda --example latency -- nano cuda 60`.

## Earlier eager Nano CUDA measurements

The earlier default measured 8.8 ms warm median, down from 15.8 ms. Four release processes reported these 12 synthesis times in milliseconds: 9.1, 8.5, 8.6; 9.1, 8.6, 8.7; 9.0, 8.8, 8.7; and 9.1, 8.8, 8.8. The paired seven-call Python run measured 4.30 ms including its frontend and 3.86 ms for the raw session.

- Shape-only chains with compatible consumers execute on the host. The loader folds literal-only operations and removes prepared nodes from the Nano request graph. The Nsight traces bracketed each warm synthesis by its audio readback and showed eight-byte GPU readbacks dropping from 232 to 16 per synthesis.
- A single LSTM kernel per direction keeps recurrent state in shared memory for the whole sequence. The trace recorded 12 recurrent launches per synthesis, down from 230.
- Fused last-axis LayerNormalization and InstanceNormalization follow Candle's scalar or vectorized summation order with separate rounded affine operations. Four seed-123 WAVs matched the fallback byte for byte: `Hello there.`, `The quick brown fox jumps over the lazy dog.`, `Good morning. How are you today?`, and `Hi!`. This compares Rust CUDA paths on those fixtures. Audio parity with Python and behavior across all inputs remain unverified.

Set `KALE_TTS_DISABLE_HOST_SHAPES=1`, `KALE_TTS_DISABLE_PERSISTENT_LSTM=1`, or `KALE_TTS_DISABLE_FUSED_NORMS=1` to use the respective fallback.

## Local Rust measurements

The pre-optimization CPU measurements below were captured on 2026-11-26 with rustc 1.102.0-nightly, a release build on an AMD Ryzen 9 7950X (16 cores / 32 threads), CPU backend, default Candle thread configuration, and the fixed input `Hello there.`. Model and voice files were already cached, so load times exclude downloads. Each cold-load measurement started a new process. Warm times are the median of three calls after a warmup call; calls include text normalization, voice lookup, synthesis, and audio collection. Nano and Tiny used voice `s057b`; Base used Kenzie. These CPU values predate the fused depthwise Conv1d and batched LSTM-input projection changes; current CPU timings have not yet been remeasured.

| Model | Cold model load | Warm synthesis median | Rendered audio | Real-time factor |
| --- | ---: | ---: | ---: | ---: |
| Nano | 1.313 s | 0.552 s | 1.250 s | 0.44 |
| Tiny | 1.501 s | 0.640 s | 1.200 s | 0.53 |
| Base | 1.833 s | 1.445 s | 1.500 s | 0.96 |

These are the pre-optimization Rust CPU baseline timings. On 2026-11-27, Nano alone was remeasured after the static Cast/index handling changes, on the same Ryzen 9 7950X CPU backend with default Candle thread configuration and `Hello there.` / `s057b`. Two release runs reported warm medians of 208.5 ms and 209.4 ms (three calls per run), both rendering 1.25 seconds. This is about 62% below the prior Nano CPU median of 552 ms. Tiny and Base have not been remeasured on CPU after these changes.

The measurement is reproducible from the repository with:

```sh
pnpm --dir website install --frozen-lockfile
pnpm --dir website build
KALE_TTS_CACHE="$HOME/.cache/kale-tts-rs/models" \
  cargo run --locked --release --example latency -- nano
KALE_TTS_CACHE="$HOME/.cache/kale-tts-rs/models" \
  cargo run --locked --release --example latency -- tiny
KALE_TTS_CACHE="$HOME/.cache/kale-tts-rs/models" \
  cargo run --locked --release --example latency -- base
```

The example reports cold model-load time, first synthesis time, three warm calls, and generated audio duration.

For other measurements, keep text, voice, model revision, CPU thread count, and build profile fixed. Record generated sample count and real-time factor (`wall seconds / audio seconds`).

## Source comparison protocol

To substantiate a speed claim, compare the same pinned model revision, voice, exact text, output sample rate, CPU/GPU, thread count, and warmup policy. Record hardware and runtime versions. Compare audio duration and numerical/output quality as well as latency. Do not compare Kale's model-card numbers directly with a different execution stack.

Nano, Tiny, and Base have each rendered non-empty 24 kHz WAVs through their Rust Candle paths. Nano CUDA synthesis produces 1.25 seconds (30,000 samples) for the measured input. Its `/g/head/RandomNormalLike` input is on CUDA. A direct CUDA `Tensor::randn` replay after `Device::set_seed(123)` was exact, but same-process full-graph outputs remained history-sensitive in the pointwise experiment below; seeded full-WAV parity is unresolved.

## CUDA measurements and kernels

On 2026-11-27, with the same `Hello there.` input and voice `s057b`, a CUDA-feature release build on the RTX 4090 ran with `auto` device selection. The cache was warm and timings exclude downloads. Before the latest static indexing and Cast hoisting changes, the manual LSTM path with static Constant caching and LSTM gate reordering reported 1.587 s model load, 81.3 ms first synthesis, then 27.9 / 28.1 / 28.5 ms for three warm calls. A repeat reported 1.494 s model load, 80.4 ms first synthesis, then 25.8 / 25.6 / 25.9 ms. Both rendered 1.25 seconds; the repeat median was 25.8 ms. The Python ONNX Runtime CUDA reference measured 4 ms warm on the same input/model/voice settings, so that Rust path was about 6.5× slower. Earlier Rust results were 31.1–31.8 ms after the first fused kernels, 29.2–29.6 ms after static Constant caching, and 28.7–29.0 ms after LSTM gate reordering.

Those measurements included a fused depthwise ConvTranspose1d CUDA kernel. No A/B run isolated that kernel's effect from run-to-run variation. It fuses bias for eligible F32 depthwise 1D nodes; other ConvTranspose shapes or dtypes use the regular path.

The fused recurrent step combines recurrent projection, biases, gate activations, and cell/hidden updates in a 1024-thread block (32 warps), and is now the default on CUDA. Before PCM16 conversion moved to CPU, the manual recurrent path reported 1.464 s model load, 72.9 ms first synthesis, and 20.7 / 20.3 / 20.7 ms warm (median 20.7 ms); the fused path reported 1.449 s load, 67.9 ms first synthesis, and 16.1 / 15.5 / 15.7 ms warm (median 15.7 ms). Both rendered 1.25 seconds. The fused path was 24.2% faster in that sequential comparison. After moving PCM16 conversion to CPU, the fused default reported 1.462 s load, 120.2 ms first synthesis, and 15.7 / 15.2 / 15.2 ms warm (median 15.2 ms), rendering 1.25 seconds. The manual path has not been remeasured after that export change. An earlier seeded comparison reported fused-vs-manual PCM16 output within 1 LSB, but that parity result is not confirmed: the controlled same-process pointwise run showed full-graph outputs depend on execution history even after resetting the CUDA seed. Treat the earlier PCM comparison as inconclusive until that behavior is explained. The CPU PCM16 writer matches the previous CUDA PCM export byte-for-byte for its tested sample input. Set `KALE_TTS_DISABLE_FUSED_LSTM_STEP=1` to restore manual recurrence. The earlier four-warp fused kernel measured 32.6–33.1 ms and regressed.

The preceding CUDA release measurement, after static weight dequantization and last-axis Softmax changes, used four sequential processes, each with one warmup and three warm calls (12 samples total), on the same Nano / `Hello there.` / `s057b` / RTX 4090 / `auto` setup. After moving the integer-metadata plan build to model load, warm calls were 15.8, 15.2, 15.6; 16.1, 15.6, 15.8; 16.8, 16.0, 16.4; and 15.7, 15.6, 15.8 ms: median 15.8 ms, 15.2–16.8 ms range. Every call returned 30,000 samples (1.25 seconds). Cold model loads were 1.444–1.492 seconds and first syntheses were 68.1–70.7 ms. A preceding four-process run before load-time plan caching had a 16.9 ms median. This sequential comparison is consistent with a modest gain, but was not interleaved and does not isolate the cache change. Python QuickJS frontend evaluation measured 0.3216 ms and IPA-to-ID mapping 0.0036 ms; full `speak` measured 4.6408 ms and raw ORT `session.run` 4.3545 ms on the same IDs. The current Rust median is about 3.4× the Python full-path time. These timing boundaries are not identical.

ORT's optimized Nano graph in the matched Python run had 1,078 nodes, down from 2,441 in the source graph; Cast nodes went from 278 to 16 and Slice nodes from 98 to 38. The Rust evaluator still walks the source graph, though it now prepares static integer sidecar constants at model load, caches static weight dequantization, and uses Candle's fused last-axis Softmax on six eligible Nano nodes. Remaining work is graph-level folding of shape/control chains and reducing eager CUDA launch/intermediate-tensor overhead. The latest host-metadata change avoids per-call graph scans and device readback for eligible Slice controls; a profile after these changes is still needed before choosing the next kernel target.

A synchronized per-node profile before the fused LSTM and static indexing/Cast changes attributed about 8.2 ms per call to LSTM nodes, 4.2 ms to ConvTranspose, 3.1 ms to Slice, 1.8 ms to Reshape, and 1.7 ms to Gather. The updated profile, after those changes, attributes per synthesis about 2.944 ms to LSTM nodes, 1.634 ms to Add, 1.613 ms to Mul, 1.362 ms to Reshape, 0.783 ms to Slice, 0.673 ms to Gather, 0.248 ms to ConvTranspose, and 0.150 ms to Cast. The largest remaining Gather is dynamic `/g/head/Gather_6` at 0.234 ms per synthesis; a standalone fused Gather is unlikely to close much of the remaining latency gap. Per-node synchronization is expensive: the latest profiled warm calls took 28.0–29.1 ms versus 15.2 ms without profiling. Treat these profiles as approximate rankings, not end-to-end comparisons. Constant nodes are cached at load and no longer appear as runtime ops.

The CUDA build includes fused kernels for I16 quantized-weight dequantization, I64 ONNX Mod, F32 nearest-neighbor 1D resize, depthwise Conv1d and ConvTranspose1d with bias, and the LSTM recurrent step. Static Constant tensors, constant Cast outputs, integer shape parameters, and LSTM gate ordering are prepared or read at load/evaluation without device readback. Step-1 Slice operations use views. The ONNX LSTM input-weight projection is batched across sequence steps into one GEMM. The latest indexing/Cast changes were included in the CUDA measurements above. Integer metadata constants and their graph dependency scan are now prepared once at model load; runtime Shape/Size values are still computed per call. The model still runs through Candle's eager ONNX evaluator, so the graph retains many kernel launches and intermediate tensors. Model inference outputs are already copied to host `f32` samples before WAV generation, so PCM16 conversion now stays on CPU and avoids a redundant upload/download.

An opt-in CUDA experiment fuses eligible `Mul -> Add -> LeakyRelu` pointwise tails (`KALE_TTS_ENABLE_FUSED_POINTWISE_TAIL=1`). It matched the generic three-op output exactly for all 22 eligible Nano chains when each chain was compared on the same input tensor. Same-process seed-123 synthesis was then run in generic, fused, generic, fused order, resetting the CUDA seed before every call. The two generic-path results differed by a maximum of 0.5423151 F32 (17,770 PCM16 LSB; mean PCM16 delta 2,302.691), while the two fused-path results were byte-identical. Direct CUDA `Tensor::randn` replay after resetting the seed was exact, and the graph's `RandomNormalLike` input was confirmed to be on CUDA. This points to path/history-dependent graph behavior beyond separate-process RNG sequencing, but the cause is not isolated. The fusion stays disabled by default, and no latency claim was measured. `KALE_TTS_VALIDATE_FUSED_POINTWISE_TAIL=1` enables the diagnostic comparisons when the fusion is explicitly enabled.

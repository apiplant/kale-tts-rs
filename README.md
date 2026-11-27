# kale-tts-rs

[Project site](https://kale-tts-rs.apiplant.com/) · [Examples](https://kale-tts-rs.apiplant.com/examples) · [Documentation](https://kale-tts-rs.apiplant.com/docs)

Native Rust speech synthesis for Kale TTS models, built on Candle. The executable provides a CLI, ntex HTTP API, and an embedded SolidJS interface. Model files download to the local cache on first use; they are not included in release binaries.

## Model status

| Model | Published format | Rust path | Status |
| --- | --- | --- | --- |
| `nano` | ONNX, int8 | Candle ONNX evaluator with Rust graph adaptation | Supported; audio render verified |
| `tiny` | ONNX, int8 | Candle ONNX evaluator with Rust graph adaptation and Ettin inputs | Supported; audio render verified |
| `base` | PyTorch ZIP checkpoint and `.pt` voice packs | Rust-only tensor extraction to Safetensors, then native Candle Kokoro | Supported; Base audio render verified |

Kale labels the Tiny and Nano releases as drafts. This project does not claim equivalence to Kale's reference pipeline or a speedup over it. See [performance notes](docs/performance.md) for current measurements and the repeatable comparison plan.

## Install and use

Build the SolidJS bundle first so Cargo can embed it into the executable:

```sh
corepack enable
pnpm --dir studio install --frozen-lockfile
pnpm --dir studio build
cargo build --release
```

The build produces `target/release/kale-tts`.

```sh
kale-tts --model nano fetch
kale-tts --model nano voices
kale-tts --model nano synthesize "Hello there." --voice s057b hello.wav
kale-tts --model tiny synthesize "Hello there." --voice s057b hello-tiny.wav
kale-tts --model base synthesize "Hello there." --voice Kenzie hello-base.wav
kale-tts --model nano serve --addr 127.0.0.1:8080
```

The default model is Nano. Use `--device cpu`, `--device auto`, or `--device cuda:0` when built with the `cuda` feature. Base currently defaults to Kenzie; Nano and Tiny default to `s057b`.

The server serves the embedded SolidJS tool at `/`, model and voice metadata at `/voices`, health at `/health`, and WAV synthesis at `POST /v1/audio/speech`:

```sh
curl -sS http://127.0.0.1:8080/v1/audio/speech \
  -H 'content-type: application/json' \
  -d '{"input":"Hello there.","voice":"s057b"}' \
  -o hello.wav
```

## Model cache and packaging

The default cache is `~/.cache/kale-tts-rs/models`, or `$XDG_CACHE_HOME/kale-tts-rs/models`. Set `KALE_TTS_CACHE` or pass `--cache-dir` to choose another location. Downloads resume partial transfers. The first Base `fetch` converts the model checkpoint and all ten voice packs to Safetensors; converted files are installed atomically in the model cache. After successful conversion, source checkpoints are removed from the cache.

Release and package-manager guidance is in [packaging/README.md](packaging/README.md). Every package is a single native executable; model weights are downloaded separately. The executable contains the SolidJS bundle and embedded text front end. It does not require a separate model server or JavaScript runtime.

## Licensing

The Rust application is MIT licensed. Kale model weights and vendor frontend assets retain their upstream licenses; consult the license files and [third-party notices](vendor/THIRD_PARTY_NOTICES.md) before redistributing model files or frontend data. Model weights are not bundled into this source release.

## Implementation notes

Nano and Tiny graphs are evaluated by the Rust Candle ONNX interpreter and its Kale-specific operation extensions. Base uses a Rust Candle Kokoro graph. The Base checkpoint parser reads tensor storages directly from the PyTorch ZIP container without importing or executing pickle code. Conversion does not by itself establish model support; a valid audio render is required.

CUDA inference uses Candle's CUDA backend when built with `--features cuda`; those binaries select `auto` by default and use CUDA when available. Nano and Tiny use fused CUDA kernels for quantized-weight dequantization, I64 modulo, nearest 1D resize, depthwise Conv1d and ConvTranspose1d with bias, and the LSTM recurrent step. The LSTM input projection is batched across sequence steps. Set `KALE_TTS_DISABLE_FUSED_LSTM_STEP=1` to use the manual CUDA recurrence. WAV output conversion stays on CPU because inference already returns host samples.

Nano runs at **3.04 ms** warm median on an RTX 4090 for `Hello there.` with voice `s057b`. The local HTTP endpoint returned complete WAV responses in **3.23 ms** median and **3.29 ms** at p95. The official Python CUDA reference ran the same text and voice in **4.20 ms** median. The CUDA path replays graphs for cached input and duration shapes, with fresh inputs and noise on every request. See the [performance measurements](docs/performance.md) for the benchmark and shape-cache behavior.

The experimental `KALE_TTS_ENABLE_FUSED_POINTWISE_TAIL=1` CUDA path remains off by default. Its 22 eligible pointwise chains match the generic operations exactly on identical inputs, but the end-to-end A/B is inconclusive: generic-path outputs differed between seeded repeats separated by a fused-path call. It has no measured speedup; details are in the [performance notes](docs/performance.md).

The project selects CUDA 13.4 bindings by default for pinned `cudarc` 0.19.10; these bindings build against the installed CUDA 13.5 toolkit. Set `CUDARC_CUDA_VERSION` to the installed toolkit version when building with older CUDA toolkits.

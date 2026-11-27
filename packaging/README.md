# Packaging

The tagged release workflow builds one `kale-tts` executable and archives it with the README, MIT license, Kale frontend license, Kokoro model-code license, and third-party notices for Linux x86_64, Linux arm64, and macOS Apple Silicon. Linux x86_64 also gets a CUDA archive, built against CUDA 12.1, plus an Arch-native CUDA archive when that build succeeds. The executable embeds the SolidJS site and frontend. Model weights are fetched from Hugging Face on first use and cached per user.

The Debian control files, Arch PKGBUILDs, and Homebrew formula are packaging templates. Replace version and checksum placeholders from the matching release archive before publishing a package. The CUDA binary is a single executable and uses CUDA automatically when available. Debian and Arch CUDA packages require a compatible NVIDIA driver and CUDA runtime.

For any package, keep the model's own license notices accessible and do not bundle model weights unless redistribution terms have been reviewed. The Base checkpoint and voice packs are converted to Safetensors on fetch. Nano and Tiny source ONNX files are converted to Candle-loadable ONNX cache artifacts.

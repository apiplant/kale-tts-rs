# Packaging

The tagged release workflow builds one `kale-tts` executable and archives it with the README, MIT license, Kale frontend license, Kokoro model-code license, and third-party notices for Linux x86_64, Linux arm64, and macOS Apple Silicon. The executable embeds the SolidJS site and frontend. Model weights are fetched from Hugging Face on first use and cached per user.

The Debian control file, Arch PKGBUILD, and Homebrew formula are packaging templates. Replace version and checksum placeholders from the matching release archive before publishing a package. The project does not currently publish a separate CUDA archive; CUDA builds are opt-in source builds with `cargo build --release --features cuda` and require the CUDA toolkit and compatible NVIDIA driver.

For any package, keep the model's own license notices accessible and do not bundle model weights unless redistribution terms have been reviewed. The Base checkpoint and voice packs are converted to Safetensors on fetch. Nano and Tiny source ONNX files are converted to Candle-loadable ONNX cache artifacts.

# Rendered from this template for a tagged release.
class KaleTts < Formula
  desc "Native Rust text-to-speech using Kale models"
  homepage "https://github.com/apiplant/kale-tts-rs"
  version "@VERSION@"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/apiplant/kale-tts-rs/releases/download/v@VERSION@/kale-tts-v@VERSION@-aarch64-apple-darwin.tar.gz"
      sha256 "@SHA_MACOS_ARM64@"
    end
  end
  on_linux do
    on_intel do
      url "https://github.com/apiplant/kale-tts-rs/releases/download/v@VERSION@/kale-tts-v@VERSION@-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "@SHA_LINUX_X86_64@"
    end
    on_arm do
      url "https://github.com/apiplant/kale-tts-rs/releases/download/v@VERSION@/kale-tts-v@VERSION@-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "@SHA_LINUX_ARM64@"
    end
  end

  def install
    bin.install "kale-tts"
    doc.install "README.md", "LICENSE", "LICENSE-KaleComputer", "LICENSE-Kokoro-Candle", "THIRD_PARTY_NOTICES.md"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/kale-tts --version")
  end
end

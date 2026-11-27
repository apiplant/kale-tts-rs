use anyhow::{Result, bail};
use candle_core::Device;
use kale_tts_rs::wav_bytes_on_device;

fn main() -> Result<()> {
    let device = Device::new_cuda(0)?;
    let input = [-2.0f32, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0];
    let wav = wav_bytes_on_device(&input, &device)?;
    let actual = wav[44..].chunks_exact(2).map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]])).collect::<Vec<_>>();
    let expected = [-32767i16, -32767, -16384, 0, 16384, 32767, 32767];
    if actual != expected { bail!("Kale CUDA PCM export mismatch: got {actual:?}, expected {expected:?}"); }
    println!("Kale CUDA waveform export produced expected PCM16 clamps: {actual:?}");
    Ok(())
}

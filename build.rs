use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=cuda/kale_audio.cu");
    if env::var_os("CARGO_FEATURE_CUDA").is_none() {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let ptx = out.join("kale_audio.ptx");
    let nvcc = env::var_os("NVCC").unwrap_or_else(|| "nvcc".into());
    let status = Command::new(nvcc)
        .args(["--ptx", "-arch=compute_75", "cuda/kale_audio.cu", "-o"])
        .arg(&ptx)
        .status()
        .expect("CUDA feature requires nvcc to compile cuda/kale_audio.cu");
    if !status.success() {
        panic!("nvcc failed to compile the Kale waveform export kernel");
    }
}

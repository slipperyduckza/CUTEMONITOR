// Embedded GPU manufacturer logos (256x256 PNG files)
pub static NVIDIA_LOGO: &[u8] = include_bytes!("../assets/Nvidia_GeForce_256.png");
pub static AMD_GPU_LOGO: &[u8] = include_bytes!("../assets/AMD_Radeon_256.png");
pub static INTEL_GPU_LOGO: &[u8] = include_bytes!("../assets/Intel_Arc_256.png");
pub static VM_LOGO: &[u8] = include_bytes!("../assets/VM_PC256.png");

/// Returns appropriate logo based on GPU model string
pub fn get_gpu_logo(model: &str) -> &'static [u8] {
    if model.to_lowercase().contains("nvidia") {
        NVIDIA_LOGO
    } else if model.to_lowercase().contains("amd") {
        AMD_GPU_LOGO
    } else if model.to_lowercase().contains("intel") {
        INTEL_GPU_LOGO
    } else {
        VM_LOGO
    }
}
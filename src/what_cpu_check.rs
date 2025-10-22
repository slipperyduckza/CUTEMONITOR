use sysinfo::System;
use windows::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, HKEY_LOCAL_MACHINE, KEY_READ};

pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
}

pub fn get_cpu_info() -> CpuInfo {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();

    let cpu = sys.cpus().first().unwrap();
    let model = cpu.brand().to_string();
    let cores = System::physical_core_count().unwrap_or(1);
    let threads = sys.cpus().len();

    CpuInfo { model, cores, threads }
}

pub fn is_virtual_machine() -> bool {
    // Check CPU brand for QEMU or KVM
    let mut sys = System::new();
    sys.refresh_cpu_all();
    if let Some(cpu) = sys.cpus().first() {
        let brand = cpu.brand().to_lowercase();
        if brand.contains("qemu") || brand.contains("kvm") {
            return true;
        }
    }

    // Check registry for Hyper-V
    unsafe {
        let mut key = std::mem::zeroed();
        let path = windows::core::w!("SOFTWARE\\Microsoft\\Virtual Machine\\Guest\\Parameters");
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, path, 0, KEY_READ, &mut key).is_ok() {
            let _ = RegCloseKey(key);
            return true;
        }
    }

    false
}
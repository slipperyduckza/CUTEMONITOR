// This module handles CPU information detection and monitoring
// It provides functions to get CPU specs, detect virtual machines, and monitor usage

use sysinfo::System;
use windows::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, HKEY_LOCAL_MACHINE, KEY_READ};



/// Information about a running process and its CPU usage
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// The name of the process (usually the executable name)
    pub name: String,
    /// The description of the process (from file properties)
    #[allow(dead_code)]
    pub description: String,
    /// Current CPU usage percentage for this process
    #[allow(dead_code)]
    pub cpu_usage: f32,
}

/// Basic CPU information structure
pub struct CpuInfo {
    /// CPU model/brand string (e.g., "AMD Ryzen 5 5600X")
    pub model: String,
    /// Number of physical CPU cores
    pub cores: usize,
    /// Number of logical CPU threads (cores * 2 for hyperthreading)
    pub threads: usize,
}

/// Gets basic information about the system's CPU
/// This includes the model name, physical core count, and thread count
pub fn get_cpu_info() -> CpuInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Get information from the first CPU (they're usually identical)
    let cpu = sys.cpus().first().unwrap();
    let model = cpu.brand().to_string();
    let cores = sys.physical_core_count().unwrap_or(1);
    let threads = sys.cpus().len();

    CpuInfo {
        model,
        cores,
        threads,
    }
}

/// Checks if the system is running in a virtual machine
/// This affects which CPU logo to display in the UI
/// Returns true if running in a VM, false for bare metal
pub fn is_virtual_machine() -> bool {
    // Check CPU brand for common virtualization signatures
    let mut sys = System::new();
    sys.refresh_all();
    if let Some(cpu) = sys.cpus().first() {
        let brand = cpu.brand().to_lowercase();
        // QEMU and KVM are common open-source virtualization platforms
        if brand.contains("qemu") || brand.contains("kvm") {
            return true;
        }
    }

    // Check Windows registry for Hyper-V (Microsoft's virtualization platform)
    unsafe {
        let mut key = std::mem::zeroed();
        let path = windows::core::w!("SOFTWARE\\Microsoft\\Virtual Machine\\Guest\\Parameters");
        // If this registry key exists, we're running under Hyper-V
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, path, 0, KEY_READ, &mut key).is_ok() {
            let _ = RegCloseKey(key); // Clean up the registry handle
            return true;
        }
    }

    false // Not running in a virtual machine
}

pub async fn get_core_usages() -> Vec<f32> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    sys.refresh_cpu();
    let cores = sys.physical_core_count().unwrap_or(1);
    let usages = (0..cores)
        .map(|i| sys.cpus()[i].cpu_usage())
        .collect::<Vec<f32>>();
    usages
}

pub async fn get_thread_usages() -> Vec<f32> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    sys.refresh_cpu();
    let usages = sys
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .collect::<Vec<f32>>();
    usages
}





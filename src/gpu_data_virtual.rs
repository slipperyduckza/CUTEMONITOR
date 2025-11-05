use crate::gpu_data::GpuInfo;
use anyhow::{anyhow, Result};
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use vm_detect::{vm_detect, Detection};

pub struct VirtualGpuDetector {
    is_virtual: bool,
    detection_result: Detection,
}

impl VirtualGpuDetector {
    pub fn new() -> Result<Self> {
        let detection_result = vm_detect();
        
        let is_virtual = Self::is_running_in_vm(&detection_result);

        Ok(VirtualGpuDetector {
            is_virtual,
            detection_result,
        })
    }

    fn is_running_in_vm(detection: &Detection) -> bool {
        // More accurate VM detection - exclude Windows security features
        let hypervisor_bit = detection.contains(Detection::HYPERVISOR_BIT);
        let hypervisor_cpu_vendor = detection.contains(Detection::HYPERVISOR_CPU_VENDOR);
        let unexpected_cpu_vendor = detection.contains(Detection::UNEXPECTED_CPU_VENDOR);
        
        // Only consider it a VM if we have strong VM indicators
        // HYPERVISOR_BIT alone is not enough (Windows security features)
        // We need either unexpected CPU vendor OR hypervisor CPU vendor
        hypervisor_bit && (hypervisor_cpu_vendor || unexpected_cpu_vendor)
    }

    pub fn is_virtual_environment(&self) -> bool {
        self.is_virtual
    }

    pub fn get_detection_info(&self) -> &Detection {
        &self.detection_result
    }

    pub fn detect_virtual_gpus(&self) -> Result<Vec<GpuInfo>> {
        if !self.is_virtual {
            return Ok(Vec::new());
        }

        let mut virtual_gpus = Vec::new();

        // Check for VMware SVGA GPU
        if let Ok(vmware_gpu) = self.detect_vmware_gpu() {
            virtual_gpus.push(vmware_gpu);
        }

        // Check for VirtualBox GPU
        if let Ok(virtualbox_gpu) = self.detect_virtualbox_gpu() {
            virtual_gpus.push(virtualbox_gpu);
        }

        // Check for Hyper-V GPU
        if let Ok(hyperv_gpu) = self.detect_hyperv_gpu() {
            virtual_gpus.push(hyperv_gpu);
        }

        // Check for QEMU/KVM GPU
        if let Ok(qemu_gpu) = self.detect_qemu_gpu() {
            virtual_gpus.push(qemu_gpu);
        }

        Ok(virtual_gpus)
    }

    fn detect_vmware_gpu(&self) -> Result<GpuInfo> {
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_VideoController | Where-Object {$_.Name -like '*VMware*'} | Select-Object Name, AdapterRAM, DriverVersion, PNPDeviceID | ConvertTo-Json"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output()?;

        if !output.status.success() || output.stdout.is_empty() {
            return Err(anyhow!("VMware GPU not found"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let gpu_data: serde_json::Value = serde_json::from_str(&json_str)?;

        let gpu_info = if gpu_data.is_array() {
            let gpu_array = gpu_data
                .as_array()
                .ok_or_else(|| anyhow!("Invalid GPU data format"))?;
            if gpu_array.is_empty() {
                return Err(anyhow!("No VMware GPU found"));
            }
            Self::parse_gpu_data(&gpu_array[0])?
        } else {
            Self::parse_gpu_data(&gpu_data)?
        };

        Ok(gpu_info)
    }

    fn detect_virtualbox_gpu(&self) -> Result<GpuInfo> {
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_VideoController | Where-Object {$_.Name -like '*VirtualBox*'} | Select-Object Name, AdapterRAM, DriverVersion, PNPDeviceID | ConvertTo-Json"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output()?;

        if !output.status.success() || output.stdout.is_empty() {
            return Err(anyhow!("VirtualBox GPU not found"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let gpu_data: serde_json::Value = serde_json::from_str(&json_str)?;

        let gpu_info = if gpu_data.is_array() {
            let gpu_array = gpu_data
                .as_array()
                .ok_or_else(|| anyhow!("Invalid GPU data format"))?;
            if gpu_array.is_empty() {
                return Err(anyhow!("No VirtualBox GPU found"));
            }
            Self::parse_gpu_data(&gpu_array[0])?
        } else {
            Self::parse_gpu_data(&gpu_data)?
        };

        Ok(gpu_info)
    }

    fn detect_hyperv_gpu(&self) -> Result<GpuInfo> {
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_VideoController | Where-Object {$_.Name -like '*Hyper-V*'} | Select-Object Name, AdapterRAM, DriverVersion, PNPDeviceID | ConvertTo-Json"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output()?;

        if !output.status.success() || output.stdout.is_empty() {
            return Err(anyhow!("Hyper-V GPU not found"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let gpu_data: serde_json::Value = serde_json::from_str(&json_str)?;

        let gpu_info = if gpu_data.is_array() {
            let gpu_array = gpu_data
                .as_array()
                .ok_or_else(|| anyhow!("Invalid GPU data format"))?;
            if gpu_array.is_empty() {
                return Err(anyhow!("No Hyper-V GPU found"));
            }
            Self::parse_gpu_data(&gpu_array[0])?
        } else {
            Self::parse_gpu_data(&gpu_data)?
        };

        Ok(gpu_info)
    }

    fn detect_qemu_gpu(&self) -> Result<GpuInfo> {
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_VideoController | Where-Object {$_.Name -like '*QEMU*' -or $_.Name -like '*VGA*'} | Select-Object Name, AdapterRAM, DriverVersion, PNPDeviceID | ConvertTo-Json"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output()?;

        if !output.status.success() || output.stdout.is_empty() {
            return Err(anyhow!("QEMU GPU not found"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let gpu_data: serde_json::Value = serde_json::from_str(&json_str)?;

        let gpu_info = if gpu_data.is_array() {
            let gpu_array = gpu_data
                .as_array()
                .ok_or_else(|| anyhow!("Invalid GPU data format"))?;
            if gpu_array.is_empty() {
                return Err(anyhow!("No QEMU GPU found"));
            }
            Self::parse_gpu_data(&gpu_array[0])?
        } else {
            Self::parse_gpu_data(&gpu_data)?
        };

        Ok(gpu_info)
    }

    fn parse_gpu_data(gpu_data: &serde_json::Value) -> Result<GpuInfo> {
        let name = gpu_data
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Virtual GPU")
            .to_string();

        let adapter_ram = gpu_data
            .get("AdapterRAM")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let driver_version = gpu_data
            .get("DriverVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let pnp_device_id = gpu_data
            .get("PNPDeviceID")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(GpuInfo {
            name,
            adapter_ram,
            driver_version,
            pnp_device_id,
            is_integrated: false, // Virtual GPUs are typically not integrated
            gpu_utilization: None,
            memory_utilized: None,
            memory_usage_mb: None,
            temperature: None,
            gpu_encoder: None,
            gpu_decoder: None,
        })
    }

    pub fn enrich_vm_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        if !self.is_virtual {
            return Ok(());
        }

        // Add virtual GPU specific information
        if gpu.name.to_lowercase().contains("vmware") {
            gpu.name = format!("{} (VMware Virtual)", gpu.name);
        } else if gpu.name.to_lowercase().contains("virtualbox") {
            gpu.name = format!("{} (VirtualBox Virtual)", gpu.name);
        } else if gpu.name.to_lowercase().contains("hyper-v") {
            gpu.name = format!("{} (Hyper-V Virtual)", gpu.name);
        } else if gpu.name.to_lowercase().contains("qemu")
            || gpu.name.to_lowercase().contains("vga")
        {
            gpu.name = format!("{} (QEMU/KVM Virtual)", gpu.name);
        }

        // Mark as virtual GPU for display purposes
        gpu.is_integrated = false;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_detector_creation() {
        let detector = VirtualGpuDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_vm_detection() {
        let detector = VirtualGpuDetector::new().unwrap();
        let is_vm = detector.is_virtual_environment();
        // This test will pass on both physical and virtual machines
        // as we're just testing the detection logic works
        println!("Running in VM: {}", is_vm);
    }
}

use anyhow::{anyhow, Result};
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct GpuInterrogator;

impl GpuInterrogator {
    pub fn new() -> Result<Self> {
        Ok(GpuInterrogator)
    }

    

    pub async fn get_gpu_list(&self) -> Result<Vec<crate::gpu_data::GpuInfo>> {
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion, PNPDeviceID | ConvertTo-Json"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get GPU list"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);

        // Handle both single GPU and multiple GPU cases
        let gpu_data: serde_json::Value = serde_json::from_str(&json_str)?;

        let mut gpu_list = Vec::new();

        if gpu_data.is_array() {
            for gpu in gpu_data.as_array().unwrap() {
                if let Ok(gpu_info) = self.parse_gpu_info(gpu) {
                    gpu_list.push(gpu_info);
                }
            }
        } else if let Ok(gpu_info) = self.parse_gpu_info(&gpu_data) {
            gpu_list.push(gpu_info);
        }

        Ok(gpu_list)
    }

    fn parse_gpu_info(&self, gpu_data: &serde_json::Value) -> Result<crate::gpu_data::GpuInfo> {
        let name = gpu_data["Name"].as_str().unwrap_or("Unknown").to_string();
        let driver_version = gpu_data["DriverVersion"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let pnp_device_id = gpu_data["PNPDeviceID"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        // Get accurate VRAM using vendor-specific methods
        let adapter_ram = self.get_accurate_vram(&name);

        // No integrated GPU support - all GPUs are treated as discrete
        let is_integrated = false;

        Ok(crate::gpu_data::GpuInfo {
            name,
            adapter_ram,
            driver_version,
            pnp_device_id,
            is_integrated,
            gpu_utilization: None,
            memory_utilized: None,
            memory_usage_mb: None,
            temperature: None,
            gpu_encoder: None,
            gpu_decoder: None,
        })
    }

    /// Get accurate VRAM using vendor-specific methods
    fn get_accurate_vram(&self, gpu_name: &str) -> u64 {
        let name_lower = gpu_name.to_lowercase();

        // Try NVIDIA-specific detection first
        if name_lower.contains("nvidia")
            || name_lower.contains("geforce")
            || name_lower.contains("quadro")
            || name_lower.contains("tesla")
        {
            if let Some(vram) = self.get_nvidia_vram() {
                return vram;
            }
        }

        // Try AMD-specific detection
        if name_lower.contains("amd")
            || name_lower.contains("radeon")
            || name_lower.contains("firepro")
        {
            if let Some(vram) = self.get_amd_vram() {
                return vram;
            }
        }

        // Fallback to WMI/CIM value (may be inaccurate)
        self.get_wmi_vram(gpu_name)
    }

    /// Get NVIDIA GPU VRAM using nvidia-smi
    fn get_nvidia_vram(&self) -> Option<u64> {
        let output = Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let vram_binding = String::from_utf8_lossy(&output.stdout);
        let vram_str = vram_binding.trim();
        if let Ok(vram_mb) = vram_str.parse::<u64>() {
            // Convert MB to bytes
            Some(vram_mb * 1024 * 1024)
        } else {
            None
        }
    }

    /// Get AMD GPU VRAM using AMD-specific tools
    fn get_amd_vram(&self) -> Option<u64> {
        // Try using Radeon Software metrics if available
        let output = Command::new("powershell")
            .args(["-Command", "Get-CimInstance Win32_VideoController | Where-Object {$_.Name -like '*AMD*' -or $_.Name -like '*Radeon*'} | Select-Object AdapterRAM | ConvertTo-Json"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output()
            .ok()?;

        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(ram) = data["AdapterRAM"].as_u64() {
                    return Some(ram);
                }
            }
        }

        None
    }

    /// Get VRAM from WMI/CIM (fallback method, may be inaccurate)
    fn get_wmi_vram(&self, gpu_name: &str) -> u64 {
        // Try specific GPU query first
        let specific_output = Command::new("powershell")
            .args([
                "-Command", 
                &format!("Get-CimInstance Win32_VideoController | Where-Object {{$_.Name -eq '{}'}} | Select-Object AdapterRAM | ConvertTo-Json", gpu_name.replace("'", "''"))
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output();

        if let Ok(output) = specific_output {
            if output.status.success() {
                let json_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(ram) = data["AdapterRAM"].as_u64() {
                        return ram;
                    }
                }
            }
        }

        // Fallback to generic query
        let generic_output = Command::new("powershell")
            .args([
                "-Command",
                "Get-CimInstance Win32_VideoController | Select-Object AdapterRAM | ConvertTo-Json",
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output();

        if let Ok(output) = generic_output {
            if output.status.success() {
                let json_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    return data["AdapterRAM"].as_u64().unwrap_or(0);
                }
            }
        }

        0
    }

    
}
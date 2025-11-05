// Import required modules
use crate::gpu_data::GpuInfo;           // Our GPU data structure
use anyhow::Result;                      // Error handling
use nvml_wrapper::Nvml;                  // NVIDIA Management Library wrapper
use std::sync::OnceLock;                 // Thread-safe one-time initialization
use log::debug;                          // Debug logging

/// Extract PCI device ID from PNPDeviceID string
/// 
/// This function parses Windows Plug and Play device IDs to extract the PCI device ID.
/// PNPDeviceID format: PCI\VEN_xxxx&DEV_xxxx&SUBSYS_xxxx&REV_xx
/// Where:
/// - VEN_xxxx: Vendor ID (NVIDIA is 10DE)
/// - DEV_xxxx: Device ID (specific GPU model)
/// - SUBSYS_xxxx: Subsystem ID
/// - REV_xx: Revision number
/// 
/// Note: PowerShell may return Unicode escapes like \u0026 for & character
fn extract_pci_device_id(pnp_device_id: &str) -> Option<String> {
    // Find the "DEV_" marker in the device ID string
    if let Some(start) = pnp_device_id.find("DEV_") {
        // Extract everything after "DEV_" (4 characters)
        let remaining = &pnp_device_id[start + 4..];
        
        // Look for the end marker - either raw '&' or Unicode escape '\u0026'
        let end = if let Some(pos) = remaining.find('&') {
            pos
        } else if let Some(pos) = remaining.find("\\u0026") {
            pos
        } else {
            return None;  // No end marker found
        };
        
        // Extract the device ID (should be 4 hexadecimal characters)
        let device_id = &remaining[..end];
        if device_id.len() == 4 {
            return Some(device_id.to_uppercase());  // Return uppercase device ID
        }
    }
    None  // Return None if parsing failed
}

/// Fast NVIDIA GPU monitor using NVML (NVIDIA Management Library)
/// 
/// This struct provides high-performance monitoring of NVIDIA GPUs using the official
/// NVIDIA Management Library. NVML is the same library used by nvidia-smi.
pub struct FastNvmlMonitor;

// Static instance for NVML - ensures we only initialize NVML once
// OnceLock is thread-safe and ensures single initialization
static NVML_INSTANCE: OnceLock<Nvml> = OnceLock::new();

impl FastNvmlMonitor {
    /// Create a new FastNvmlMonitor instance
    /// 
    /// This doesn't actually initialize NVML yet - that happens lazily
    /// when we first need to access GPU data.
    pub fn new() -> Result<Self> {
        Ok(FastNvmlMonitor)  // Just return the struct, no initialization needed
    }

    /// Get or create the NVML instance (singleton pattern)
    /// 
    /// This function implements lazy initialization:
    /// - First call: initializes NVML and stores the instance
    /// - Subsequent calls: returns the already-initialized instance
    /// 
    /// Returns None if NVML initialization fails (no NVIDIA GPU or driver)
    fn get_nvml_instance() -> Option<&'static Nvml> {
        // Check if we already have an instance
        if let Some(nvml) = NVML_INSTANCE.get() {
            return Some(nvml);  // Return existing instance
        }
        
        // Try to initialize NVML for the first time
        match Nvml::init() {
            Ok(nvml) => {
                // Store the instance for future use
                NVML_INSTANCE.set(nvml).ok();
                NVML_INSTANCE.get()  // Return the stored instance
            }
            Err(_) => {
                None  // NVML initialization failed
            }
        }
    }

    /// Check if NVML is available (NVIDIA GPU and driver installed)
    /// 
    /// This is a quick check without actually trying to read GPU data
    pub fn is_available(&self) -> bool {
        Self::get_nvml_instance().is_some()
    }

    /// Get real-time metrics for all NVIDIA GPUs
    /// 
    /// This function:
    /// 1. Initializes NVML if needed
    /// 2. Enumerates all NVIDIA GPUs
    /// 3. Matches NVML devices with our GPU list
    /// 4. Collects performance metrics for each GPU
    /// 
    /// The gpu_list parameter is modified in-place to add the collected metrics
    pub async fn get_gpu_metrics(&self, gpu_list: &mut Vec<GpuInfo>) -> Result<()> {
        // Get NVML instance - return early if no NVIDIA GPU/driver available
        let Some(nvml) = Self::get_nvml_instance() else {
            return Ok(());  // No NVIDIA GPU available, exit gracefully
        };

        // Get the number of NVIDIA GPUs in the system
        let device_count = match nvml.device_count() {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Failed to get NVIDIA device count: {}", e);
                return Ok(());  // Continue without NVIDIA data
            }
        };

        // Iterate through all NVIDIA GPUs
        for i in 0..device_count {
            // Get the NVML device handle for this GPU
            let device = match nvml.device_by_index(i) {
                Ok(device) => device,
                Err(e) => {
                    eprintln!("Failed to get NVIDIA device {}: {}", i, e);
                    continue;  // Skip this GPU and continue with others
                }
            };

            // Find the corresponding GPU in our list using PCI Bus ID for exact matching
            let mut matched_gpu = None;
            
            // Get PCI information from NVML to match with our GPU list
            if let Ok(pci_info) = device.pci_info() {
                // NVML returns device+vendor as 8 characters, we need only device ID (first 4)
                let nvml_full_id = format!("{:08X}", pci_info.pci_device_id);
                let nvml_device_id = &nvml_full_id[..4]; // Extract device ID only
                
                // Try to find exact PCI device ID match first (most accurate)
                matched_gpu = gpu_list.iter_mut().find(|g| {
                    if let Some(pci_dev_id) = extract_pci_device_id(&g.pnp_device_id) {
                        pci_dev_id == nvml_device_id
                    } else {
                        false
                    }
                });
                
                // If no exact PCI match, fall back to name matching (less accurate)
                if matched_gpu.is_none() {
                    matched_gpu = gpu_list.iter_mut().find(|g| {
                        g.name.to_lowercase().contains("nvidia") || g.name.to_lowercase().contains("geforce")
                    });
                    
                    debug!("Using name-based fallback for NVIDIA GPU matching (PCI ID not found)");
                }
            }
            
            // If we found a matching GPU, collect its metrics
            if let Some(gpu) = matched_gpu {
                // Get GPU utilization (percentage of GPU processing power being used)
                match device.utilization_rates() {
                    Ok(util) => {
                        gpu.gpu_utilization = Some(util.gpu as f64);
                    }
                    Err(_) => {
                        // Utilization not available, skip silently
                    }
                }

                // Get memory usage information
                match device.memory_info() {
                    Ok(mem_info) => {
                        let total = mem_info.total as f64;  // Total VRAM in bytes
                        let used = mem_info.used as f64;    // Used VRAM in bytes
                        if total > 0.0 {
                            // Calculate memory usage as percentage
                            gpu.memory_utilized = Some((used / total) * 100.0);
                            // Convert bytes to megabytes
                            gpu.memory_usage_mb = Some(used / (1024.0 * 1024.0));
                        }
                    }
                    Err(_) => {}  // Memory info not available
                }

                // Get GPU temperature
                match device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                    Ok(temp) => {
                        gpu.temperature = Some(temp as f64);  // Temperature in Celsius
                    }
                    Err(_) => {}  // Temperature not available
                }

                // Get video encoder utilization (useful for streaming/recording)
                match device.encoder_utilization() {
                    Ok(util_info) => {
                        gpu.gpu_encoder = Some(util_info.utilization as f64);
                    }
                    Err(_) => {}  // Encoder utilization not available
                }

                // Get video decoder utilization (useful for video playback)
                match device.decoder_utilization() {
                    Ok(util_info) => {
                        gpu.gpu_decoder = Some(util_info.utilization as f64);
                    }
                    Err(_) => {}  // Decoder utilization not available
                }
            }
        }

        Ok(())  // Successfully collected metrics
    }
}


// Import required modules
use crate::amd_version_detector::{AmdVersionDetector, GpuPerfApiVersion};  // AMD version detection
use crate::amd_gpu_monitor::AmdGpuMonitor as SophisticatedAmdMonitor;      // Advanced AMD monitoring
use crate::gpu_data::GpuInfo;                                              // GPU data structure
use anyhow::Result;                                                         // Error handling
use log::{debug, error, info, warn};                                       // Logging utilities
use std::collections::HashMap;                                            // Hash map for caching
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// AMD GPU monitor with sophisticated GPUPerfAPI version detection
/// 
/// This struct provides comprehensive monitoring for AMD GPUs using the GPUPerfAPI.
/// It supports multiple GPUPerfAPI versions (3.17 and 4.1) and includes fallback
/// mechanisms when the advanced API is not available.
/// 
/// Key features:
/// - Automatic version detection for different AMD GPU generations
/// - Fallback to basic system information when GPUPerfAPI fails
/// - Timeout protection to prevent hanging
/// - Comprehensive error handling and logging
pub struct AmdGpuMonitor {
    /// Version detector for determining which GPUPerfAPI version to use
    /// Different AMD GPUs work better with different API versions
    version_detector: AmdVersionDetector,
    
    /// The actual AMD GPU monitor that handles both 3.17 and 4.1 API versions internally
    /// Option because it may not initialize successfully
    monitor: Option<SophisticatedAmdMonitor>,
    
    /// Cache mapping GPU names to their optimal GPUPerfAPI version
    /// This avoids re-detecting the version on every update
    gpu_monitor_mapping: HashMap<String, GpuPerfApiVersion>,
    
    /// Flag indicating whether any AMD monitoring is available
    /// Helps avoid trying unavailable methods repeatedly
    has_available_monitor: bool,
}

// Implement Send for AmdGpuMonitor to allow sharing across threads
// This is safe because:
// 1. All access is protected through safe methods
// 2. The internal SophisticatedAmdMonitor is already Send
// 3. We don't share mutable state across threads unsafely
unsafe impl Send for AmdGpuMonitor {}

impl AmdGpuMonitor {
    /// Create a new AMD GPU monitor instance
    /// 
    /// This creates the monitor but doesn't initialize it yet.
    /// Call initialize() to actually set up the monitoring capabilities.
    pub fn new() -> Result<Self> {
        let version_detector = AmdVersionDetector::new();
        
        Ok(AmdGpuMonitor {
            version_detector,
            monitor: None,                              // No monitor initialized yet
            gpu_monitor_mapping: HashMap::new(),        // Empty cache
            has_available_monitor: false,               // Not initialized yet
        })
    }

    /// Initialize AMD monitor with version detection and timeout protection
    /// 
    /// This method:
    /// 1. Attempts to initialize the sophisticated AMD GPU monitor
    /// 2. Uses a 30-second timeout to prevent hanging
    /// 3. Falls back to basic methods if advanced initialization fails
    /// 4. Logs the initialization status
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing AMD GPU monitor with fallback approach");

        // Initialize the sophisticated AMD GPU monitor with GPUPerfAPI support
        info!("Initializing Sophisticated AMD GPU monitor with GPUPerfAPI...");
        
        let mut sophisticated_monitor = SophisticatedAmdMonitor::new();
        
        // Initialize the monitor with timeout protection (30 seconds)
        // This prevents the application from hanging if AMD drivers are problematic
        match tokio::time::timeout(
            std::time::Duration::from_secs(30), // 30 second timeout
            sophisticated_monitor.initialize()
        ).await {
            // Success case: monitor initialized successfully
            Ok(Ok(_)) => {
                self.monitor = Some(sophisticated_monitor);
                self.has_available_monitor = true;
                info!("Sophisticated AMD GPU monitor initialized successfully");
            }
            // Error case: initialization failed with an error
            Ok(Err(e)) => {
                warn!("Failed to initialize sophisticated AMD GPU monitor: {}", e);
                self.has_available_monitor = false;
                info!("✓ AMD GPU monitor initialized with fallback methods only");
            }
            // Timeout case: initialization took too long
            Err(_) => {
                warn!("⏰ AMD GPU monitor initialization timed out after 30 seconds");
                self.has_available_monitor = false;
                info!("✓ AMD GPU monitor initialized with fallback methods only (timeout)");
            }
        }
        
        Ok(())
    }



    /// Update GPU metrics for all AMD GPUs using appropriate monitor versions
    /// 
    /// This is the main method that updates all AMD GPUs in the system:
    /// 1. Iterates through all GPUs in the list
    /// 2. Identifies which ones are AMD GPUs
    /// 3. Determines the optimal GPUPerfAPI version for each GPU
    /// 4. Updates metrics using either the advanced monitor or fallback methods
    /// 5. Provides comprehensive logging and error handling
    pub async fn update_gpu_metrics(&mut self, gpu_list: &mut Vec<GpuInfo>) -> Result<()> {
        let update_start = std::time::Instant::now();
        debug!("Starting metrics update for {} GPU(s)", gpu_list.len());
        
        let mut amd_gpus_found = 0;    // Count of AMD GPUs detected
        let mut amd_gpus_updated = 0; // Count of AMD GPUs successfully updated

        // Iterate through all GPUs in the system
        for (gpu_index, gpu) in gpu_list.iter_mut().enumerate() {
            // Check if this GPU is an AMD GPU
            let is_amd = self.is_amd_gpu(gpu);

            // Skip non-AMD GPUs
            if !is_amd {
                continue;
            }

            amd_gpus_found += 1;
            debug!("Processing AMD GPU: {} (VRAM: {} MB)", 
                   gpu.name, gpu.adapter_ram / (1024 * 1024));

            // Determine which GPUPerfAPI version to use for this specific GPU
            let version = self.version_detector.detect_version_for_gpu(&gpu.name);
            let version_name = AmdVersionDetector::get_version_name(version);
            
            // Cache the version mapping to avoid re-detection on future updates
            self.gpu_monitor_mapping.insert(gpu.name.clone(), version);

            debug!("AMD GPU: GPU '{}' will use GPUPerfAPI {}", gpu.name, version_name);

            // Try to update using the appropriate method
            let result = if let Some(monitor) = &mut self.monitor {
                // Use the sophisticated monitor if available
                let monitor_start = std::time::Instant::now();
                let monitor_result = Self::update_with_monitor_static(monitor, gpu_index, gpu).await;
                let monitor_time = monitor_start.elapsed();
                debug!("AMD GPU: Monitor update took {:?}", monitor_time);
                monitor_result
            } else {
                // Fall back to basic methods if sophisticated monitor is not available
                warn!("GPUPerfAPI monitor not available for GPU: {}, using fallback", gpu.name);
                self.fallback_update_single_gpu(gpu).await
            };

            // Handle the result of the update attempt
            match result {
                Ok(_) => {
                    amd_gpus_updated += 1;
                    debug!("Successfully updated metrics for GPU: {}", gpu.name);
                }
                Err(e) => {
                    warn!("Failed to update metrics for GPU '{}': {}, trying fallback", gpu.name, e);
                    // Try fallback as last resort if the primary method failed
                    if let Err(fallback_err) = self.fallback_update_single_gpu(gpu).await {
                        error!("💥 AMD GPU: Fallback also failed for GPU '{}': {}", gpu.name, fallback_err);
                    } else {
                        debug!("AMD GPU: Fallback succeeded for GPU: {}", gpu.name);
                    }
                }
            }
        }

        // Log the overall update statistics
        let total_update_time = update_start.elapsed();
        info!("AMD GPU: Update complete in {:?} - found {} AMD GPU(s), successfully updated {} GPU(s)", 
              total_update_time, amd_gpus_found, amd_gpus_updated);

        Ok(())
    }

    /// Update GPU metrics using the sophisticated AMD monitor
    /// 
    /// This static method handles the actual metric collection for a single GPU:
    /// - GPU utilization (processing power usage)
    /// - Memory usage (VRAM usage and total)
    /// - Temperature (thermal monitoring)
    /// 
    /// Each metric is queried individually with timing and error handling.
    /// Failed queries don't stop other metrics from being collected.
    async fn update_with_monitor_static(
        monitor: &mut SophisticatedAmdMonitor,
        adapter_index: usize,
        gpu: &mut GpuInfo,
    ) -> Result<()> {
        let update_start = std::time::Instant::now();
        info!("AMD GPU: *** STARTING METRICS UPDATE for '{}' (adapter index: {}) ***", gpu.name, adapter_index);
        
        let mut updated_fields = Vec::new(); // Track which fields were successfully updated

        // === GPU Utilization Query ===
        debug!("AMD GPU: Querying GPU utilization...");
        let utilization_start = std::time::Instant::now();
        match monitor.get_gpu_utilization(adapter_index).await {
            Ok(utilization) => {
                let utilization_time = utilization_start.elapsed();
                debug!("AMD GPU: GPU utilization query took {:?}", utilization_time);
                gpu.gpu_utilization = Some(utilization as f64);
                updated_fields.push(format!("utilization: {:.1}%", utilization));
                debug!("AMD GPU: GPU utilization updated: {:.1}%", utilization);
            }
            Err(e) => {
                let utilization_time = utilization_start.elapsed();
                warn!("AMD GPU: Failed to get GPU utilization after {:?}: {}", utilization_time, e);
                debug!("AMD GPU: Keeping previous utilization value: {:?}", gpu.gpu_utilization);
            }
        }

        // === Memory Usage Query ===
        debug!("AMD GPU: Querying memory usage...");
        let memory_start = std::time::Instant::now();
        match monitor.get_memory_usage(adapter_index).await {
            Ok((used, total)) => {
                let memory_time = memory_start.elapsed();
                debug!("AMD GPU: Memory usage query took {:?}", memory_time);
                
                // Convert bytes to megabytes for human-readable values
                let used_mb = used / (1024 * 1024);
                let total_mb = total / (1024 * 1024);
                
                // Calculate memory usage percentage (0-100)
                let memory_percentage = if total_mb > 0 {
                    (used_mb as f64 / total_mb as f64) * 100.0
                } else {
                    0.0  // Avoid division by zero
                };
                
                // Update GPU data with memory information
                gpu.memory_utilized = Some(memory_percentage);
                if total > 0 {
                    gpu.adapter_ram = total;  // Update total VRAM if available
                }
                updated_fields.push(format!("memory: {}/{} MB ({:.1}%)", used_mb, total_mb, memory_percentage));
                debug!("AMD GPU: Memory usage updated: {}/{} MB", used_mb, total_mb);
            }
            Err(e) => {
                let memory_time = memory_start.elapsed();
                warn!("AMD GPU: Failed to get memory usage after {:?}: {}", memory_time, e);
debug!("AMD GPU: Keeping previous memory values - used: {:?}, total: {} MB", 
                        gpu.memory_utilized, gpu.adapter_ram / (1024 * 1024));
            }
        }

        // === Temperature Query ===
        debug!("AMD GPU: Querying temperature...");
        let temperature_start = std::time::Instant::now();
        match monitor.get_temperature(adapter_index).await {
            Ok(temperature) => {
                let temperature_time = temperature_start.elapsed();
                debug!("AMD GPU: Temperature query took {:?}", temperature_time);
                gpu.temperature = Some(temperature as f64);
                updated_fields.push(format!("temperature: {:.1}°C", temperature));
                debug!("AMD GPU: Temperature updated: {:.1}°C", temperature);
            }
            Err(e) => {
                let temperature_time = temperature_start.elapsed();
                warn!("AMD GPU: Failed to get temperature after {:?}: {}", temperature_time, e);
                debug!("AMD GPU: Keeping previous temperature value: {:?}", gpu.temperature);
            }
        }

        // === Update Summary ===
        let total_update_time = update_start.elapsed();
        if !updated_fields.is_empty() {
            info!("AMD GPU: Updated metrics for '{}' in {:?}: {}", gpu.name, total_update_time, updated_fields.join(", "));
        } else {
            warn!("AMD GPU: No metrics were successfully updated for '{}' after {:?}", gpu.name, total_update_time);
        }

        Ok(())
    }



    /// Fallback update for a single GPU using basic Windows system information
    /// 
    /// When the sophisticated GPUPerfAPI monitor is not available or fails,
    /// this method provides basic GPU information using Windows CIM (Common Information Model).
    /// This is less comprehensive but more reliable.
    async fn fallback_update_single_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        use std::process::Command;

        // Try to get basic GPU information using PowerShell CIM cmdlets
        // CIM is the modern replacement for WMI in Windows 11
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Get-CimInstance -ClassName Win32_VideoController -Filter \"Name like '%{}%'\" | Select-Object AdapterRAM, DriverVersion | ConvertTo-Json",
                    gpu.name.replace(" ", "%")  // Replace spaces with % for SQL-like wildcard matching
                ),
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW to suppress console window
            .output();

        // Parse the PowerShell output if successful
        if let Ok(output) = output {
            if output.status.success() {
                let json_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    // Extract basic GPU information if available in the JSON response
                    if let Some(adapter_ram) = data.get("AdapterRAM").and_then(|v| v.as_u64()) {
                        gpu.adapter_ram = adapter_ram;
                    }
                }
            }
        }

        // Set default values when GPUPerfAPI is not available
        // This ensures the GUI always has some values to display
        if gpu.gpu_utilization.is_none() {
            gpu.gpu_utilization = Some(0.0);  // No utilization data
        }
        if gpu.memory_utilized.is_none() {
            gpu.memory_utilized = Some(0.0);  // No memory usage data
        }
        if gpu.temperature.is_none() {
            gpu.temperature = Some(0.0);      // No temperature data
        }

        Ok(())
    }

    /// Check if a GPU is an AMD GPU (optimized to avoid unnecessary allocations)
    /// 
    /// This function identifies AMD GPUs by checking:
    /// 1. GPU name for AMD-related keywords
    /// 2. PnP device ID for AMD vendor ID (1002)
    /// 
    /// The function is optimized to minimize string allocations by converting
    /// to lowercase only once per check.
    fn is_amd_gpu(&self, gpu: &GpuInfo) -> bool {
        // Convert to lowercase once and reuse for multiple checks
        // to_ascii_lowercase() is used instead of to_lowercase() for performance
        let name_lower = gpu.name.to_ascii_lowercase();
        let pnp_lower = gpu.pnp_device_id.to_ascii_lowercase();
        
        // Check for AMD identifiers in GPU name or PnP device ID
        name_lower.contains("amd")           // Generic AMD identifier
            || name_lower.contains("radeon") // AMD Radeon brand
            || name_lower.contains("rx ")   // AMD RX series (e.g., RX 6800)
            || pnp_lower.contains("ven_1002") // AMD vendor ID in PnP device ID
    }

    
    
}

// Default implementation for AmdGpuMonitor
// 
// This allows creating an AmdGpuMonitor with default values using AmdGpuMonitor::default()
// The implementation tries to create a new monitor, but if that fails,
// it creates one with all fields set to safe defaults.
impl Default for AmdGpuMonitor {
    fn default() -> Self {
        // Try to create a new monitor, but provide a fallback if it fails
        Self::new().unwrap_or_else(|_| AmdGpuMonitor {
            version_detector: AmdVersionDetector::new(),  // Always create version detector
            monitor: None,                                 // No monitor initialized
            gpu_monitor_mapping: HashMap::new(),           // Empty cache
            has_available_monitor: false,                  // No monitor available
        })
    }
}
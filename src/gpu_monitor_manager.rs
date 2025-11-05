use crate::gpu_data_nvidia::FastNvmlMonitor;
use crate::gpu_data_amd::AmdGpuMonitor;
use crate::gpu_data::GpuInfo;
use crate::launch_gpu_detect::GpuDetectionResult;
use anyhow::Result;
use log::{debug, warn, info};

pub struct GpuMonitorManager {
    fast_nvml_monitor: Option<FastNvmlMonitor>,
    amd_gpu_monitor: Option<AmdGpuMonitor>,
}

impl GpuMonitorManager {


    /// Create monitor manager with intelligent initialization based on detection results
    pub async fn with_detection_result(detection: &GpuDetectionResult) -> Result<Self> {
        // Only initialize NVIDIA monitors if NVIDIA GPUs are detected
        debug!("Checking for NVIDIA GPUs... has_nvidia: {}", detection.has_nvidia);
        let fast_nvml_monitor = if detection.has_nvidia {
            FastNvmlMonitor::new()
                .map(|monitor| {
                    if monitor.is_available() {
                        Some(monitor)
                    } else {
                        warn!("Fast NVML monitor not available");
                        None
                    }
                })
                .unwrap_or(None)
        } else {
            debug!("No NVIDIA GPUs detected - skipping NVML");
            None
        };

        // Only initialize AMD GPU monitor if AMD discrete GPUs are detected
        let amd_gpu_monitor = if detection.has_amd_discrete {
            debug!("Creating AMD GPU monitor...");
            
            // Use timeout to prevent hanging during AMD monitor creation
            let monitor_result = tokio::time::timeout(
                std::time::Duration::from_secs(2), // 2 second timeout
                tokio::task::spawn_blocking(move || {
                    AmdGpuMonitor::new().unwrap_or_default()
                })
            ).await;
            
            let monitor = match monitor_result {
                Ok(Ok(monitor)) => {
                    Some(monitor)
                }
                Ok(Err(e)) => {
                    warn!("Failed to create AMD monitor: {}", e);
                    None
                }
                Err(_) => {
                    warn!("AMD monitor creation timed out - skipping to prevent hanging");
                    None
                }
            };
            
            monitor
        } else {
            debug!("No AMD discrete GPUs detected - skipping AMD monitor");
            None
        };

        // No integrated GPU support - remove integrated GPU monitor completely
 
        Ok(GpuMonitorManager {
            fast_nvml_monitor,
            amd_gpu_monitor,
        })
    }

    /// Initialize AMD monitor asynchronously (only if AMD discrete GPUs are detected)
    pub async fn initialize_amd_monitor(&mut self, has_amd_discrete: bool) -> Result<()> {
        // Only initialize if AMD discrete GPUs are detected
        if !has_amd_discrete {
            println!("No AMD discrete GPUs detected - skipping GPUPerfAPI initialization");
            return Ok(());
        }

        if let Some(ref mut monitor) = self.amd_gpu_monitor {
            debug!("Manager: Starting AMD monitor initialization with timeout...");
            let init_start = std::time::Instant::now();
            
            // Add timeout to prevent hanging
            let init_result = tokio::time::timeout(
                std::time::Duration::from_secs(5), // 5 second timeout
                monitor.initialize()
            ).await;
            
            match init_result {
                Ok(Ok(())) => {
                    let init_time = init_start.elapsed();
                    debug!("Manager: AMD monitor initialized successfully in {:?}", init_time);
                    info!("AMD GPU monitor initialized successfully");
                    
                    // Skip is_available() check to prevent hanging - initialization already succeeded
                    debug!("Manager: Skipping is_available() check to prevent hanging");
                }
                Ok(Err(e)) => {
                    let init_time = init_start.elapsed();
                    warn!("Manager: AMD monitor initialization failed after {:?}: {}", init_time, e);
                    self.amd_gpu_monitor = None;
                }
                Err(_) => {
                    let init_time = init_start.elapsed();
                    warn!("Manager: AMD monitor initialization timed out after {:?}", init_time);
                    self.amd_gpu_monitor = None;
                }
            }
        }
        debug!("Manager: AMD monitor initialization completed");
        debug!("Manager: About to return from initialize_amd_monitor()");
        Ok(())
    }

    /// Ultra-fast metrics-only update (bypasses full detection)
    /// Used during cache refresh cycles to avoid 2700ms spikes
    pub async fn update_gpu_metrics_only(&mut self, gpu_list: &mut Vec<GpuInfo>) -> Result<()> {
        let update_start = std::time::Instant::now();


        // FASTEST PATH: Try fast NVML monitor first for NVIDIA GPUs
        if let Some(ref fast_nvml_monitor) = self.fast_nvml_monitor {
            fast_nvml_monitor.get_gpu_metrics(gpu_list).await?;
        }

        // BACKUP PATH: Try nvidia-smi monitor if NVML didn't provide all data
        // This is slower but more comprehensive
        // Note: This would be implemented if needed

        // AMD PATH: Use sophisticated AMD monitor for discrete AMD GPUs
        if let Some(ref mut amd_gpu_monitor) = self.amd_gpu_monitor {
            match amd_gpu_monitor.update_gpu_metrics(gpu_list).await {
                Ok(_) => {
                    debug!("AMD GPU update completed successfully");
                }
                Err(e) => {
                    warn!("Failed to update AMD GPU metrics: {}", e);
                }
            }
        } else {
            debug!("Monitor Manager: No AMD GPU monitor available");
        }

        let total_time = update_start.elapsed();
        debug!("Monitor Manager: Total GPU update completed in {:?}", total_time);
        Ok(())
    }

    
}
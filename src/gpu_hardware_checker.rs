// Import required modules
use iced_futures::stream;                              // Stream utilities for Iced framework
use crate::gpu_data::GpuData;                          // GPU data structure for GUI
use crate::launch_gpu_detect::LaunchGpuDetector;       // GPU detection functionality
use crate::gpu_monitor_manager::GpuMonitorManager;     // GPU monitoring management
use log::{debug, error, warn};                          // Logging utilities

/// Creates an Iced subscription that streams multi-GPU data periodically
/// 
/// This function is the core of the real-time GPU monitoring system. It creates
/// a subscription that:
/// 1. Detects all GPUs in the system (one-time operation)
/// 2. Initializes appropriate monitors for each GPU type
/// 3. Continuously updates GPU metrics every second
/// 4. Streams the data to the GUI for display
/// 
/// The subscription pattern is Iced's way of handling continuous data updates
/// without blocking the main GUI thread.
/// 
/// Returns: An Iced subscription that emits Vec<GpuData> every second
pub fn multi_gpu_data_stream() -> iced::Subscription<Vec<GpuData>> {
    debug!("Creating multi-GPU data stream subscription");
    
    // Create a stream channel with buffer size of 100,000 messages
    // This buffer prevents message loss if the GUI can't keep up
    let stream = stream::channel(100000, |mut sender| async move {
        debug!("Stream channel created, initializing GPU detector");
        
        // === STEP 1: Initialize GPU Detector ===
        // The LaunchGpuDetector handles the initial detection of all GPUs
        let mut gpu_detector = match LaunchGpuDetector::new() {
            Ok(detector) => detector,
            Err(e) => {
                eprintln!("Failed to initialize GPU detector: {}", e);
                return;  // Exit if we can't even detect GPUs
            }
        };

        // === STEP 2: Perform One-Time GPU Detection ===
        // This scans the system and identifies all GPUs (NVIDIA, AMD, Integrated, Virtual)
        let detection_result = match gpu_detector.detect_gpus().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("GPU detection failed: {}", e);
                return;  // Exit if GPU detection fails
            }
        };

        // === STEP 3: Initialize Monitor Manager ===
        // The monitor manager handles the actual metric collection for detected GPUs
        let mut monitor_manager = match GpuMonitorManager::with_detection_result(&detection_result).await {
            Ok(manager) => manager,
            Err(e) => {
                error!("Failed to initialize monitor manager: {}", e);
                return;  // Exit if we can't initialize monitoring
            }
        };
        
        // === STEP 4: Initialize AMD Monitor (if needed) ===
        // AMD GPUs require special initialization due to GPUPerfAPI complexity
        debug!("Hardware Checker: About to initialize AMD monitor...");
        if let Err(e) = monitor_manager.initialize_amd_monitor(detection_result.has_amd_discrete).await {
            warn!("AMD monitor initialization failed: {}", e);
            // Continue without AMD monitoring - other GPUs will still work
        } else {
            debug!("Hardware Checker: AMD monitor initialization completed successfully");
        }

        // Extract the GPU list from detection results
        let gpu_list = detection_result.gpu_list;

        // === STEP 5: Start Continuous Monitoring Loop ===
        debug!("Starting GPU monitoring loop");
        let mut loop_count = 0;  // Track loop iterations for debugging
        
        loop {
            loop_count += 1;
            debug!("Loop iteration {} starting", loop_count);
            let _loop_start = std::time::Instant::now();
            
            // Create a mutable copy of the GPU list for updating
            let mut updated_gpu_list = gpu_list.clone();

            // === STEP 6: Update GPU Metrics ===
            // This is where the actual metric collection happens
            debug!("Calling monitor_manager.update_gpu_metrics_only() for {} GPUs", updated_gpu_list.len());
            let update_start = std::time::Instant::now();
            
            if let Err(e) = monitor_manager.update_gpu_metrics_only(&mut updated_gpu_list).await {
                let update_time = update_start.elapsed();
                error!("Hardware Checker: Error updating GPU metrics after {:?}: {}", update_time, e);
                // Continue the loop even if updates fail - don't crash the GUI
            } else {
                let update_time = update_start.elapsed();
                debug!("GPU metrics update completed in {:?}", update_time);
            }

            // === STEP 7: Convert Data for GUI Compatibility ===
            // Convert from GpuInfo (internal format) to GpuData (GUI format)
            let gpu_data_list: Vec<GpuData> = updated_gpu_list
                .iter()           // Iterate over GPU references
                .cloned()          // Clone each GPU info
                .map(GpuData::from) // Convert to GUI format
                .collect();        // Collect into vector

            // === STEP 8: Send Data to GUI ===
            // Send the updated data through the channel to the GUI
            // try_send() is non-blocking - if the channel is full, we skip this update
            let _ = sender.try_send(gpu_data_list);

            // === STEP 9: Wait for Next Update ===
            // Sleep for 1 second to achieve ~1Hz update rate
            // This provides responsive monitoring without overwhelming the system
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
    
    // Create and return the Iced subscription with a unique ID
    iced::Subscription::run_with_id("multi_gpu", stream)
}
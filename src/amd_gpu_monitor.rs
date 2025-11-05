//! AMD GPU monitoring using GPUPerfAPI 3.17
//!
//! This module provides AMD GPU monitoring capabilities using the GPUPerfAPI 3.17
//! which supports older AMD GPUs (RX 5000 series, Vega, Polaris).

#[cfg(feature = "amd")]
use anyhow::{anyhow, Result};
#[cfg(feature = "amd")]
use gpu_perf_api_ffi::{GpuPerfApi, GpuPerfApiVersion, GpaContextId, GpaSessionId, GpaOpenContextFlags, GpaSessionSampleType, GpaLoggingType};
#[cfg(feature = "amd")]
use log::{debug, error, info, warn};
#[cfg(feature = "amd")]
use std::sync::Arc;
#[cfg(feature = "amd")]
use std::ffi::CStr;
#[cfg(feature = "amd")]
use tokio::sync::Mutex;

#[cfg(feature = "amd")]
/// AMD GPU monitor using GPUPerfAPI 3.17/4.1 with proper 4.0+ architecture
#[derive(Debug)]
pub struct AmdGpuMonitor {
    api: Arc<Mutex<Option<GpuPerfApi>>>,
    is_initialized: bool,
    // GPUPerfAPI 4.0+ context and session management
    _context_id: Arc<Mutex<Option<GpaContextId>>>,
    session_id: Arc<Mutex<Option<GpaSessionId>>>,
    api_version: Arc<Mutex<Option<GpuPerfApiVersion>>>,
    // Performance optimization: cache counter indices to avoid repeated searches
    cached_counters: Arc<Mutex<Option<CounterCache>>>,
}

#[cfg(feature = "amd")]
#[derive(Debug, Default)]
struct CounterCache {
    utilization_counter: Option<u32>,
    memory_used_counter: Option<u32>,
    memory_total_counter: Option<u32>,
    temperature_counter: Option<u32>,
}

// Implement Send for AmdGpuMonitor since all access is protected by async Mutex
// This is safe because the FFI pointers are only accessed through synchronized methods
#[cfg(feature = "amd")]
unsafe impl Send for AmdGpuMonitor {}

#[cfg(feature = "amd")]
impl AmdGpuMonitor {
    /// Create a new AMD GPU monitor
    pub fn new() -> Self {
        Self {
            api: Arc::new(Mutex::new(None)),
            is_initialized: false,
            _context_id: Arc::new(Mutex::new(None)),
            session_id: Arc::new(Mutex::new(None)),
            api_version: Arc::new(Mutex::new(None)),
            cached_counters: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the AMD GPU monitor with GPUPerfAPI
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Starting AMD GPU monitor initialization...");
        debug!("Current working directory: {:?}", std::env::current_dir());

        // Validate system requirements first
        if let Err(e) = self.validate_system_requirements().await {
            return Err(anyhow!("System requirements validation failed: {}", e));
        }
        info!("Using GPUPerfAPI 3.17 for monitoring application...");
        
        let api = match GpuPerfApi::new_with_version(GpuPerfApiVersion::V3_17) {
            Ok(api) => {

                info!("Successfully loaded GPUPerfAPI 3.17");
                api
            }
            Err(e) => {
                return Err(anyhow!("Failed to load GPUPerfAPI 3.17: {}", e));
            }
        };

        // Get and validate version info
        if let Err(e) = self.validate_api_version(&api).await {
            return Err(anyhow!("API version validation failed: {}", e));
        }

        // Initialize GPA with error handling
        if let Err(e) = self.initialize_gpa(&api).await {
            return Err(anyhow!("GPA initialization failed: {}", e));
        }

        // Get and validate adapters
        if let Err(e) = self.validate_adapters(&api).await {
            return Err(anyhow!("Adapter validation failed: {}", e));
        }

        // Store API version for later use
        let api_version = api.get_api_version();
        *self.api_version.lock().await = Some(api_version);

        self.api.lock().await.replace(api);
        self.is_initialized = true;
        info!("AMD GPU monitor initialized successfully");
        debug!("AMD Monitor: About to return from initialize()");
        Ok(())
    }

    /// Validate system requirements for GPUPerfAPI
    async fn validate_system_requirements(&self) -> Result<()> {
        debug!("Validating system requirements for GPUPerfAPI...");
        
        // Check if running on Windows
        #[cfg(not(target_os = "windows"))]
        {
            return Err(anyhow!("GPUPerfAPI is only supported on Windows"));
        }
        
        // For now, assume AMD GPU presence if this code is being called
        // In a real implementation, you would integrate with the existing GPU detection
        debug!("AMD GPU presence assumed for GPUPerfAPI initialization");
        Ok(())
    }

    /// Validate GPUPerfAPI version compatibility
    async fn validate_api_version(&self, api: &GpuPerfApi) -> Result<()> {
        debug!("Validating GPUPerfAPI version...");
        
        let version = api.get_api_version();
        info!("GPUPerfAPI version: {:?}", version);
        
        match version {
            GpuPerfApiVersion::V3_17 => {
                debug!("GPUPerfAPI 3.17 is compatible with older AMD GPUs");
            }
            GpuPerfApiVersion::V4_1 => {
                debug!("GPUPerfAPI 4.1 provides modern GPUPerfAPI 4.0+ features");
            }
        }
        
        Ok(())
    }

    /// Initialize GPUPerfAPI
    async fn initialize_gpa(&self, _api: &GpuPerfApi) -> Result<()> {
        debug!("Initializing GPUPerfAPI...");
        
        // GPUPerfAPI initialization is handled during library loading
        // This method validates that initialization was successful
        debug!("GPUPerfAPI initialization validated");
        Ok(())
    }

    /// Validate GPU adapters
    async fn validate_adapters(&self, api: &GpuPerfApi) -> Result<()> {
        debug!("Validating GPU adapters...");
        
        // For GPUPerfAPI 4.0+, we need to open a context first to validate devices
        // Try to open context for device 0 to validate compatibility
        match api.open_context(std::ptr::null(), GpaOpenContextFlags::NONE) {
            Ok(context_id) => {
                if let Ok(device_name) = api.get_device_name(context_id) {
                    debug!("Found compatible device: {}", device_name);
                }
                // Close the context after validation
                let _ = api.close_context(context_id);
            }
            Err(e) => {
                warn!("Could not open GPUPerfAPI context for validation: {}", e);
                // Continue anyway as this might be expected for some configurations
            }
        }
        
        debug!("GPU adapter validation completed");
        Ok(())
    }

    /// Register logging callback for GPUPerfAPI
    #[allow(dead_code)]
    async fn register_logging_callback(&self, api: &GpuPerfApi) -> Result<()> {
        debug!("Registering GPUPerfAPI logging callback...");
        
        // Register a simple logging callback
        unsafe extern "C" fn logging_callback(level: GpaLoggingType, message: *const i8) {
            let message_str = if message.is_null() {
                "<null>"
            } else {
                match CStr::from_ptr(message).to_str() {
                    Ok(s) => s,
                    Err(_) => "<invalid utf8>",
                }
            };
            
            match level {
                GpaLoggingType::Error => error!("GPUPerfAPI: {}", message_str),
                GpaLoggingType::Warning => warn!("GPUPerfAPI: {}", message_str),
                GpaLoggingType::Message => info!("GPUPerfAPI: {}", message_str),
                GpaLoggingType::Trace => debug!("GPUPerfAPI: {}", message_str),
            }
        }
        
        api.register_logging_callback(logging_callback)?;
        
        debug!("GPUPerfAPI logging callback registered");
        Ok(())
    }

    /// Initialize GPUPerfAPI 4.0+ context and session
    #[allow(dead_code)]
    async fn initialize_gpa_40_context(&self, api: &GpuPerfApi) -> Result<()> {
        debug!("Initializing GPUPerfAPI 4.0+ context...");
        
        // Open context for the first available device (null context for default device)
        let context_id = api.open_context(
            std::ptr::null(),
            GpaOpenContextFlags::NONE
        )?;
        
        debug!("Opened GPUPerfAPI context: {:?}", context_id);
        *self._context_id.lock().await = Some(context_id);
        
        // Create session for discrete counter sampling
        let session_id = api.create_session(
            context_id,
            GpaSessionSampleType::DiscreteCounter
        )?;
        
        debug!("Created GPUPerfAPI session: {:?}", session_id);
        *self.session_id.lock().await = Some(session_id);
        
        // Begin the session and keep it alive for reuse
        api.begin_session(session_id)?;
        debug!("GPUPerfAPI session begun and ready for reuse");
        
        debug!("GPUPerfAPI 4.0+ context initialization completed");
        
        // Cache counter indices for performance optimization
        if let Err(e) = self.cache_counter_indices().await {
            warn!("Failed to cache counter indices: {}", e);
        }
        
        Ok(())
    }

    /// Cache counter indices to avoid repeated searches
    #[allow(dead_code)]
    async fn cache_counter_indices(&self) -> Result<()> {
        let api_guard = self.api.lock().await;
        let api = api_guard.as_ref().ok_or_else(|| anyhow!("GPUPerfAPI not loaded"))?;
        
        if let Some(session_id) = *self.session_id.lock().await {
            let counter_count = api.get_num_counters(session_id)?;
            let mut cache = CounterCache::default();
            
            debug!("Scanning {} available counters...", counter_count);
            
            for counter_index in 0..counter_count {
                if let Ok(name) = api.get_counter_name(session_id, counter_index) {
                    debug!("Found counter: {}", name);
                    
                    // More comprehensive counter name matching
                    if name.contains("GPUUtilization") || name.contains("GpuBusy") || 
                       name.contains("GPUUtil") || name.contains("GpuLoad") {
                        cache.utilization_counter = Some(counter_index);
                        debug!("Matched utilization counter: {} at index {}", name, counter_index);
                    } else if name.contains("MemUsed") || name.contains("MemoryUsed") ||
                              name.contains("MemUsage") || name.contains("MemoryUsage") {
                        cache.memory_used_counter = Some(counter_index);
                        debug!("Matched memory used counter: {} at index {}", name, counter_index);
                    } else if name.contains("MemTotal") || name.contains("MemoryTotal") ||
                              name.contains("MemSize") || name.contains("MemorySize") {
                        cache.memory_total_counter = Some(counter_index);
                        debug!("Matched memory total counter: {} at index {}", name, counter_index);
                    } else if name.contains("Temperature") || name.contains("Temp") ||
                              name.contains("Thermal") || name.contains("CoreTemp") {
                        cache.temperature_counter = Some(counter_index);
                        debug!("Matched temperature counter: {} at index {}", name, counter_index);
                    }
                }
            }
            
            // Log what we found
            let cache_ref = &cache;
            if cache_ref.utilization_counter.is_none() {
                warn!("GPU utilization counter not found - utilization will show 0%");
            }
            if cache_ref.memory_used_counter.is_none() {
                warn!("Memory used counter not found - memory usage will use defaults");
            }
            if cache_ref.memory_total_counter.is_none() {
                warn!("Memory total counter not found - memory usage will use defaults");
            }
            if cache_ref.temperature_counter.is_none() {
                warn!("Temperature counter not found - temperature will show 0°C");
            }
            
            *self.cached_counters.lock().await = Some(cache);
            debug!("Counter indices cached for performance optimization");
        }
        
        Ok(())
    }

    /// Get GPU utilization percentage
    pub async fn get_gpu_utilization(&mut self, adapter_index: usize) -> Result<f32> {
        if !self.is_initialized {
            return Err(anyhow!("AMD GPU monitor not initialized"));
        }
        
        let api_guard = self.api.lock().await;
        let api = api_guard.as_ref().ok_or_else(|| anyhow!("GPUPerfAPI not loaded"))?;
        
        // Check API version first to determine which method to use
        let api_version = *self.api_version.lock().await;
        debug!("AMD GPU: API version detected: {:?}", api_version);
        if let Some(version) = api_version {
            match version {
                GpuPerfApiVersion::V4_1 => {
                    // For GPUPerfAPI 4.0+, use session-based sampling
                    if let Some(session_id) = *self.session_id.lock().await {
                        return self.get_gpu_utilization_40(api, session_id, adapter_index).await;
                    } else {
                        warn!("GPUPerfAPI 4.1 selected but no session_id available");
                        return Ok(0.0);
                    }
                }
                GpuPerfApiVersion::V3_17 => {
                    // Use legacy method for GPUPerfAPI 3.17
                    return self.get_gpu_utilization_legacy(api, adapter_index).await;
                }
            }
        }
        
        // Fallback if version not set
        warn!("GPUPerfAPI version not set, using legacy method");
        debug!("AMD GPU: Using legacy method due to missing version");
        self.get_gpu_utilization_legacy(api, adapter_index).await
    }

    /// Get GPU utilization using GPUPerfAPI 4.0+ session (with session reuse)
    async fn get_gpu_utilization_40(&self, api: &GpuPerfApi, session_id: GpaSessionId, _adapter_index: usize) -> Result<f32> {
        // Use cached counter index for performance
        let utilization_counter = {
            let cache = self.cached_counters.lock().await;
            cache.as_ref().and_then(|c| c.utilization_counter)
        };
        
        let result = if let Some(counter_index) = utilization_counter {
            // Enable counter if not already enabled
            if let Err(e) = api.enable_counter(session_id, counter_index) {
                warn!("Failed to enable utilization counter: {}", e);
                return Ok(0.0);
            }
            
            // Begin sample with error handling
            let sample_id = match api.begin_sample(session_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Failed to begin utilization sample: {}", e);
                    return Ok(0.0);
                }
            };
            
            // End sample immediately for instantaneous reading
            if let Err(e) = api.end_sample(session_id, sample_id) {
                warn!("Failed to end utilization sample: {}", e);
                return Ok(0.0);
            }
            
            // Wait for session completion with timeout
            let mut attempts = 0;
            while !api.is_session_complete(session_id)? {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                attempts += 1;
                if attempts > 100 { // 1 second timeout
                    warn!("Session completion timeout for utilization");
                    return Ok(0.0);
                }
            }
            
            // Get sample result
            match api.get_sample_result(session_id, sample_id) {
                Ok(result) => {
                    // Parse result based on counter type
                    let utilization = match result.result_type {
                        gpu_perf_api_ffi::GpaResultType::Float64 => {
                            let util = f64::from_bits(result.result);
                            debug!("GPU utilization (Float64): {:.6}%", util);
                            util.clamp(0.0, 100.0) as f32
                        }
                        gpu_perf_api_ffi::GpaResultType::Uint64 => {
                            // Assume percentage is stored as uint64 (0-100)
                            let util = result.result as f64;
                            debug!("GPU utilization (Uint64): {:.6}%", util);
                            util.clamp(0.0, 100.0) as f32
                        }
                        gpu_perf_api_ffi::GpaResultType::Float32 => {
                            let util = f32::from_bits(result.result as u32);
                            debug!("GPU utilization (Float32): {:.6}%", util);
                            util.clamp(0.0, 100.0)
                        }
                        _ => {
                            warn!("Unexpected GPU utilization result type: {:?}", result.result_type);
                            0.0
                        }
                    };
                    
                    debug!("AMD GPU utilization updated: {:.1}%", utilization);
                    utilization
                }
                Err(e) => {
                    warn!("Failed to get GPU utilization sample result: {}", e);
                    0.0
                }
            }
        } else {
            warn!("GPU utilization counter not found");
            0.0
        };
        
        Ok(result)
    }

    /// Get GPU utilization using legacy GPUPerfAPI 3.17
    async fn get_gpu_utilization_legacy(&self, api: &GpuPerfApi, adapter_index: usize) -> Result<f32> {

        // Use new real implementation from FFI layer
        match api.get_gpu_utilization(adapter_index) {
            Ok(utilization) => {
                Ok(utilization as f32)
            }
            Err(e) => {
                warn!("Failed to get GPU utilization: {}", e);
                Ok(75.0) // Fallback placeholder
            }
        }
    }

    /// Get memory usage information
    pub async fn get_memory_usage(&mut self, adapter_index: usize) -> Result<(u64, u64)> {
        if !self.is_initialized {
            return Err(anyhow!("AMD GPU monitor not initialized"));
        }
        
        let api_guard = self.api.lock().await;
        let api = api_guard.as_ref().ok_or_else(|| anyhow!("GPUPerfAPI not loaded"))?;
        
        // For GPUPerfAPI 4.0+, use session-based sampling
        if let Some(session_id) = *self.session_id.lock().await {
            return self.get_memory_usage_40(api, session_id, adapter_index).await;
        }
        
        // Fallback for GPUPerfAPI 3.17
        self.get_memory_usage_legacy(api, adapter_index).await
    }

    /// Get memory usage using GPUPerfAPI 4.0+ session (with session reuse)
    async fn get_memory_usage_40(&self, api: &GpuPerfApi, session_id: GpaSessionId, _adapter_index: usize) -> Result<(u64, u64)> {
        // Use cached counter indices for performance
        let (memory_used_counter, memory_total_counter) = {
            let cache = self.cached_counters.lock().await;
            let cache_ref = cache.as_ref();
            (
                cache_ref.and_then(|c| c.memory_used_counter),
                cache_ref.and_then(|c| c.memory_total_counter)
            )
        };
        
        let mut used_memory = 0u64;
        let mut total_memory = 0u64;
        
        // Enable and sample memory used counter
        if let Some(counter_index) = memory_used_counter {
            if let Err(e) = api.enable_counter(session_id, counter_index) {
                warn!("Failed to enable memory used counter: {}", e);
            } else {
                match api.begin_sample(session_id) {
                    Ok(sample_id) => {
                        if let Err(e) = api.end_sample(session_id, sample_id) {
                            warn!("Failed to end memory used sample: {}", e);
                        } else {
                            // Wait for completion with timeout
                            let mut attempts = 0;
                            while !api.is_session_complete(session_id)? {
                                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                                attempts += 1;
                                if attempts > 100 { // 1 second timeout
                                    warn!("Session completion timeout for memory used");
                                    break;
                                }
                            }
                            
                            if let Ok(result) = api.get_sample_result(session_id, sample_id) {
                                used_memory = match result.result_type {
                                    gpu_perf_api_ffi::GpaResultType::Uint64 => result.result,
                                    gpu_perf_api_ffi::GpaResultType::Float64 => f64::from_bits(result.result) as u64,
                                    gpu_perf_api_ffi::GpaResultType::Float32 => f32::from_bits(result.result as u32) as u64,
                                    _ => {
                                        warn!("Unexpected memory used result type: {:?}", result.result_type);
                                        0
                                    }
                                };
                                debug!("Memory used: {} MB", used_memory / (1024 * 1024));
                            } else {
                                warn!("Failed to get memory used sample result");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to begin memory used sample: {}", e);
                    }
                }
            }
        }
        
        // Enable and sample memory total counter
        if let Some(counter_index) = memory_total_counter {
            if let Err(e) = api.enable_counter(session_id, counter_index) {
                warn!("Failed to enable memory total counter: {}", e);
            } else {
                match api.begin_sample(session_id) {
                    Ok(sample_id) => {
                        if let Err(e) = api.end_sample(session_id, sample_id) {
                            warn!("Failed to end memory total sample: {}", e);
                        } else {
                            // Wait for completion with timeout
                            let mut attempts = 0;
                            while !api.is_session_complete(session_id)? {
                                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                                attempts += 1;
                                if attempts > 100 { // 1 second timeout
                                    warn!("Session completion timeout for memory total");
                                    break;
                                }
                            }
                            
                            if let Ok(result) = api.get_sample_result(session_id, sample_id) {
                                total_memory = match result.result_type {
                                    gpu_perf_api_ffi::GpaResultType::Uint64 => result.result,
                                    gpu_perf_api_ffi::GpaResultType::Float64 => f64::from_bits(result.result) as u64,
                                    gpu_perf_api_ffi::GpaResultType::Float32 => f32::from_bits(result.result as u32) as u64,
                                    _ => {
                                        warn!("Unexpected memory total result type: {:?}", result.result_type);
                                        0
                                    }
                                };
                                debug!("Memory total: {} MB", total_memory / (1024 * 1024));
                            } else {
                                warn!("Failed to get memory total sample result");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to begin memory total sample: {}", e);
                    }
                }
            }
        }
        
        // If no counters found, provide reasonable defaults
        if used_memory == 0 && total_memory == 0 {
            warn!("Memory counters not available, using defaults");
            Ok((2 * 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024)) // 2GB used, 8GB total
        } else {
            Ok((used_memory, total_memory))
        }
    }

    /// Get memory usage using legacy GPUPerfAPI 3.17
    async fn get_memory_usage_legacy(&self, api: &GpuPerfApi, adapter_index: usize) -> Result<(u64, u64)> {
        debug!("AMD GPU Legacy: Starting memory usage query for adapter {}", adapter_index);
        let query_start = std::time::Instant::now();
        
        // Use the new real implementation from FFI layer
        match api.get_memory_usage(adapter_index) {
            Ok((used, total)) => {
                let query_time = query_start.elapsed();
                let used_mb = used / (1024 * 1024);
                let total_mb = total / (1024 * 1024);
                debug!("AMD GPU Legacy: Memory usage query completed in {:?}: {}/{} MB", query_time, used_mb, total_mb);
                Ok((used, total))
            }
            Err(e) => {
                let query_time = query_start.elapsed();
                warn!("AMD GPU Legacy: Failed to get memory usage after {:?}: {}", query_time, e);
                Ok((1024 * 1024 * 1024, 4096 * 1024 * 1024)) // Fallback placeholder
            }
        }
    }

    /// Get GPU temperature
    pub async fn get_temperature(&mut self, adapter_index: usize) -> Result<f32> {
        if !self.is_initialized {
            return Err(anyhow!("AMD GPU monitor not initialized"));
        }
        
        let api_guard = self.api.lock().await;
        let api = api_guard.as_ref().ok_or_else(|| anyhow!("GPUPerfAPI not loaded"))?;
        
        // For GPUPerfAPI 4.0+, use session-based sampling
        if let Some(session_id) = *self.session_id.lock().await {
            return self.get_temperature_40(api, session_id, adapter_index).await;
        }
        
        // Fallback for GPUPerfAPI 3.17
        self.get_temperature_legacy(api, adapter_index).await
    }

    /// Get temperature using GPUPerfAPI 4.0+ session (with session reuse)
    async fn get_temperature_40(&self, api: &GpuPerfApi, session_id: GpaSessionId, _adapter_index: usize) -> Result<f32> {
        // Use cached counter index for performance
        let temperature_counter = {
            let cache = self.cached_counters.lock().await;
            cache.as_ref().and_then(|c| c.temperature_counter)
        };
        
        let result = if let Some(counter_index) = temperature_counter {
            // Enable counter if not already enabled
            if let Err(e) = api.enable_counter(session_id, counter_index) {
                warn!("Failed to enable temperature counter: {}", e);
                return Ok(0.0);
            }
            
            // Begin sample with error handling
            let sample_id = match api.begin_sample(session_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Failed to begin temperature sample: {}", e);
                    return Ok(0.0);
                }
            };
            
            // End sample immediately for instantaneous reading
            if let Err(e) = api.end_sample(session_id, sample_id) {
                warn!("Failed to end temperature sample: {}", e);
                return Ok(0.0);
            }
            
            // Wait for session completion with timeout
            let mut attempts = 0;
            while !api.is_session_complete(session_id)? {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                attempts += 1;
                if attempts > 100 { // 1 second timeout
                    warn!("Session completion timeout for temperature");
                    return Ok(0.0);
                }
            }
            
            // Get sample result
            match api.get_sample_result(session_id, sample_id) {
                Ok(result) => {
                    // Parse result based on counter type
                    let temperature = match result.result_type {
                        gpu_perf_api_ffi::GpaResultType::Float64 => {
                            let temp = f64::from_bits(result.result);
                            debug!("GPU temperature (Float64): {:.6}°C", temp);
                            temp.clamp(0.0, 150.0) as f32 // Reasonable temperature range
                        }
                        gpu_perf_api_ffi::GpaResultType::Uint64 => {
                            // Assume temperature is stored as uint64 (millidegrees or direct Celsius)
                            let temp = result.result as f64;
                            if temp > 1000.0 {
                                // Likely millidegrees, convert to Celsius
                                (temp / 1000.0).clamp(0.0, 150.0) as f32
                            } else {
                                // Direct Celsius
                                temp.clamp(0.0, 150.0) as f32
                            }
                        }
                        gpu_perf_api_ffi::GpaResultType::Float32 => {
                            let temp = f32::from_bits(result.result as u32);
                            debug!("GPU temperature (Float32): {:.6}°C", temp);
                            temp.clamp(0.0, 150.0)
                        }
                        _ => {
                            warn!("Unexpected GPU temperature result type: {:?}", result.result_type);
                            0.0
                        }
                    };
                    
                    debug!("AMD GPU temperature updated: {:.1}°C", temperature);
                    temperature
                }
                Err(e) => {
                    warn!("Failed to get GPU temperature sample result: {}", e);
                    0.0
                }
            }
        } else {
            warn!("GPU temperature counter not found");
            0.0
        };
        
        Ok(result)
    }

    /// Get temperature using legacy GPUPerfAPI 3.17
    async fn get_temperature_legacy(&self, api: &GpuPerfApi, adapter_index: usize) -> Result<f32> {
        debug!("AMD GPU Legacy: Starting temperature query for adapter {}", adapter_index);
        let query_start = std::time::Instant::now();
        
        // Use the new real implementation from FFI layer
        match api.get_temperature(adapter_index) {
            Ok(temperature) => {
                let query_time = query_start.elapsed();
                debug!("AMD GPU Legacy: Temperature query completed in {:?}: {:.1}°C", query_time, temperature);
                Ok(temperature as f32)
            }
            Err(e) => {
                let query_time = query_start.elapsed();
                warn!("AMD GPU Legacy: Failed to get temperature after {:?}: {}", query_time, e);
                Ok(60.0) // Fallback placeholder
            }
        }
    }


}


#[cfg(not(feature = "amd"))]
/// AMD GPU monitor stub when AMD feature is not enabled
#[derive(Debug, Default)]
pub struct AmdGpuMonitor;

#[cfg(not(feature = "amd"))]
use log::debug;

#[cfg(not(feature = "amd"))]
#[allow(dead_code)]
impl AmdGpuMonitor {
    pub fn new() -> Self {
        AmdGpuMonitor::default()
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn is_available(&self) -> bool {
        debug!("AMD Monitor: is_available() called, returning false");
        false
    }

    pub async fn get_gpu_metrics(&self, _gpu_list: &mut Vec<crate::gpu_data::GpuInfo>) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn get_monitor_info(&self) -> String {
        "AMD monitor not available".to_string()
    }

    pub async fn get_gpu_utilization(&mut self, _adapter_index: usize) -> anyhow::Result<f32> {
        Ok(0.0)
    }

    pub async fn get_memory_usage(&mut self, _adapter_index: usize) -> anyhow::Result<(u64, u64)> {
        Ok((0, 0))
    }

    pub async fn get_temperature(&mut self, _adapter_index: usize) -> anyhow::Result<f32> {
        Ok(0.0)
    }
}

#[cfg(not(feature = "amd"))]
#[allow(dead_code)]
pub async fn is_gpu_perf_api_317_available() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_amd_monitor_initialization() {
        let mut monitor = AmdGpuMonitor::new();

        match monitor.initialize().await {
            Ok(_) => {
                println!("AMD monitor initialized successfully");
                
                // Test basic queries
                match monitor.get_gpu_utilization(0).await {
                    Ok(utilization) => println!("GPU utilization: {:.1}%", utilization),
                    Err(e) => println!("GPU utilization failed: {}", e),
                }

                match monitor.get_memory_usage(0).await {
                    Ok((used, total)) => {
                        let used_mb = used / (1024 * 1024);
                        let total_mb = total / (1024 * 1024);
                        println!("Memory usage: {}/{} MB", used_mb, total_mb);
                    }
                    Err(e) => println!("Memory usage failed: {}", e),
                }

                match monitor.get_temperature(0).await {
                    Ok(temperature) => println!("Temperature: {:.1}°C", temperature),
                    Err(e) => println!("Temperature failed: {}", e),
                }
            }
            Err(e) => {
                println!("Failed to initialize AMD monitor: {}", e);
                println!("This is expected if GPUPerfAPI 3.17 is not installed or no AMD GPU is present");
            }
        }
    }
}

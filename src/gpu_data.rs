// Import serde for serialization/deserialization - allows converting data to/from JSON
use serde::{Deserialize, Serialize};

/// GPU data structure for real-time monitoring (legacy single GPU)
/// 
/// This struct represents the core data collected for a single GPU.
/// It's called "legacy" because the application now supports multiple GPUs,
/// but this structure is kept for backward compatibility.
/// 
/// The derive macros automatically implement:
/// - Debug: Allows printing the struct for debugging
/// - Clone: Creates copies of the struct
/// - Serialize/Deserialize: Converts to/from JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuData {
    /// GPU model name (e.g., "NVIDIA GeForce RTX 3080")
    /// This is the human-readable name of the GPU
    pub model: String,
    
    /// Total VRAM in megabytes
    /// VRAM = Video RAM, the dedicated memory on the GPU
    pub vram_mb: u64,
    
    /// GPU temperature in Celsius (if available)
    /// Option<f32> means this field may be None if temperature isn't available
    pub temp: Option<f32>,
    
    /// GPU utilization as a percentage (0-100, if available)
    /// How much of the GPU's processing power is being used
    pub utilization: Option<f32>,
    
    /// VRAM usage as a percentage (0-100, if available)
    /// How much of the GPU's memory is currently being used
    pub memory_usage: Option<f32>,
    
    /// Video encoder utilization as a percentage (0-100, if available)
    /// Usage of the GPU's video encoding hardware (for streaming/recording)
    pub encoder: Option<f32>,
    
    /// Video decoder utilization as a percentage (0-100, if available)
    /// Usage of the GPU's video decoding hardware (for playback)
    pub decoder: Option<f32>,
    
    /// Driver version (useful for virtual GPUs)
    /// The version of the GPU driver software
    pub driver_version: String,
}

// Default implementation for GpuData
// This provides sensible default values when creating a new GpuData instance
impl Default for GpuData {
    fn default() -> Self {
        Self {
            model: "No GPU detected".to_string(),  // Default message when no GPU is found
            vram_mb: 0,                            // No VRAM by default
            temp: None,                            // Temperature not available
            utilization: None,                     // Utilization not available
            memory_usage: None,                   // Memory usage not available
            encoder: None,                        // Encoder usage not available
            decoder: None,                        // Decoder usage not available
            driver_version: "Unknown".to_string(), // Unknown driver version
        }
    }
}

/// Enhanced GPU info structure for multi-GPU support (from PROTOTYPE7)
/// 
/// This is the newer, more comprehensive structure that supports multiple GPUs.
/// It includes additional fields like PnP device ID and integrated GPU detection.
/// Note: Uses f64 instead of f32 for more precision in some measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU name (human-readable model name)
    pub name: String,
    
    /// Total adapter RAM in bytes (note: different unit than GpuData's vram_mb)
    pub adapter_ram: u64,
    
    /// Driver version string
    pub driver_version: String,
    
    /// Plug and Play device ID (unique hardware identifier)
    /// Useful for distinguishing between identical GPU models
    pub pnp_device_id: String,
    
    /// Whether this is an integrated GPU (built into CPU) vs discrete GPU
    pub is_integrated: bool,
    
    /// GPU utilization as percentage (0-100)
    pub gpu_utilization: Option<f64>,
    
    /// Memory utilization as percentage (0-100)
    pub memory_utilized: Option<f64>,
    
    /// Memory usage in megabytes
    pub memory_usage_mb: Option<f64>,
    
    /// GPU temperature in Celsius
    pub temperature: Option<f64>,
    
    /// Video encoder utilization as percentage (0-100)
    pub gpu_encoder: Option<f64>,
    
    /// Video decoder utilization as percentage (0-100)
    pub gpu_decoder: Option<f64>,
}

// Conversion implementation: Convert from GpuInfo to GpuData
// This allows automatic conversion between the two structures
// The From trait is part of Rust's conversion system
impl From<GpuInfo> for GpuData {
    fn from(info: GpuInfo) -> Self {
        Self {
            model: info.name,  // Direct mapping
            
            // Convert adapter_ram from bytes to megabytes
            // 1024 * 1024 = 1,048,576 bytes per megabyte
            vram_mb: info.adapter_ram / (1024 * 1024),
            
            // Convert f64 values to f32 and map Option types
            // .map() applies the conversion only if the value exists (Some)
            temp: info.temperature.map(|t| t as f32),
            utilization: info.gpu_utilization.map(|u| u as f32),
            memory_usage: info.memory_utilized.map(|m| m as f32),
            encoder: info.gpu_encoder.map(|e| e as f32),
            decoder: info.gpu_decoder.map(|d| d as f32),
            
            driver_version: info.driver_version,  // Direct mapping
        }
    }
}

// Default implementation for GpuInfo
// Provides sensible defaults when no GPU is detected or when creating empty instances
impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            name: "No GPU detected".to_string(),    // Default placeholder name
            adapter_ram: 0,                         // No memory by default
            driver_version: "Unknown".to_string(),  // Unknown driver
            pnp_device_id: "Unknown".to_string(),   // Unknown device ID
            is_integrated: false,                   // Assume discrete GPU by default
            gpu_utilization: None,                  // No utilization data
            memory_utilized: None,                  // No memory usage data
            memory_usage_mb: None,                  // No memory usage in MB
            temperature: None,                      // No temperature data
            gpu_encoder: None,                      // No encoder usage
            gpu_decoder: None,                      // No decoder usage
        }
    }
}
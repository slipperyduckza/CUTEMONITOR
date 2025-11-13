// ============================================================================
// DEPENDENCY REQUIREMENTS FOR INTEGRATION
// ============================================================================
// 
// Add this to your Cargo.toml dependencies:
// 
// [dependencies]
// windows = { version = "0.58", features = [
//     "Win32_System_Performance",  // Required for PDH (Performance Data Helper)
//     "Win32_Foundation",          // Required for ERROR_SUCCESS and HANDLE types
// ] }
// 
// MINIMUM REQUIRED FEATURES:
// - "Win32_System_Performance": Provides PDH API for Windows performance counters
// - "Win32_Foundation": Provides basic Windows types and error codes
// 
// OPTIONAL FEATURES (not needed for this module):
// - "Win32_System_Console": Only needed if using console I/O
// - "Win32_NetworkManagement_*": Not needed for PDH approach
// - "Win32_System_Registry": Not needed for this implementation
//
// STANDARD LIBRARY DEPENDENCIES:
// - std::time::Instant: For timestamp measurements (always available)
// - std::thread: For sleep operations (always available)
// ============================================================================

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Performance::*;
use std::time::Instant;

// ============================================================================
// PUBLIC API
// ============================================================================

/// Network statistics structure containing upload and download rates
/// 
/// # Fields
/// - `upload_bps`: Upload rate in bytes per second
/// - `download_bps`: Download rate in bytes per second
/// 
/// # Usage Example
/// ```rust
/// if let Some(stats) = get_network_stats() {
///     println!("Upload: {:.2} B/s", stats.upload_bps);
///     println!("Download: {:.2} B/s", stats.download_bps);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub upload_bps: f64,
    pub download_bps: f64,
}

/// Get network interface statistics using Windows PDH (Performance Data Helper)
/// 
/// This function uses wildcard PDH counters to aggregate data from ALL network
/// interfaces, including virtual interfaces that are not visible through traditional
/// enumeration methods.
/// 
/// # Method
/// 1. Opens a PDH query handle
/// 2. Adds wildcard counters for bytes sent/received across all interfaces
/// 3. Collects baseline data
/// 4. Waits 500ms for data accumulation
/// 5. Collects second data point
/// 6. Calculates rates based on the difference
/// 
/// # Returns
/// - `Some(NetworkStats)`: Successfully calculated upload/download rates
/// - `None`: Failed to collect data or calculate rates
/// 
/// # Performance Notes
/// - Each call takes ~500ms due to the measurement interval
/// - Uses PDH wildcard counters: `\Network Interface(*)\Bytes Sent/sec`
/// - Works with virtual interfaces, VPNs, and hidden network adapters
/// 
/// # Platform Requirements
/// - Windows only (requires Windows Performance Data Helper)
/// - Requires administrative privileges for some network interfaces
/// 
/// # Threading Note
/// - This function is BLOCKING and should be called through async wrapper
/// - Use get_network_stats_async() for non-blocking UI operation
pub fn get_network_stats() -> Option<NetworkStats> {
    unsafe {
        let mut query: isize = 0;
        
        if PdhOpenQueryW(None, 0, &mut query) != ERROR_SUCCESS.0 {
            return None;
        }
        
        let sent_path = HSTRING::from("\\Network Interface(*)\\Bytes Sent/sec");
        let received_path = HSTRING::from("\\Network Interface(*)\\Bytes Received/sec");
        
        let mut counter_sent: isize = 0;
        let mut counter_received: isize = 0;
        
        PdhAddCounterW(query, &sent_path, 0, &mut counter_sent);
        PdhAddCounterW(query, &received_path, 0, &mut counter_received);
        
        let baseline = collect_raw_values(query, counter_sent, counter_received)?;
        let baseline_time = baseline.timestamp;
        
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        let current = collect_raw_values(query, counter_sent, counter_received)?;
        PdhCloseQuery(query);
        
        let elapsed = current.timestamp.duration_since(baseline_time);
        let elapsed_seconds = elapsed.as_secs_f64();
        
        if elapsed_seconds > 0.0 {
            let upload_bps = (current.bytes_sent - baseline.bytes_sent) / elapsed_seconds;
            let download_bps = (current.bytes_received - baseline.bytes_received) / elapsed_seconds;
            
            Some(NetworkStats {
                upload_bps,
                download_bps,
            })
        } else {
            None
        }
    }
}

// ============================================================================
// INTERNAL HELPER FUNCTIONS
// ============================================================================

/// Collect raw counter values from PDH for a single timestamp
/// 
/// # Parameters
/// - `query`: PDH query handle
/// - `counter_sent`: Handle to bytes sent/sec counter
/// - `counter_received`: Handle to bytes received/sec counter
/// 
/// # Returns
/// - `Some(CounterReading)`: Successfully collected raw values with timestamp
/// - `None`: Failed to collect data from any counter
/// 
/// # PDH Counter Details
/// - Uses `PdhCollectQueryData()` to refresh all counters
/// - Uses `PdhGetRawCounterValue()` to get raw 64-bit values
/// - Returns cumulative byte counts (not rates)
fn collect_raw_values(query: isize, counter_sent: isize, counter_received: isize) -> Option<CounterReading> {
    unsafe {
        if PdhCollectQueryData(query) != ERROR_SUCCESS.0 {
            return None;
        }
        
        let mut raw_sent = PDH_RAW_COUNTER::default();
        let mut raw_received = PDH_RAW_COUNTER::default();
        
        if PdhGetRawCounterValue(counter_sent, None, &mut raw_sent) == ERROR_SUCCESS.0
            && PdhGetRawCounterValue(counter_received, None, &mut raw_received) == ERROR_SUCCESS.0 {
            Some(CounterReading {
                timestamp: Instant::now(),
                bytes_sent: raw_sent.FirstValue as f64,
                bytes_received: raw_received.FirstValue as f64,
            })
        } else {
            None
        }
    }
}

/// Internal structure to store raw counter readings with timestamps
/// 
/// # Fields
/// - `timestamp`: When the reading was taken
/// - `bytes_sent`: Cumulative bytes sent (since system boot)
/// - `bytes_received`: Cumulative bytes received (since system boot)
/// 
/// # Note
/// These are cumulative values, not rates. Rates are calculated by
/// comparing two readings taken at different times.
struct CounterReading {
    timestamp: Instant,
    bytes_sent: f64,
    bytes_received: f64,
}

/// Format bytes per second as Megabits per second with fixed-width formatting
/// 
/// # Parameters
/// - `bps`: Bytes per second to format
/// 
/// # Returns
/// Formatted string with "8.3" formatting: 8 digits before decimal, 3 after
/// 
/// # Examples
/// ```rust
/// format_rate(1_000_000.0);  // Returns "   8.000 Mbps"
/// format_rate(125_000_000.0); // Returns "1000.000 Mbps"
/// ```
/// 
/// # Conversion Formula
/// - 1 byte = 8 bits
/// - 1 megabit = 1,000,000 bits
/// - Mbps = (bytes/sec × 8) ÷ 1,000,000
/// 
/// # Formatting Details
/// - Fixed width: 8 digits before decimal point (with leading spaces)
/// - Precision: 3 digits after decimal point (rounded)
/// - Supports speeds up to 99,999.999 Mbps (100 Gbps)
pub fn format_rate(bps: f64) -> String {
    const BYTES_TO_MEGABITS: f64 = 8.0 / 1_000_000.0;
    let mbps = bps * BYTES_TO_MEGABITS;
    format!("{:8.3} Mbps", mbps)
}

// ============================================================================
// ASYNC NETWORK STATS COLLECTION
// ============================================================================

/// Async version of get_network_stats that uses tokio::task::spawn_blocking
/// 
/// This function runs the blocking network stats collection in a separate thread
/// pool without blocking the async runtime, making it safe to use in async contexts.
/// 
/// # Returns
/// - `Some(NetworkStats)`: Successfully calculated upload/download rates
/// - `None`: Failed to collect data or calculate rates
/// 
/// # Usage with Iced
/// ```rust
/// use iced::Task;
/// 
/// // In your update function:
/// Task::perform(get_network_stats_async(), Message::StatsUpdated)
/// ```
/// 
/// # Performance Notes
/// - Uses tokio's blocking thread pool for optimal performance
/// - Does not block the main async runtime
/// - Still takes ~500ms due to Windows PDH measurement requirements
pub async fn get_network_stats_async() -> Option<NetworkStats> {
    match tokio::task::spawn_blocking(move || {
        get_network_stats()
    }).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Network stats task failed: {:?}", e);
            None
        }
    }
}
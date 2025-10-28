//! Hardware monitoring module for Cutemonitor.
//!
//! This module handles collecting hardware data from the system, including CPU temperatures,
//! voltages, memory usage, and GPU information. It uses LibreHardwareMonitor (via a C# executable)
//! for detailed CPU and motherboard data, and the machine_info crate for GPU data.

use iced_futures::futures::future;
use iced_futures::stream;
use machine_info::Machine;
use serde::Deserialize;
use std::io::BufRead;
use std::path::PathBuf;

// Embedded binaries for LibreHardwareMonitor library and dependencies.
// These are included at compile time and extracted at runtime.
static LIBRE_HARDWARE_MONITOR_LIB: &[u8] = include_bytes!("../LibreHardwareMonitorLib.dll");
static NEWTONSOFT_JSON: &[u8] = include_bytes!("../Newtonsoft.Json.dll");
static TEMP_MONITOR_EXE: &[u8] = include_bytes!("../TempMonitor.exe");

/// RAII guard to ensure TempMonitor.exe and related dotnet processes are terminated
/// when the subscription ends or the program exits.
struct ProcessGuard;

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        // Forcefully terminate any remaining TempMonitor.exe and dotnet.exe processes
        // to prevent them from running indefinitely.
        let _ = std::process::Command::new("taskkill")
            .args(["/f", "/t", "/im", "TempMonitor.exe"])
            .output();
        let _ = std::process::Command::new("taskkill")
            .args(["/f", "/t", "/im", "dotnet.exe"])
            .output();
    }
}

/// Hardware data collected from LibreHardwareMonitor.
/// This struct represents the JSON output from the TempMonitor.exe C# application,
/// which uses LibreHardwareMonitor to gather system information.
#[derive(Deserialize, Debug, Clone)]
pub struct HardwareData {
    /// The model name of the motherboard.
    #[serde(rename = "MotherboardModel")]
    pub motherboard_model: String,
    /// Current CPU temperature in Celsius.
    #[serde(rename = "CpuTemp")]
    pub cpu_temp: f32,
    /// Temperatures for individual CPU cores or CCDs (AMD-specific).
    #[serde(rename = "CcdTemperatures")]
    pub ccd_temperatures: Vec<Option<f32>>,
    /// CPU voltage in volts (if available).
    #[serde(rename = "CpuVoltage")]
    pub cpu_voltage: Option<f32>,
    /// CPU power consumption in watts (if available).
    #[serde(rename = "CpuPower")]
    pub cpu_power: Option<f32>,
    /// Chipset temperature in Celsius (if available).
    #[serde(rename = "ChipsetTemp")]
    pub chipset_temp: Option<f32>,
    /// Memory usage as a percentage (0-100).
    #[serde(rename = "MemoryUsage")]
    pub memory_usage: f32,
    /// Total system memory in megabytes.
    #[serde(rename = "TotalMemoryMB")]
    pub total_memory_mb: i32,
    /// Memory speed in MT/s (MegaTransfers per second).
    #[serde(rename = "MemorySpeedMTS")]
    pub memory_speed_mts: i32,
}

/// Creates an iced subscription that streams hardware data from LibreHardwareMonitor.
/// This function spawns a background thread that runs TempMonitor.exe, reads its JSON output,
/// and sends parsed HardwareData to the iced application every 500ms.
pub fn hardware_data_stream() -> iced::Subscription<HardwareData> {
    let stream = stream::channel(100000, |mut sender| async move {
        std::thread::spawn(move || {
            // Extract embedded binaries to a temporary directory.
            let temp_dir = extract_resources();
            let exe_path = temp_dir.join("TempMonitor.exe");
            // Spawn the C# executable with piped stdout for reading output.
            let mut cmd = std::process::Command::new(&exe_path)
                .stdout(std::process::Stdio::piped())
                .spawn()
                .expect("Failed to spawn TempMonitor.exe. Ensure the C# project is built.");
            let stdout = cmd.stdout.take().unwrap();
            // ProcessGuard ensures processes are killed when this scope ends.
            let _guard = ProcessGuard;
            let mut reader = std::io::BufReader::new(stdout);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break, // EOF reached
                    Ok(_) => {
                        let line = line.trim();
                        // Parse JSON line into HardwareData and send it.
                        if let Ok(data) = serde_json::from_str::<HardwareData>(line) {
                            let _ = sender.try_send(data);
                            // Throttle updates to every 500ms.
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                    }
                    Err(_) => break, // Error reading line
                }
            }
        });
        // Keep the async task alive indefinitely.
        future::pending::<()>().await
    });
    iced::Subscription::run_with_id("hardware", stream)
}

/// Extracts embedded binaries to a temporary directory.
/// This function writes the LibreHardwareMonitor DLL, Newtonsoft.Json DLL, and TempMonitor.exe
/// to the system's temp directory so they can be executed.
/// Returns the path to the temporary directory.
fn extract_resources() -> PathBuf {
    let temp_dir = std::env::temp_dir().join("libre_hardware_temp");
    std::fs::create_dir_all(&temp_dir).unwrap();

    std::fs::write(
        temp_dir.join("LibreHardwareMonitorLib.dll"),
        LIBRE_HARDWARE_MONITOR_LIB,
    )
    .unwrap();
    std::fs::write(temp_dir.join("Newtonsoft.Json.dll"), NEWTONSOFT_JSON).unwrap();
    std::fs::write(temp_dir.join("TempMonitor.exe"), TEMP_MONITOR_EXE).unwrap();

    temp_dir
}

/// GPU data collected using the machine_info crate.
/// This struct holds information about the primary GPU, including model, memory, temperature,
/// and utilization metrics.
#[derive(Debug, Clone)]
pub struct GpuData {
    /// GPU model name (e.g., "NVIDIA GeForce RTX 3080").
    pub model: String,
    /// Total VRAM in megabytes.
    pub vram_mb: u64,
    /// GPU temperature in Celsius (if available).
    pub temp: Option<f32>,
    /// GPU utilization as a percentage (0-100, if available).
    pub utilization: Option<f32>,
    /// VRAM usage as a percentage (0-100, if available).
    pub memory_usage: Option<f32>,
    /// Video encoder utilization as a percentage (0-100, if available).
    pub encoder: Option<f32>,
    /// Video decoder utilization as a percentage (0-100, if available).
    pub decoder: Option<f32>,
}

/// Retrieves current GPU data using the machine_info crate.
/// This function queries the system's graphics information and returns data for the primary GPU.
/// If no GPU is detected, it returns a placeholder GpuData with "No GPU detected".
pub fn get_gpu_data() -> GpuData {
    let mut machine = Machine::new();
    let sys_info = machine.system_info();
    let graphics_cards = &sys_info.graphics;

    if graphics_cards.is_empty() {
        return GpuData {
            model: "No GPU detected".to_string(),
            vram_mb: 0,
            temp: None,
            utilization: None,
            memory_usage: None,
            encoder: None,
            decoder: None,
        };
    }

    let card = &graphics_cards[0];
    let graphics_status = machine.graphics_status();
    let usage = graphics_status.first();

    GpuData {
        model: card.name.clone(),
        vram_mb: card.memory / 1_000_000, // Convert bytes to MB
        temp: usage
            .map(|u| u.temperature as f32)
            .or(Some(card.temperature as f32)), // Fallback to static temp if dynamic unavailable
        utilization: usage.map(|u| u.gpu as f32),
        memory_usage: usage.map(|u| u.memory_usage as f32),
        encoder: usage.map(|u| u.encoder as f32),
        decoder: usage.map(|u| u.decoder as f32),
    }
}

/// Creates an iced subscription that streams GPU data periodically.
/// This function polls GPU information every second using get_gpu_data() and sends
/// the data to the iced application for real-time updates.
pub fn gpu_data_stream() -> iced::Subscription<GpuData> {
    let stream = stream::channel(100000, |mut sender| async move {
        loop {
            let data = get_gpu_data();
            let _ = sender.try_send(data);
            // Update every 1 second to balance responsiveness and performance.
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
    iced::Subscription::run_with_id("gpu", stream)
}

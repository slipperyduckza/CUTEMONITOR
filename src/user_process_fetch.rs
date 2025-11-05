use serde::Deserialize;

// This module handles fetching and monitoring user processes using PowerShell
// It uses Get-Counter for real-time CPU metrics, providing accurate and efficient monitoring
// System processes are filtered out to show only user applications

use std::fs::File;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use lazy_static::lazy_static;

#[derive(Clone, Deserialize)]
pub struct Process {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "CPU")]
    pub cpu_usage: f64,
}

lazy_static! {
    static ref CURRENT_TOP_PROCESSES: Mutex<Vec<(String, String, f64)>> = Mutex::new(Vec::new());
    static ref IS_LOADING: Mutex<bool> = Mutex::new(true);
}

pub fn start_collection() {
    // Start background thread for continuous updates (no blocking initial query)
    thread::spawn(move || {
        loop {
            match fetch_processes() {
                Ok(processes) => {
                    let top4: Vec<(String, String, f64)> = processes.into_iter().take(4).map(|p| {
                        let desc_str = p.description.unwrap_or_else(|| "Unknown".to_string());
                        (p.name, desc_str, p.cpu_usage)
                    }).collect();
                    
                    *CURRENT_TOP_PROCESSES.lock().unwrap() = top4;
                    *IS_LOADING.lock().unwrap() = false;
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

pub fn get_top_processes() -> Vec<(String, String, f64)> {
    let is_loading = *IS_LOADING.lock().unwrap();
    let processes = CURRENT_TOP_PROCESSES.lock().unwrap().clone();
    
    if is_loading && processes.is_empty() {
        vec![
            ("Loading...".to_string(), "Initializing process monitor".to_string(), 0.0),
            ("".to_string(), "".to_string(), 0.0),
            ("".to_string(), "".to_string(), 0.0),
            ("".to_string(), "".to_string(), 0.0),
        ]
    } else {
        processes
    }
}

// Fetches the top user processes using PowerShell
// Uses Get-Counter for real-time CPU metrics, providing accurate and efficient monitoring
// Filters out system processes and returns the top 4 by CPU usage
fn fetch_processes() -> std::result::Result<Vec<Process>, String> {
    let command = r#"$ProgressPreference = 'SilentlyContinue'; Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty CounterSamples | Where-Object { $_.InstanceName -notlike '_total' -and $_.InstanceName -notlike 'idle' -and $_.InstanceName -notlike 'system' -and $_.InstanceName -notlike '*cutemonitor*' -and $_.InstanceName -notlike '*TempMonitor*' -and $_.InstanceName -notlike '*powershell*' } | ForEach-Object { $procName = ($_.InstanceName -split '#')[0]; $desc = (Get-Process -Name $procName -ErrorAction SilentlyContinue | Select-Object -First 1).Description; [PSCustomObject]@{ Name = $procName; Description = $desc; CPU = [math]::Round($_.CookedValue / [Environment]::ProcessorCount, 2) } } | Sort-Object CPU -Descending | Select-Object -First 4 | ConvertTo-Json"#;
    
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| e.to_string())?;

    if !output.stderr.is_empty() {
        let _ = File::create("debug_err.txt").and_then(|mut f| f.write_all(&output.stderr));
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    
    let processes: Vec<Process> = ::serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap_or_default();
    Ok(processes)
}
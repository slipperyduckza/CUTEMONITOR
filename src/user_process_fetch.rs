use serde::Deserialize;

// This module handles fetching and monitoring user processes using PowerShell
// It calculates CPU usage by polling twice and computing the difference
// System processes are filtered out to show only user applications

use std::collections::HashMap;
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
    #[serde(rename = "CPU")]
    pub cpu_usage: f64,
}

lazy_static! {
    static ref TOP_PROCESSES: Mutex<Vec<(String, String, f64)>> = Mutex::new(Vec::new());
}

pub fn start_collection() {
    // Initial once-off query
    if let Ok(processes) = fetch_processes() {
        let mut current_map: HashMap<String, (f64, Option<String>)> = HashMap::new();
        for p in &processes {
            let entry = current_map.entry(p.name.clone()).or_insert((0.0, None));
            entry.0 += p.cpu_usage;
        }
        let mut current_vec: Vec<(String, (f64, Option<String>))> = current_map.into_iter().collect();
        current_vec.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap_or(std::cmp::Ordering::Equal));
        let initial_top3: Vec<(String, String, f64)> = current_vec.into_iter().take(4).map(|(name, (cpu, desc))| {
            let desc_str = desc.unwrap_or_else(|| "Unknown".to_string());
            (name, desc_str, cpu)
        }).collect();
        *TOP_PROCESSES.lock().unwrap() = initial_top3;
    }

    // Background thread for averaged data
    thread::spawn(move || {
        let mut history: Vec<Vec<Process>> = Vec::new();
        loop {
            match fetch_processes() {
                Ok(processes) => {
                    history.push(processes);
                    if history.len() == 2 {
                        let mut avg_map: HashMap<String, (f64, Option<String>)> = HashMap::new();
                        for poll in &history {
                            for p in poll {
                                let entry = avg_map.entry(p.name.clone()).or_insert((0.0, None));
                                entry.0 += p.cpu_usage;
                            }
                        }
                        for (sum, _) in avg_map.values_mut() {
                            *sum /= 2.0;
                        }
                        let mut avg_vec: Vec<(String, (f64, Option<String>))> = avg_map.into_iter().collect();
                        avg_vec.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap_or(std::cmp::Ordering::Equal));
                        let top3: Vec<(String, String, f64)> = avg_vec.into_iter().take(4).map(|(name, (avg, desc))| {
                            let desc_str = desc.unwrap_or_else(|| "Unknown".to_string());
                            (name, desc_str, avg)
                        }).collect();
                        *TOP_PROCESSES.lock().unwrap() = top3;
                        history.clear();
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            thread::sleep(Duration::from_millis(2000));
        }
    });
}

pub fn get_top_processes() -> Vec<(String, String, f64)> {
    TOP_PROCESSES.lock().unwrap().clone()
}

// Fetches the top user processes using PowerShell
// The script polls CPU usage twice with a 1-second delay to calculate accurate usage percentages
// It filters out system processes and returns the top 4 by CPU usage
fn fetch_processes() -> Result<Vec<Process>, String> {
    let command = r#"$ProgressPreference = 'SilentlyContinue'; $currentUser = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $processes1 = Get-Process -IncludeUserName | Where-Object { $_.UserName -eq $currentUser -and $_.Name -notlike '*svchost*' -and $_.Name -notlike '*services*' -and $_.Name -notlike '*winlogon*' -and $_.Name -notlike '*lsass*' -and $_.Name -notlike '*csrss*' -and $_.Name -notlike '*sihost*' -and $_.Name -notlike '*StartMenuExperienceHost*' -and $_.Name -notlike '*ShellExperienceHost*' -and $_.Name -notlike '*explorer*' -and $_.Name -notlike '*taskhostw*' -and $_.Name -notlike '*runtimebroker*' -and $_.Name -notlike '*ShellHost*' -and $_.Name -notlike '*SearchHost*' -and $_.Name -notlike '*ApplicationFrameHost*' -and $_.Name -notlike '*LockApp*' -and $_.Name -notlike '*conhost*' -and $_.Name -notlike '*SystemSettings*' -and $_.Name -notlike '*smartscreen*' -and $_.Name -notlike '*SearchIndexer*' -and $_.Name -notlike '*PhoneExperienceHost*' -and $_.Name -notlike '*GameBarPresenceWriter*' -and $_.Name -notlike '*Widgets*' } | Group-Object Name | ForEach-Object { [PSCustomObject]@{ Name = $_.Name; CPU = ($_.Group | Measure-Object CPU -Sum).Sum } }; Start-Sleep -Milliseconds 1000; $processes2 = Get-Process | Group-Object Name | ForEach-Object { [PSCustomObject]@{ Name = $_.Name; CPU = ($_.Group | Measure-Object CPU -Sum).Sum } }; $usage = @(); foreach ($p1 in $processes1) { $p2 = $processes2 | Where-Object { $_.Name -eq $p1.Name } | Select-Object -First 1; if ($p2) { $diff = ($p2.CPU - $p1.CPU).TotalSeconds; $usage += [PSCustomObject]@{ Name = $p1.Name; CPU = $diff * 100 / $env:NUMBER_OF_PROCESSORS } } };         $usage | Where-Object { $_.Name -notlike '*cutemonitor*' -and $_.Name -notlike '*TempMonitor*' -and $_.Name -notlike '*powershell*' } | Sort-Object CPU -Descending | Select-Object -First 4 Name, CPU | ConvertTo-Json"#;
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
use regex::Regex;
use tokio::process::Command as TokioCommand;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

// Embedded temp monitor files
static TEMP_CS: &[u8] = include_bytes!("../TempMonitor.cs");
static TEMP_CSPROJ: &[u8] = include_bytes!("../TempMonitor.csproj");
static TEMP_DLL: &[u8] = include_bytes!("../LibreHardwareMonitorLib.dll");

pub fn parse_temp(s: &str) -> Option<f32> {
    let re = Regex::new(r"(\d+(?:[.,]\d+)?)°C").unwrap();
    let num_str = re.captures(s)?.get(1)?.as_str().replace(',', ".");
    num_str.parse().ok()
}

pub async fn get_temperatures() -> Vec<String> {
    if crate::what_cpu_check::is_virtual_machine() {
        return vec!["Virtual environment detected".to_string()];
    }

    // Create a temp directory
    let temp_dir = std::env::temp_dir().join("cutemonitor_temp");
    if tokio::fs::create_dir_all(&temp_dir).await.is_err() {
        return vec!["Error creating temp dir".to_string()];
    }

    // Write the files
    let cs_path = temp_dir.join("TempMonitor.cs");
    let csproj_path = temp_dir.join("TempMonitor.csproj");
    let dll_path = temp_dir.join("LibreHardwareMonitorLib.dll");

    if tokio::fs::write(&cs_path, TEMP_CS).await.is_err() {
        return vec!["Error writing cs".to_string()];
    }
    if tokio::fs::write(&csproj_path, TEMP_CSPROJ).await.is_err() {
        return vec!["Error writing csproj".to_string()];
    }
    if tokio::fs::write(&dll_path, TEMP_DLL).await.is_err() {
        return vec!["Error writing dll".to_string()];
    }

    // Run the command
    let output = TokioCommand::new("dotnet")
        .arg("run")
        .arg("--project")
        .arg(&csproj_path)
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
        .await;

    // Clean up temp files
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().map(|line| line.to_string()).collect()
        }
        _ => vec!["Error getting temperatures".to_string()],
    }
}
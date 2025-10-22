use tokio::process::Command;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

pub async fn get_top_processes() -> Vec<String> {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(r"Get-Process | Where-Object { $_.CPU -ne $null -and $_.Name -notmatch '^(System|Idle|csrss|wininit|services|lsass|winlogon|smss|svchost|explorer|dwm|taskhostw|sihost|fontdrvhost|ctfmon|SearchIndexer|SearchHost|RuntimeBroker|StartMenuExperienceHost|ShellExperienceHost|ApplicationFrameHost|TextInputHost|LockApp|WWAHost|MicrosoftEdge|msedge|msedgewebview2|conhost|eServiceHost|eOppFrame|LibreHardwareService|wlanext|ngclso|UserOOBEBroker|dllhost|SystemSettings|ShellHost|cutemonitor|tui-jfedab57)$' } | Sort-Object -Property CPU -Descending | Select-Object -First 3 -Property @{Name='DisplayName'; Expression={if ($_.Description) {$_.Description} else {$_.Name}}} | Format-Table -HideTableHeaders")
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
        .await;

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }
        Err(_) => vec![],
    }
}
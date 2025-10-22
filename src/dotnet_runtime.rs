// Embedded .NET installer
static DOTNET_INSTALLER: &[u8] = include_bytes!("../windowsdesktop-runtime-8.0.21-win-x64.exe");

pub fn check_and_install_dotnet() -> Result<(), Box<dyn std::error::Error>> {
    // Check for .NET 8 Desktop Runtime
    let output_result = std::process::Command::new("dotnet")
        .arg("--list-runtimes")
        .output();

    let is_installed = if let Ok(output) = output_result {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("Microsoft.WindowsDesktop.App 8.")
    } else {
        return Err("Failed to check .NET runtimes".into());
    };

    if !is_installed {
        // Load the embedded installer
        let installer_data = DOTNET_INSTALLER;
        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join("dotnet_installer.exe");
        std::fs::write(&installer_path, installer_data)?;
        // Run the installer
        let status = std::process::Command::new(&installer_path)
            .arg("/install")
            .arg("/passive")
            .arg("/norestart")
            .status()?;
        // Clean up
        let _ = std::fs::remove_file(&installer_path);
        if !status.success() {
            return Err("Failed to install .NET 8 Desktop Runtime".into());
        }
    }
    Ok(())
}
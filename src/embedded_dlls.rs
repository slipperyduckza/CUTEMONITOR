use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use anyhow::{Result, Context};
use log::{debug, info, warn};

// Embedded DLL files
pub static GPU_PERF_API_DX11_DLL: &[u8] = include_bytes!("../assets/GPUPerfAPIDX11-x64.dll");
pub static GPU_PERF_API_VK_DLL: &[u8] = include_bytes!("../assets/GPUPerfAPIVK-x64.dll");
pub static GPU_PERF_API_3_DX11_DLL: &[u8] = include_bytes!("../assets/3GPUPerfAPIDX11-x64.dll");
pub static GPU_PERF_API_3_VK_DLL: &[u8] = include_bytes!("../assets/3GPUPerfAPIVK-x64.dll");

pub struct EmbeddedDlls {
    temp_dir: PathBuf,
}

impl EmbeddedDlls {
    pub fn new() -> Result<Self> {
        let temp_dir = env::temp_dir().join("gpu_gui_monitor");
        
        // Create temp directory if it doesn't exist
        fs::create_dir_all(&temp_dir)
            .with_context(|| format!("Failed to create temp directory: {:?}", temp_dir))?;
        
        debug!("Created/verified temp directory: {:?}", temp_dir);
        
        Ok(EmbeddedDlls { temp_dir })
    }
    
    pub fn extract_dlls(&self) -> Result<()> {
        info!("Extracting embedded DLLs to temp directory...");
        
        let dlls = vec![
            ("GPUPerfAPIDX11-x64.dll", GPU_PERF_API_DX11_DLL),
            ("GPUPerfAPIVK-x64.dll", GPU_PERF_API_VK_DLL),
            ("3GPUPerfAPIDX11-x64.dll", GPU_PERF_API_3_DX11_DLL),
            ("3GPUPerfAPIVK-x64.dll", GPU_PERF_API_3_VK_DLL),
        ];
        
        for (dll_name, dll_data) in dlls {
            let dll_path = self.temp_dir.join(dll_name);
            
            // Check if DLL already exists and has the same size
            if dll_path.exists() {
                if let Ok(metadata) = fs::metadata(&dll_path) {
                    if metadata.len() as usize == dll_data.len() {
                        debug!("DLL {} already exists with correct size, skipping", dll_name);
                        continue;
                    }
                }
            }
            
            // Write the DLL to temp directory
            fs::write(&dll_path, dll_data)
                .with_context(|| format!("Failed to write DLL: {:?}", dll_path))?;
            
            info!("Extracted {} to {:?}", dll_name, dll_path);
        }
        
        // Add temp directory to PATH so the DLLs can be found
        let current_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{};{}", self.temp_dir.display(), current_path);
        env::set_var("PATH", new_path);
        
        debug!("Added temp directory to PATH: {:?}", self.temp_dir);
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn get_temp_dir(&self) -> &Path {
        &self.temp_dir
    }
    
    pub fn cleanup(&self) {
        debug!("Cleaning up temporary DLL files...");
        
        let dlls = vec![
            "GPUPerfAPIDX11-x64.dll",
            "GPUPerfAPIVK-x64.dll", 
            "3GPUPerfAPIDX11-x64.dll",
            "3GPUPerfAPIVK-x64.dll",
        ];
        
        for dll_name in dlls {
            let dll_path = self.temp_dir.join(dll_name);
            if let Err(e) = fs::remove_file(&dll_path) {
                warn!("Failed to remove DLL {:?}: {}", dll_path, e);
            } else {
                debug!("Removed DLL: {:?}", dll_path);
            }
        }
        
        // Try to remove the temp directory if it's empty
        if let Err(e) = fs::remove_dir(&self.temp_dir) {
            debug!("Could not remove temp directory {:?}: {}", self.temp_dir, e);
        } else {
            debug!("Removed temp directory: {:?}", self.temp_dir);
        }
    }
}

impl Drop for EmbeddedDlls {
    fn drop(&mut self) {
        self.cleanup();
    }
}
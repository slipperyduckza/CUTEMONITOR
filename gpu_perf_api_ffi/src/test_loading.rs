#[cfg(test)]
mod tests {
    use super::*;
    use libloading::{Library, Symbol};
    use std::env;
    
    #[test]
    fn test_asset_library_loading() {
        println!("Current working directory: {:?}", env::current_dir());
        
        let test_paths = vec![
            "assets/3GPUPerfAPIDX11-x64.dll",
            "assets\\3GPUPerfAPIDX11-x64.dll",
            "./assets/3GPUPerfAPIDX11-x64.dll",
            ".\\assets\\3GPUPerfAPIDX11-x64.dll",
        ];
        
        for path in test_paths {
            println!("Trying to load: {}", path);
            match unsafe { Library::new(path) } {
                Ok(_lib) => {
                    println!("Successfully loaded: {}", path);
                    return;
                }
                Err(e) => {
                    println!("Failed to load {}: {}", path, e);
                }
            }
        }
    }
}
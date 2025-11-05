use crate::gpu_interrogate::GpuInterrogator;
use crate::gpu_data_virtual::VirtualGpuDetector;
use crate::amd_version_detector::{AmdVersionDetector, GpuPerfApiVersion};
use crate::gpu_data::GpuInfo;
use anyhow::Result;
use std::time::Instant;
use log::warn;

#[derive(Debug, Clone)]
pub struct GpuDetectionResult {
    pub gpu_list: Vec<GpuInfo>,
    pub has_nvidia: bool,
    pub has_amd_discrete: bool,
    #[allow(dead_code)]
    pub has_virtual: bool,
    
    #[allow(dead_code)]
    pub amd_gpu_versions: Vec<(usize, GpuPerfApiVersion)>, // GPU index -> GPUPerfAPI version mapping
}

pub struct LaunchGpuDetector {
    interrogator: GpuInterrogator,
    vm_detector: VirtualGpuDetector,
    amd_version_detector: AmdVersionDetector,
}

impl LaunchGpuDetector {
    pub fn new() -> Result<Self> {
        let interrogator = GpuInterrogator::new()?;
        let vm_detector = VirtualGpuDetector::new()?;
        let amd_version_detector = AmdVersionDetector::new();

        Ok(LaunchGpuDetector {
            interrogator,
            vm_detector,
            amd_version_detector,
        })
    }

    /// Perform one-time GPU detection with virtual environment support
    pub async fn detect_gpus(&mut self) -> Result<GpuDetectionResult> {
        println!("Detecting GPUs...");
        let detection_start = Instant::now();

        // Check if running in virtual environment first
        let is_virtual = self.vm_detector.is_virtual_environment();
        if is_virtual {
            println!("✓ Running in virtual environment detected");
            let detection_info = self.vm_detector.get_detection_info();
            if detection_info.contains(vm_detect::Detection::HYPERVISOR_BIT) {
                println!("  - Hypervisor bit detected");
            }
            if detection_info.contains(vm_detect::Detection::HYPERVISOR_CPU_VENDOR) {
                println!("  - Hypervisor CPU vendor detected");
            }
            if detection_info.contains(vm_detect::Detection::UNEXPECTED_CPU_VENDOR) {
                println!("  - Unexpected CPU vendor detected");
            }
        }

        let gpu_list = match self.interrogator.get_gpu_list().await {
            Ok(mut gpus) => {
                // Enrich GPU data with virtual GPU information if in VM
                if is_virtual {
                    println!("✓ Enriching GPU data with virtual GPU information...");
                    for gpu in &mut gpus {
                        if let Err(e) = self.vm_detector.enrich_vm_gpu(gpu) {
                            warn!("Failed to enrich GPU {}: {}", gpu.name, e);
                        }
                    }

                    // Try to detect additional virtual GPUs that might not be in the main list
                    if let Ok(virtual_gpus) = self.vm_detector.detect_virtual_gpus() {
                        for virtual_gpu in virtual_gpus {
                            // Check if this virtual GPU is already in the list
                            let already_exists = gpus.iter().any(|gpu| {
                                gpu.pnp_device_id == virtual_gpu.pnp_device_id
                                    || gpu.name == virtual_gpu.name
                            });

                            if !already_exists {
                                println!("  + Found additional virtual GPU: {}", virtual_gpu.name);
                                gpus.push(virtual_gpu);
                            }
                        }
                    }
                }

                let detection_time = detection_start.elapsed();
                println!(
                    "✓ GPU Detection completed in {:.2}ms",
                    detection_time.as_millis()
                );
                gpus
            }
            Err(e) => {
                eprintln!("✗ Error during GPU detection: {}", e);
                return Err(e);
            }
        };

        if gpu_list.is_empty() {
            return Err(anyhow::anyhow!("No GPUs detected"));
        }

        // Analyze detected GPUs to determine which monitors are needed
        let mut has_nvidia = false;
        let mut has_amd_discrete = false;
        let has_virtual = is_virtual;
        let mut amd_gpu_versions = Vec::new();

        for gpu in gpu_list.iter() {
            let name_lower = gpu.name.to_lowercase();

            if name_lower.contains("nvidia") || name_lower.contains("geforce") {
                has_nvidia = true;
            }

            if name_lower.contains("amd")
                || name_lower.contains("radeon")
                || name_lower.contains("firepro")
            {
                // Treat all AMD GPUs as discrete (no integrated GPU support)
                has_amd_discrete = true;

                // Detect GPUPerfAPI version for this AMD GPU
                let version = self.amd_version_detector.detect_version_for_gpu(&gpu.name);
                amd_gpu_versions.push((gpu_list.iter().position(|g| std::ptr::eq(g, gpu)).unwrap(), version));

                println!(
                    "  ✓ AMD GPU {} detected - will use {}",
                    gpu.name,
                    AmdVersionDetector::get_version_name(version)
                );
            }
        }

        // Sort GPUs by expected update speed (NVIDIA first, AMD last)
        let mut gpu_indices: Vec<usize> = (0..gpu_list.len()).collect();
        gpu_indices.sort_by(|&a, &b| {
            let a_gpu = &gpu_list[a];
            let b_gpu = &gpu_list[b];

            // NVIDIA GPUs (fast) first, AMD GPUs (slower) last
            let a_is_nvidia = a_gpu.name.to_lowercase().contains("nvidia")
                || a_gpu.name.to_lowercase().contains("geforce");
            let b_is_nvidia = b_gpu.name.to_lowercase().contains("nvidia")
                || b_gpu.name.to_lowercase().contains("geforce");

            match (a_is_nvidia, b_is_nvidia) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal, // Keep original order for same vendor
            }
        });

        println!("GPU Update Order (optimized for speed):");
        for &index in &gpu_indices {
            let gpu = &gpu_list[index];
            let vendor = if gpu.name.to_lowercase().contains("nvidia")
                || gpu.name.to_lowercase().contains("geforce")
            {
                "NVIDIA (Fast)"
            } else if gpu.name.to_lowercase().contains("amd")
                || gpu.name.to_lowercase().contains("radeon")
            {
                "AMD (Slower)"
            } else {
                "Unknown"
            };
            println!("  {}. {} - {}", index + 1, gpu.name, vendor);
        }

        // Print detection summary
        println!("\nDetection Summary:");
        if has_nvidia {
        }

        Ok(GpuDetectionResult {
            gpu_list,
            has_nvidia,
            has_amd_discrete,
            has_virtual,
            
            amd_gpu_versions,
        })
    }

    /// Get reference to the interrogator for updating GPU metrics
    

    /// Consume the detector and return the interrogator
    #[allow(dead_code)]
    pub fn into_interrogator(self) -> GpuInterrogator {
        self.interrogator
    }
}
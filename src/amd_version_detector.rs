//! AMD GPUPerfAPI version detection based on GPU model names
//!
//! This module determines which GPUPerfAPI version to use for AMD GPUs
//! based on the GPU model name and supported card lists.

use std::collections::HashMap;

/// GPUPerfAPI version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPerfApiVersion {
    V3_17,
    V4_1,
}

/// AMD GPUPerfAPI version detector
pub struct AmdVersionDetector {
    // Cache for model name to version mapping
    version_cache: HashMap<String, GpuPerfApiVersion>,
}

impl AmdVersionDetector {
    pub fn new() -> Self {
        Self {
            version_cache: HashMap::new(),
        }
    }

    /// Determine GPUPerfAPI version for AMD GPU based on model name
    pub fn detect_version_for_gpu(&mut self, gpu_name: &str) -> GpuPerfApiVersion {
        // Check cache first
        if let Some(&version) = self.version_cache.get(gpu_name) {
            return version;
        }

        let version = self.determine_version_from_name(gpu_name);

        // Cache the result
        self.version_cache.insert(gpu_name.to_string(), version);

        version
    }

    /// Determine version based on GPU model name patterns
    fn determine_version_from_name(&self, gpu_name: &str) -> GpuPerfApiVersion {
        let name_lower = gpu_name.to_lowercase();

        // GPUPerfAPI 4.1 supported cards (newer AMD GPUs)
        if self.is_gpa_41_supported(&name_lower) {
            GpuPerfApiVersion::V4_1
        }
        // GPUPerfAPI 3.17 supported cards (older AMD GPUs)
        else if self.is_gpa_317_supported(&name_lower) {
            GpuPerfApiVersion::V3_17
        }
        // Default fallback - use 3.17 for unclear/unknown AMD models
        else {
            GpuPerfApiVersion::V3_17
        }
    }

    /// Check if GPU is supported by GPUPerfAPI 4.1
    fn is_gpa_41_supported(&self, name_lower: &str) -> bool {
        // RX 9000 Series
        if name_lower.contains("rx 90") {
            return true;
        }

        // RX 7000 Series
        if name_lower.contains("rx 7") {
            return true;
        }

        // RX 6000 Series
        if name_lower.contains("rx 6") {
            return true;
        }

        // RX 5000 Series
        if name_lower.contains("rx 5")
            && (name_lower.contains("5300")
                || name_lower.contains("5400")
                || name_lower.contains("5500")
                || name_lower.contains("5600")
                || name_lower.contains("5700"))
        {
            return true;
        }

        // Radeon AI PRO
        if name_lower.contains("radeon") && name_lower.contains("ai") {
            return true;
        }

        false
    }

    /// Check if GPU is supported by GPUPerfAPI 3.17
    fn is_gpa_317_supported(&self, name_lower: &str) -> bool {
        // RX Vega Series
        if name_lower.contains("vega") {
            return true;
        }

        // RX 500 Series (excluding RX 5000 series which are handled above)
        if name_lower.contains("rx 5")
            && (name_lower.contains("rx 5")
                && !name_lower.contains("5300")
                && !name_lower.contains("5400")
                && !name_lower.contains("5500")
                && !name_lower.contains("5600")
                && !name_lower.contains("5700"))
        {
            return true;
        }

        // RX 400 Series
        if name_lower.contains("rx 4") {
            return true;
        }

        // R9 Fury series
        if name_lower.contains("fury") {
            return true;
        }

        // R9 Nano
        if name_lower.contains("nano") {
            return true;
        }

        // R9 Pro Duo
        if name_lower.contains("pro duo") {
            return true;
        }

        // Radeon Pro WX Series
        if name_lower.contains("wx") && name_lower.contains("radeon") {
            return true;
        }

        // R7/R5 300 Series
        if (name_lower.contains("r7") || name_lower.contains("r5")) && name_lower.contains("3") {
            return true;
        }

        // R7/R5 200 Series
        if (name_lower.contains("r7") || name_lower.contains("r5")) && name_lower.contains("2") {
            return true;
        }

        // Generic AMD Radeon Graphics (commonly found in laptops/APUs)
        if name_lower.contains("amd") && name_lower.contains("radeon") && name_lower.contains("graphics") {
            return true;
        }

        false
    }

    /// Get version name for display
    pub fn get_version_name(version: GpuPerfApiVersion) -> &'static str {
        match version {
            GpuPerfApiVersion::V4_1 => "GPUPerfAPI 4.1",
            GpuPerfApiVersion::V3_17 => "GPUPerfAPI 3.17",
        }
    }
}

impl Default for AmdVersionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_detection() {
        let mut detector = AmdVersionDetector::new();

        // Test RX 7000 series (should use 4.1)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 7900 XTX"),
            GpuPerfApiVersion::V4_1
        );
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 7600 XT"),
            GpuPerfApiVersion::V4_1
        );

        // Test RX 6000 series (should use 4.1)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 6950 XT"),
            GpuPerfApiVersion::V4_1
        );
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 6600"),
            GpuPerfApiVersion::V4_1
        );

        // Test RX 5000 series (should use 4.1)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 5700 XT"),
            GpuPerfApiVersion::V4_1
        );

        // Test Vega series (should use 3.17)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX Vega 64"),
            GpuPerfApiVersion::V3_17
        );
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon Vega Frontier Edition"),
            GpuPerfApiVersion::V3_17
        );

        // Test RX 500 series (should use 3.17)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 580"),
            GpuPerfApiVersion::V3_17
        );
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 590"),
            GpuPerfApiVersion::V3_17
        );

        // Test RX 400 series (should use 3.17)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 480"),
            GpuPerfApiVersion::V3_17
        );
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon RX 470"),
            GpuPerfApiVersion::V3_17
        );

        // Test R9 Fury (should use 3.17)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon R9 Fury X"),
            GpuPerfApiVersion::V3_17
        );

        // Test Radeon Pro WX (should use 3.17)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon Pro WX 9100"),
            GpuPerfApiVersion::V3_17
        );

        // Test fallback for unclear AMD models (should use 3.17)
        assert_eq!(
            detector.detect_version_for_gpu("AMD Radeon(TM) Graphics"),
            GpuPerfApiVersion::V3_17
        );
        assert_eq!(
            detector.detect_version_for_gpu("AMD Graphics"),
            GpuPerfApiVersion::V3_17
        );
    }
}

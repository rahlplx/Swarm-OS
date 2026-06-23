use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareProfile {
    pub cpu_cores: usize,
    pub cpu_name: String,
    pub ram_total_bytes: u64,
    pub ram_available_bytes: u64,
    pub gpus: Vec<GpuInfo>,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuInfo {
    pub name: String,
    pub vram_bytes: u64,
    pub backend: GpuBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpuBackend {
    Cuda,
    Rocm,
    Metal,
    None,
}

pub trait GpuDetector: Send + Sync {
    fn detect_gpus(&self) -> Vec<GpuInfo>;
}

pub struct SystemProfiler<G: GpuDetector> {
    gpu_detector: G,
}

impl<G: GpuDetector> SystemProfiler<G> {
    pub fn new(gpu_detector: G) -> Self {
        Self { gpu_detector }
    }

    pub fn detect(&self) -> HardwareProfile {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_name = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        HardwareProfile {
            cpu_cores: sys.cpus().len(),
            cpu_name,
            ram_total_bytes: sys.total_memory(),
            ram_available_bytes: sys.available_memory(),
            gpus: self.gpu_detector.detect_gpus(),
            os: System::long_os_version().unwrap_or_else(|| "Unknown".to_string()),
        }
    }
}

pub struct NoGpuDetector;

impl GpuDetector for NoGpuDetector {
    fn detect_gpus(&self) -> Vec<GpuInfo> {
        vec![]
    }
}

pub fn detect_hardware_default() -> HardwareProfile {
    SystemProfiler::new(NoGpuDetector).detect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockGpuDetector {
        gpus: Vec<GpuInfo>,
    }

    impl GpuDetector for MockGpuDetector {
        fn detect_gpus(&self) -> Vec<GpuInfo> {
            self.gpus.clone()
        }
    }

    #[test]
    fn detect_hardware_returns_valid_profile() {
        let profiler = SystemProfiler::new(NoGpuDetector);
        let profile = profiler.detect();

        assert!(profile.cpu_cores > 0);
        assert!(profile.ram_total_bytes > 0);
        assert!(!profile.cpu_name.is_empty());
        assert!(profile.gpus.is_empty());
    }

    #[test]
    fn detect_hardware_with_mock_gpu() {
        let mock = MockGpuDetector {
            gpus: vec![GpuInfo {
                name: "RTX 4090".to_string(),
                vram_bytes: 24 * 1024 * 1024 * 1024,
                backend: GpuBackend::Cuda,
            }],
        };

        let profiler = SystemProfiler::new(mock);
        let profile = profiler.detect();

        assert_eq!(profile.gpus.len(), 1);
        assert_eq!(profile.gpus[0].name, "RTX 4090");
        assert_eq!(profile.gpus[0].vram_bytes, 24 * 1024 * 1024 * 1024);
    }

    #[test]
    fn hardware_profile_serializes() {
        let profile = HardwareProfile {
            cpu_cores: 8,
            cpu_name: "Test CPU".to_string(),
            ram_total_bytes: 16 * 1024 * 1024 * 1024,
            ram_available_bytes: 8 * 1024 * 1024 * 1024,
            gpus: vec![],
            os: "Linux".to_string(),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: HardwareProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, deserialized);
    }
}

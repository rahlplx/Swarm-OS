use serde::{Deserialize, Serialize};

use super::profiler::{GpuBackend, HardwareProfile};

/// Weighted capability score: vram×4 + ram×0.5 + cpu×0.25 + backend_bonus
/// See architecture.md §3 for the canonical algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScore {
    pub total: f64,
    pub vram_score: f64,
    pub ram_score: f64,
    pub cpu_score: f64,
    pub backend_bonus: f64,
}

const VRAM_WEIGHT: f64 = 4.0;
const RAM_WEIGHT: f64 = 0.5;
const CPU_WEIGHT: f64 = 0.25;
const CUDA_BONUS: f64 = 10.0;
const ROCM_BONUS: f64 = 8.0;
const METAL_BONUS: f64 = 9.0;

pub fn compute_capability(profile: &HardwareProfile) -> CapabilityScore {
    let vram_gib: f64 = profile
        .gpus
        .iter()
        .map(|g| g.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        .sum();
    let ram_gib = profile.ram_total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let cpu_count = profile.cpu_cores as f64;

    let backend_bonus = profile
        .gpus
        .iter()
        .map(|g| match g.backend {
            GpuBackend::Cuda => CUDA_BONUS,
            GpuBackend::Rocm => ROCM_BONUS,
            GpuBackend::Metal => METAL_BONUS,
            GpuBackend::None => 0.0,
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    let vram_score = vram_gib * VRAM_WEIGHT;
    let ram_score = ram_gib * RAM_WEIGHT;
    let cpu_score = cpu_count * CPU_WEIGHT;
    let total = vram_score + ram_score + cpu_score + backend_bonus;

    CapabilityScore {
        total,
        vram_score,
        ram_score,
        cpu_score,
        backend_bonus,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::profiler::{GpuBackend, GpuInfo, HardwareProfile};

    fn test_profile(gpus: Vec<GpuInfo>, ram_gib: u64, cpu_cores: usize) -> HardwareProfile {
        HardwareProfile {
            cpu_cores,
            cpu_name: "Test".to_string(),
            ram_total_bytes: ram_gib * 1024 * 1024 * 1024,
            ram_available_bytes: ram_gib * 1024 * 1024 * 1024 / 2,
            gpus,
            os: "Linux".to_string(),
        }
    }

    #[test]
    fn cpu_only_node_scores_correctly() {
        let profile = test_profile(vec![], 16, 8);
        let score = compute_capability(&profile);

        assert_eq!(score.vram_score, 0.0);
        assert_eq!(score.ram_score, 16.0 * 0.5);
        assert_eq!(score.cpu_score, 8.0 * 0.25);
        assert_eq!(score.backend_bonus, 0.0);
        assert_eq!(score.total, 8.0 + 2.0);
    }

    #[test]
    fn rtx_4090_node_scores_correctly() {
        let profile = test_profile(
            vec![GpuInfo {
                name: "RTX 4090".to_string(),
                vram_bytes: 24 * 1024 * 1024 * 1024,
                backend: GpuBackend::Cuda,
            }],
            32,
            16,
        );
        let score = compute_capability(&profile);

        assert_eq!(score.vram_score, 24.0 * 4.0); // 96.0
        assert_eq!(score.ram_score, 32.0 * 0.5); // 16.0
        assert_eq!(score.cpu_score, 16.0 * 0.25); // 4.0
        assert_eq!(score.backend_bonus, 10.0); // CUDA
        assert_eq!(score.total, 96.0 + 16.0 + 4.0 + 10.0);
    }

    #[test]
    fn metal_node_gets_metal_bonus() {
        let profile = test_profile(
            vec![GpuInfo {
                name: "M3 Max".to_string(),
                vram_bytes: 36 * 1024 * 1024 * 1024,
                backend: GpuBackend::Metal,
            }],
            36,
            12,
        );
        let score = compute_capability(&profile);

        assert_eq!(score.backend_bonus, 9.0);
    }

    #[test]
    fn multi_gpu_sums_vram() {
        let profile = test_profile(
            vec![
                GpuInfo {
                    name: "RTX 4090".to_string(),
                    vram_bytes: 24 * 1024 * 1024 * 1024,
                    backend: GpuBackend::Cuda,
                },
                GpuInfo {
                    name: "RTX 4090".to_string(),
                    vram_bytes: 24 * 1024 * 1024 * 1024,
                    backend: GpuBackend::Cuda,
                },
            ],
            64,
            32,
        );
        let score = compute_capability(&profile);

        assert_eq!(score.vram_score, 48.0 * 4.0); // 192.0
    }
}

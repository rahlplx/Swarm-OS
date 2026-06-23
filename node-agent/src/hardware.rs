use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub cpu_cores: u32,
    pub ram_total_gib: f32,
    pub ram_free_gib: f32,
    pub gpu_name: Option<String>,
    pub vram_total_gib: Option<f32>,
    pub vram_free_gib: Option<f32>,
    pub backend: InferenceBackend,
    // No capability_score field — call capability_score(&caps) as the single source of truth
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum InferenceBackend {
    Cuda,
    Metal,
    Vulkan,
    CpuOnly,
}

/// Detect hardware capabilities of this node.
/// Phase 0: implements sysinfo for CPU/RAM; nvml-wrapper for NVIDIA VRAM.
pub fn detect_capabilities() -> NodeCapabilities {
    todo!("implement via sysinfo + nvml-wrapper crates")
}

/// Two-phase scheduler scoring formula — canonical values from architecture.md §3:
/// vram×4 + ram×0.5 + cpu×0.25 + backend_bonus (cuda=10/metal=8/vulkan=5/cpu=0)
pub fn capability_score(caps: &NodeCapabilities) -> f32 {
    let vram = caps.vram_total_gib.unwrap_or(0.0);
    let backend_bonus: f32 = match caps.backend {
        InferenceBackend::Cuda => 10.0,
        InferenceBackend::Metal => 8.0,
        InferenceBackend::Vulkan => 5.0,
        InferenceBackend::CpuOnly => 0.0,
    };
    (vram * 4.0) + (caps.ram_total_gib * 0.5) + (caps.cpu_cores as f32 * 0.25) + backend_bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_caps(
        cpu_cores: u32,
        ram_gib: f32,
        vram_gib: Option<f32>,
        backend: InferenceBackend,
    ) -> NodeCapabilities {
        NodeCapabilities {
            cpu_cores,
            ram_total_gib: ram_gib,
            ram_free_gib: ram_gib * 0.75,
            gpu_name: vram_gib.map(|_| "test-gpu".into()),
            vram_total_gib: vram_gib,
            vram_free_gib: vram_gib.map(|v| v * 0.85),
            backend,
        }
    }

    #[test]
    fn score_rtx4090_node() {
        // (24×4) + (32×0.5) + (8×0.25) + 10 = 96 + 16 + 2 + 10 = 124
        let caps = make_caps(8, 32.0, Some(24.0), InferenceBackend::Cuda);
        let score = capability_score(&caps);
        assert!((score - 124.0).abs() < 0.01, "score={score}, expected 124");
    }

    #[test]
    fn score_cpu_only_node() {
        // (0×4) + (8×0.5) + (4×0.25) + 0 = 0 + 4 + 1 + 0 = 5
        let caps = make_caps(4, 8.0, None, InferenceBackend::CpuOnly);
        let score = capability_score(&caps);
        assert!((score - 5.0).abs() < 0.01, "score={score}, expected 5");
    }

    #[test]
    fn score_apple_m3_max() {
        // M3 Max (unified 48 GiB, Metal backend), 8 perf cores
        // (48×4) + (48×0.5) + (8×0.25) + 8 = 192 + 24 + 2 + 8 = 226
        let caps = make_caps(8, 48.0, Some(48.0), InferenceBackend::Metal);
        let score = capability_score(&caps);
        assert!((score - 226.0).abs() < 0.01, "score={score}, expected 226");
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn detect_capabilities_is_not_implemented_yet() {
        detect_capabilities();
    }
}

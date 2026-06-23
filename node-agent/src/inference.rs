use std::path::Path;
use tokio::sync::mpsc;

/// Phase 0 single-device inference via llama-cpp-2 (utilityai crate).
/// llama-rs (rustformers) is ARCHIVED — do not use.
pub struct ModelHandle {
    pub model_path: std::path::PathBuf,
    pub context_size: u32,
}

pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stream: bool,
}

pub struct TokenDelta {
    pub token: String,
    pub finish_reason: Option<String>,
}

/// Load a GGUF model. Returns a handle for subsequent inference calls.
/// Phase 0 constraint: Metal backend enforces METAL_MAX_CONTEXT limit.
pub async fn load_model(_path: &Path, _context_size: u32) -> anyhow::Result<ModelHandle> {
    todo!("implement via llama-cpp-2 crate (utilityai/llama-cpp-2)")
}

/// Run streaming inference; each token is sent on `tx`.
pub async fn infer(
    _handle: &ModelHandle,
    _req: InferenceRequest,
    _tx: mpsc::Sender<TokenDelta>,
) -> anyhow::Result<()> {
    todo!("implement streaming generation loop via llama_decode + llama_get_logits_ith")
}

/// VRAM headroom required per model size + quantization (research.md benchmarks).
/// Returns safe headroom in GiB (warm load + activation buffer).
pub fn vram_headroom_gib(model_params: u64, quant: &str) -> f32 {
    // Headroom values from research.md benchmarks + guide.md appendix.
    // "Safe headroom" = warm VRAM usage + activation buffer.
    match (model_params, quant) {
        (p, "Q4_K_M") if (7_000_000_000..=7_999_999_999).contains(&p) => 5.20,
        (p, "Q5_K_M") if (7_000_000_000..=7_999_999_999).contains(&p) => 6.00,
        (p, "Q8_0") if (7_000_000_000..=7_999_999_999).contains(&p) => 9.50,
        // 13B Llama-2 Q4_K_M: warm 8.11 GiB, headroom 9.50 GiB (guide.md appendix)
        (p, "Q4_K_M") if (13_000_000_000..=13_999_999_999).contains(&p) => 9.50,
        (p, "Q4_K_M") if (70_000_000_000..=70_999_999_999).contains(&p) => 48.00,
        (p, "Q5_K_M") if (70_000_000_000..=70_999_999_999).contains(&p) => 60.00,
        _ => 0.0,
    }
}

/// Apple Metal unified memory context limit. Exceeding this causes a segfault
/// due to the OS enforcing a 75% recommendedMaxWorkingSetSize cap on KV cache allocation.
pub const METAL_MAX_CONTEXT: u32 = 8192;

/// llama.cpp HTTP error codes (from research.md §1.1 HTTP Server Error Codes).
#[derive(Debug, PartialEq)]
pub enum LlamaHttpError {
    BadRequest,      // 400 — invalid JSON payload
    ContextExceeded, // 422 — context length > configured window
    OomOrGpuFailure, // 500 — VRAM exhaustion or driver timeout
    QueueFull,       // 503 — request queue capacity exceeded
}

impl LlamaHttpError {
    pub fn from_status(code: u16) -> Option<Self> {
        match code {
            400 => Some(Self::BadRequest),
            422 => Some(Self::ContextExceeded),
            500 => Some(Self::OomOrGpuFailure),
            503 => Some(Self::QueueFull),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vram_headroom_7b_q4km() {
        assert_eq!(vram_headroom_gib(7_500_000_000, "Q4_K_M"), 5.20);
    }

    #[test]
    fn vram_headroom_70b_q4km() {
        assert_eq!(vram_headroom_gib(70_000_000_000, "Q4_K_M"), 48.00);
    }

    #[test]
    fn vram_headroom_13b_q4km() {
        // 13B Llama-2 Q4_K_M: guide.md appendix — warm 8.11 GiB, headroom 9.50 GiB
        assert_eq!(vram_headroom_gib(13_000_000_000, "Q4_K_M"), 9.50);
    }

    #[test]
    fn vram_headroom_unknown_returns_zero() {
        // 15B is not in the table — should return 0.0 to indicate "unknown, check manually"
        assert_eq!(vram_headroom_gib(15_000_000_000, "Q4_K_M"), 0.0);
    }

    #[test]
    fn metal_context_limit_is_enforced_at_8192() {
        assert_eq!(METAL_MAX_CONTEXT, 8192);
    }

    #[test]
    fn llama_http_error_mapping() {
        assert_eq!(
            LlamaHttpError::from_status(400),
            Some(LlamaHttpError::BadRequest)
        );
        assert_eq!(
            LlamaHttpError::from_status(422),
            Some(LlamaHttpError::ContextExceeded)
        );
        assert_eq!(
            LlamaHttpError::from_status(500),
            Some(LlamaHttpError::OomOrGpuFailure)
        );
        assert_eq!(
            LlamaHttpError::from_status(503),
            Some(LlamaHttpError::QueueFull)
        );
        assert_eq!(LlamaHttpError::from_status(200), None);
    }

    #[tokio::test]
    #[should_panic(expected = "not yet implemented")]
    async fn load_model_panics_until_implemented() {
        load_model(Path::new("test.gguf"), 4096).await.unwrap();
    }
}

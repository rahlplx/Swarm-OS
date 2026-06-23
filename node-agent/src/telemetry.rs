use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

/// Phase 0 telemetry: in-process counters. Phase 1+ exports to Prometheus.
/// Custom metrics per architecture.md §F7:
///   swarm_tokens_per_second, swarm_node_vram_used, swarm_job_queue_depth
static TOKENS_GENERATED: AtomicU64 = AtomicU64::new(0);
static INFERENCE_REQUESTS: AtomicU64 = AtomicU64::new(0);
static INFERENCE_ERRORS: AtomicU64 = AtomicU64::new(0);

pub fn record_inference(tokens_out: u64, succeeded: bool) {
    TOKENS_GENERATED.fetch_add(tokens_out, Ordering::Relaxed);
    INFERENCE_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if !succeeded {
        INFERENCE_ERRORS.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn snapshot() -> TelemetrySnapshot {
    TelemetrySnapshot {
        tokens_generated: TOKENS_GENERATED.load(Ordering::Relaxed),
        inference_requests: INFERENCE_REQUESTS.load(Ordering::Relaxed),
        inference_errors: INFERENCE_ERRORS.load(Ordering::Relaxed),
    }
}

#[derive(Debug, Serialize)]
pub struct TelemetrySnapshot {
    pub tokens_generated: u64,
    pub inference_requests: u64,
    pub inference_errors: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_snapshot_delta() {
        let before = snapshot();
        record_inference(128, true);
        record_inference(0, false);
        let after = snapshot();
        assert_eq!(after.tokens_generated - before.tokens_generated, 128);
        assert_eq!(after.inference_requests - before.inference_requests, 2);
        assert_eq!(after.inference_errors - before.inference_errors, 1);
    }
}

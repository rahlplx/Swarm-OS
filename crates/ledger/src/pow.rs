use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

pub struct ProofOfWork;

pub const MAX_DIFFICULTY: u32 = 16;

#[derive(Debug)]
pub enum PoWError {
    Timeout { attempts: u64, elapsed: Duration },
    Cancelled,
    DifficultyTooHigh { required: u32, max: u32 },
}

impl std::fmt::Display for PoWError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoWError::Timeout { attempts, elapsed } => {
                write!(
                    f,
                    "Mining timeout after {} attempts ({:?})",
                    attempts, elapsed
                )
            }
            PoWError::Cancelled => write!(f, "Mining cancelled"),
            PoWError::DifficultyTooHigh { required, max } => {
                write!(f, "Difficulty {} exceeds maximum allowed {}", required, max)
            }
        }
    }
}

impl std::error::Error for PoWError {}

impl ProofOfWork {
    pub fn mine(data: &[u8], difficulty: u32) -> Result<u64, PoWError> {
        Self::mine_with_timeout(data, difficulty, Duration::from_secs(60))
    }

    pub fn mine_with_timeout(
        data: &[u8],
        difficulty: u32,
        timeout: Duration,
    ) -> Result<u64, PoWError> {
        if difficulty > MAX_DIFFICULTY {
            return Err(PoWError::DifficultyTooHigh {
                required: difficulty,
                max: MAX_DIFFICULTY,
            });
        }
        let prefix = "0".repeat(difficulty as usize);
        let mut nonce: u64 = 0;
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(PoWError::Timeout {
                    attempts: nonce,
                    elapsed: start.elapsed(),
                });
            }

            let hash = Self::compute_hash(data, nonce);
            let hash_hex = hex::encode(hash);
            if hash_hex.starts_with(&prefix) {
                return Ok(nonce);
            }
            nonce += 1;

            if nonce.is_multiple_of(1_000_000) {
                tracing::debug!(
                    "Mining: tried {} nonces, elapsed {:?}",
                    nonce,
                    start.elapsed()
                );
            }
        }
    }

    pub fn mine_cancellable<F>(
        data: &[u8],
        difficulty: u32,
        should_cancel: F,
    ) -> Result<u64, PoWError>
    where
        F: Fn() -> bool,
    {
        if difficulty > MAX_DIFFICULTY {
            return Err(PoWError::DifficultyTooHigh {
                required: difficulty,
                max: MAX_DIFFICULTY,
            });
        }
        let prefix = "0".repeat(difficulty as usize);
        let mut nonce: u64 = 0;

        loop {
            if should_cancel() {
                return Err(PoWError::Cancelled);
            }

            let hash = Self::compute_hash(data, nonce);
            let hash_hex = hex::encode(hash);
            if hash_hex.starts_with(&prefix) {
                return Ok(nonce);
            }
            nonce += 1;

            if nonce.is_multiple_of(1_000_000) {
                tracing::debug!("Mining: tried {} nonces", nonce);
            }
        }
    }

    pub fn verify(data: &[u8], nonce: u64, difficulty: u32) -> bool {
        let hash = Self::compute_hash(data, nonce);
        let hash_hex = hex::encode(hash);
        let prefix = "0".repeat(difficulty as usize);
        hash_hex.starts_with(&prefix)
    }

    fn compute_hash(data: &[u8], nonce: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(nonce.to_le_bytes());
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_timeout() {
        let data = b"timeout test";
        let start = Instant::now();
        let result = ProofOfWork::mine_with_timeout(data, 8, Duration::from_millis(100));
        assert!(result.is_err());
        assert!(start.elapsed().as_millis() < 200);
    }

    #[test]
    fn test_pow_cancellation() {
        let data = b"cancel test";
        let cancelled = std::sync::atomic::AtomicBool::new(true);
        let result = ProofOfWork::mine_cancellable(data, 10, || {
            cancelled.load(std::sync::atomic::Ordering::Relaxed)
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_pow_success_within_timeout() {
        let data = b"easy mining";
        let result = ProofOfWork::mine_with_timeout(data, 2, Duration::from_secs(5));
        assert!(result.is_ok());
        assert!(ProofOfWork::verify(data, result.unwrap(), 2));
    }

    #[test]
    fn test_pow_difficulty_cap() {
        let data = b"difficulty cap test";
        let result = ProofOfWork::mine(data, MAX_DIFFICULTY + 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            PoWError::DifficultyTooHigh { required, max } => {
                assert_eq!(required, MAX_DIFFICULTY + 1);
                assert_eq!(max, MAX_DIFFICULTY);
            }
            _ => panic!("Expected DifficultyTooHigh error"),
        }
    }

    #[test]
    fn test_pow_at_max_difficulty_allowed() {
        // Verify that max difficulty doesn't get rejected
        let data = b"max difficulty allowed";
        // This will likely timeout, but should NOT return DifficultyTooHigh
        let result =
            ProofOfWork::mine_with_timeout(data, MAX_DIFFICULTY, Duration::from_millis(50));
        match result {
            Err(PoWError::DifficultyTooHigh { .. }) => panic!("Max difficulty should be allowed"),
            Err(PoWError::Timeout { .. }) | Err(PoWError::Cancelled) => {}
            Ok(_) => {}
        }
    }
}

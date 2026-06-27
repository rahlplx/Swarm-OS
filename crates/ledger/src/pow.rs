use sha2::{Digest, Sha256};

pub struct ProofOfWork;

impl ProofOfWork {
    pub fn mine(data: &[u8], difficulty: u32) -> u64 {
        let prefix = "0".repeat(difficulty as usize);
        let mut nonce: u64 = 0;

        loop {
            let hash = Self::compute_hash(data, nonce);
            let hash_hex = hex::encode(hash);
            if hash_hex.starts_with(&prefix) {
                return nonce;
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

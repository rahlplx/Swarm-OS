use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type BlockHash = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub parent_hash: BlockHash,
    pub data: Vec<u8>,
    pub nonce: u64,
    pub timestamp: i64,
    pub hash: BlockHash,
}

impl Block {
    pub fn new(parent_hash: BlockHash, data: Vec<u8>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let hash = Self::compute_hash(parent_hash, &data, 0, timestamp);
        Self {
            parent_hash,
            data,
            nonce: 0,
            timestamp,
            hash,
        }
    }

    pub fn compute_hash(
        parent_hash: BlockHash,
        data: &[u8],
        nonce: u64,
        timestamp: i64,
    ) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(parent_hash);
        hasher.update(data);
        hasher.update(nonce.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

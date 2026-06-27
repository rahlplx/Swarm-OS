use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreditCommitmentToken {
    pub id: Uuid,
    pub amount: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature: Vec<u8>,
}

impl CreditCommitmentToken {
    pub fn new(amount: u64, secret_key: &[u8]) -> Self {
        let created_at = Utc::now();
        let expires_at = created_at + chrono::Duration::minutes(5);

        // Simple HMAC signature
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac =
            HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
        mac.update(&amount.to_le_bytes());
        mac.update(&created_at.timestamp().to_le_bytes());
        let signature = mac.finalize().into_bytes().to_vec();

        Self {
            id: Uuid::new_v4(),
            amount,
            created_at,
            expires_at,
            signature,
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn verify(&self, secret_key: &[u8]) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac =
            HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
        mac.update(&self.amount.to_le_bytes());
        mac.update(&self.created_at.timestamp().to_le_bytes());
        mac.verify_slice(&self.signature).is_ok()
    }
}

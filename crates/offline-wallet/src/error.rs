use std::fmt;

#[derive(Debug)]
pub enum WalletError {
    InsufficientCredits { available: u64, requested: u64 },
    InsufficientLocked { locked: u64, requested: u64 },
    CctExpired { cct_id: String },
    CctInvalidSignature { cct_id: String },
    CctAlreadyRedeemed { cct_id: String },
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::InsufficientCredits {
                available,
                requested,
            } => {
                write!(
                    f,
                    "Insufficient credits: available {}, requested {}",
                    available, requested
                )
            }
            WalletError::InsufficientLocked { locked, requested } => {
                write!(
                    f,
                    "Insufficient locked credits: locked {}, requested {}",
                    locked, requested
                )
            }
            WalletError::CctExpired { cct_id } => {
                write!(f, "CCT expired: {}", cct_id)
            }
            WalletError::CctInvalidSignature { cct_id } => {
                write!(f, "CCT invalid signature: {}", cct_id)
            }
            WalletError::CctAlreadyRedeemed { cct_id } => {
                write!(f, "CCT already redeemed: {}", cct_id)
            }
        }
    }
}

impl std::error::Error for WalletError {}

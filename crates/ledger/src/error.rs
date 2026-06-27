use std::fmt;

#[derive(Debug)]
pub enum LedgerError {
    BlockNotFound { hash: String },
    InvalidHash { hash: String },
    CycleDetected { hash: String },
    MiningTimeout { attempts: u64 },
    MiningCancelled,
    InsufficientDifficulty { required: u32, max: u32 },
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::BlockNotFound { hash } => write!(f, "Block not found: {}", hash),
            LedgerError::InvalidHash { hash } => write!(f, "Invalid hash: {}", hash),
            LedgerError::CycleDetected { hash } => write!(f, "Cycle detected at block: {}", hash),
            LedgerError::MiningTimeout { attempts } => {
                write!(f, "Mining timeout after {} attempts", attempts)
            }
            LedgerError::MiningCancelled => write!(f, "Mining cancelled"),
            LedgerError::InsufficientDifficulty { required, max } => {
                write!(
                    f,
                    "Insufficient difficulty: required {}, max allowed {}",
                    required, max
                )
            }
        }
    }
}

impl std::error::Error for LedgerError {}

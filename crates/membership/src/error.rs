use std::fmt;

#[derive(Debug)]
pub enum MembershipError {
    MaxPeersReached { current: usize, max: usize },
    NodeNotFound { node_id: String },
    AlreadyRunning,
    NotRunning,
}

impl fmt::Display for MembershipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MembershipError::MaxPeersReached { current, max } => {
                write!(f, "Max peers reached: current {}, max {}", current, max)
            }
            MembershipError::NodeNotFound { node_id } => {
                write!(f, "Node not found: {}", node_id)
            }
            MembershipError::AlreadyRunning => write!(f, "Mesh already running"),
            MembershipError::NotRunning => write!(f, "Mesh not running"),
        }
    }
}

impl std::error::Error for MembershipError {}

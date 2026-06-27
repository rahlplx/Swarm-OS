use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    pub fn generate() -> Self {
        let key = libp2p::identity::Keypair::generate_ed25519();
        Self(key.public().to_peer_id().to_string())
    }

    pub fn peer_id(&self) -> Result<libp2p::PeerId, Box<dyn std::error::Error>> {
        Ok(self.0.parse()?)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpec {
    pub cpu_cores: u32,
    pub ram_mb: u32,
    pub has_gpu: bool,
    pub gpu_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Online,
    Idle,
    Busy,
    Offline,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub addr: String,
    pub hardware: HardwareSpec,
    pub models: Vec<String>,
    pub status: NodeStatus,
}

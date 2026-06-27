use crate::{MembershipService, NodeId};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct GossipMesh {
    port: u16,
    peers: Arc<Mutex<HashMap<NodeId, String>>>,
    running: Arc<Mutex<bool>>,
}

impl GossipMesh {
    pub async fn new(port: u16) -> Result<Self> {
        Ok(Self {
            port,
            peers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        })
    }
}

#[async_trait]
impl MembershipService for GossipMesh {
    async fn start(&mut self) -> Result<()> {
        *self.running.lock().unwrap() = true;
        tracing::info!("GossipMesh started on port {}", self.port);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.running.lock().unwrap() = false;
        tracing::info!("GossipMesh stopped");
        Ok(())
    }

    async fn join(&mut self, node: NodeId) -> Result<()> {
        let node_str = node.to_string();
        self.peers.lock().unwrap().insert(node, String::new());
        tracing::info!("Node joined: {}", node_str);
        Ok(())
    }

    async fn leave(&mut self, node: NodeId) -> Result<()> {
        let node_str = node.to_string();
        self.peers.lock().unwrap().remove(&node);
        tracing::info!("Node left: {}", node_str);
        Ok(())
    }

    async fn heartbeat(&self) -> Result<()> {
        let peer_count = self.peers.lock().unwrap().len();
        tracing::debug!("Heartbeat: {} peers connected", peer_count);
        Ok(())
    }

    fn peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }

    fn peers(&self) -> Vec<NodeId> {
        self.peers.lock().unwrap().keys().cloned().collect()
    }
}

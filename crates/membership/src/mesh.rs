use crate::{MembershipService, NodeId};
use anyhow::{bail, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct GossipMesh {
    port: u16,
    peers: Arc<RwLock<HashMap<NodeId, String>>>,
    running: Arc<RwLock<bool>>,
    max_peers: usize,
}

impl GossipMesh {
    pub const DEFAULT_MAX_PEERS: usize = 256;

    pub async fn new(port: u16) -> Result<Self> {
        Ok(Self::with_max_peers(port, Self::DEFAULT_MAX_PEERS))
    }

    pub fn with_max_peers(port: u16, max_peers: usize) -> Self {
        Self {
            port,
            peers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            max_peers,
        }
    }

    pub fn max_peers(&self) -> usize {
        self.max_peers
    }
}

#[async_trait]
impl MembershipService for GossipMesh {
    async fn start(&mut self) -> Result<()> {
        *self.running.write().unwrap() = true;
        tracing::info!("GossipMesh started on port {}", self.port);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.running.write().unwrap() = false;
        tracing::info!("GossipMesh stopped");
        Ok(())
    }

    async fn join(&mut self, node: NodeId) -> Result<()> {
        {
            let peers = self.peers.read().unwrap();
            if peers.len() >= self.max_peers {
                bail!("Max peer limit reached: {}/{}", peers.len(), self.max_peers);
            }
        }
        let node_str = node.to_string();
        self.peers.write().unwrap().insert(node, String::new());
        tracing::info!("Node joined: {}", node_str);
        Ok(())
    }

    async fn leave(&mut self, node: NodeId) -> Result<()> {
        let node_str = node.to_string();
        self.peers.write().unwrap().remove(&node);
        tracing::info!("Node left: {}", node_str);
        Ok(())
    }

    async fn heartbeat(&self) -> Result<()> {
        let peer_count = self.peers.read().unwrap().len();
        tracing::debug!("Heartbeat: {} peers connected", peer_count);
        Ok(())
    }

    fn peer_count(&self) -> usize {
        self.peers.read().unwrap().len()
    }

    fn peers(&self) -> Vec<NodeId> {
        self.peers.read().unwrap().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_max_peer_limit() {
        let mut mesh = GossipMesh::with_max_peers(4242, 2);
        mesh.start().await.unwrap();

        let node1 = NodeId::generate();
        let node2 = NodeId::generate();
        let node3 = NodeId::generate();

        mesh.join(node1.clone()).await.unwrap();
        mesh.join(node2.clone()).await.unwrap();

        // Third join should fail
        let result = mesh.join(node3).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max peer limit"));
        assert_eq!(mesh.peer_count(), 2);

        // After one leaves, new join should work
        mesh.leave(node1).await.unwrap();
        mesh.join(NodeId::generate()).await.unwrap();
        assert_eq!(mesh.peer_count(), 2);
    }
}

use crate::NodeId;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait MembershipService {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn join(&mut self, node: NodeId) -> Result<()>;
    async fn leave(&mut self, node: NodeId) -> Result<()>;
    async fn heartbeat(&self) -> Result<()>;
    fn peer_count(&self) -> usize;
    fn peers(&self) -> Vec<NodeId>;
}

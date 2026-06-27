use std::time::Duration;
use swarm_membership::{GossipMesh, MembershipService, NodeId, NodeInfo};

#[tokio::test]
async fn test_node_id_creation() {
    let id = NodeId::generate();
    assert!(!id.as_str().is_empty());
}

#[tokio::test]
async fn test_node_info_serialization() {
    let info = NodeInfo {
        node_id: NodeId::generate(),
        addr: "/ip4/127.0.0.1/tcp/4001".to_string(),
        hardware: swarm_membership::HardwareSpec {
            cpu_cores: 8,
            ram_mb: 16384,
            has_gpu: false,
            gpu_model: None,
        },
        models: vec!["mistral-7b".to_string()],
        status: swarm_membership::NodeStatus::Online,
    };
    let json = serde_json::to_string(&info).unwrap();
    let deserialized: NodeInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(info.hardware.cpu_cores, deserialized.hardware.cpu_cores);
    assert_eq!(info.models, deserialized.models);
}

#[tokio::test]
async fn test_gossip_mesh_creation() {
    let mesh = GossipMesh::new(4001).await;
    assert!(mesh.is_ok());
    let mesh = mesh.unwrap();
    assert_eq!(mesh.peer_count(), 0);
}

#[tokio::test]
async fn test_join_and_leave() {
    let mut mesh = GossipMesh::new(4002).await.unwrap();

    // mesh1 should be able to start without error
    mesh.start().await.unwrap();

    // Give time for initialization
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Peer count should be 0 (no one to connect to yet)
    assert_eq!(mesh.peer_count(), 0);
}

#[tokio::test]
async fn test_heartbeat_mechanism() {
    let mut mesh = GossipMesh::new(4004).await.unwrap();
    mesh.start().await.unwrap();

    // Heartbeat should not panic
    let result = mesh.heartbeat().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_membership_service_trait() {
    let mesh = GossipMesh::new(4005).await.unwrap();

    // Verify trait is implemented
    fn assert_service<T: MembershipService>(_m: &T) {}
    assert_service(&mesh);
}

use swarm_ledger::{MerkleDAG, ProofOfWork};
use swarm_membership::{GossipMesh, MembershipService, NodeId};
use swarm_offline_wallet::OfflineWallet;

/// Test cross-crate workflow: Create wallet, mine block with PoW, store in DAG
#[tokio::test]
async fn test_full_workflow_wallet_to_dag() {
    let mut wallet = OfflineWallet::new(1000);
    let mut dag = MerkleDAG::new();

    // 1. Create genesis block
    let genesis = dag.append_genesis(b"network genesis".to_vec());
    assert_eq!(dag.height(), 1);

    // 2. Create CCT
    let cct = wallet.create_cct(100).unwrap();
    assert_eq!(cct.amount, 100);

    // 3. Mine a block with PoW
    let block_data = format!("CCT:amount={}", cct.amount).into_bytes();
    let pow_input = [genesis.as_ref(), &block_data].concat();
    let nonce =
        ProofOfWork::mine_with_timeout(&pow_input, 2, std::time::Duration::from_secs(5)).unwrap();
    assert!(ProofOfWork::verify(&pow_input, nonce, 2));

    // 4. Append block to DAG
    let block_hash = dag.append(genesis, block_data);
    assert_eq!(dag.height(), 2);
    assert!(dag.validate_chain(block_hash).is_ok());

    // 5. Redeem CCT
    wallet.redeem_cct(&cct).unwrap();
    assert_eq!(wallet.balance().amount, 1100);
}

/// Test node registration with mesh
#[tokio::test]
async fn test_node_registration_with_mesh() {
    let mut mesh = GossipMesh::new(4242).await.unwrap();
    mesh.start().await.unwrap();

    let node1 = NodeId::generate();
    let node2 = NodeId::generate();

    mesh.join(node1.clone()).await.unwrap();
    mesh.join(node2.clone()).await.unwrap();
    assert_eq!(mesh.peer_count(), 2);

    mesh.leave(node1).await.unwrap();
    assert_eq!(mesh.peer_count(), 1);

    mesh.stop().await.unwrap();
}

/// Test CCT replay prevention across wallet instances
#[test]
fn test_cct_replay_prevention_cross_instance() {
    let mut wallet_a = OfflineWallet::new(1000);
    let mut wallet_b = OfflineWallet::new(1000);

    let cct = wallet_a.create_cct(200).unwrap();
    wallet_a.redeem_cct(&cct).unwrap();
    assert_eq!(wallet_a.balance().amount, 1200);

    let result = wallet_b.redeem_cct(&cct);
    assert!(result.is_err());
    assert_eq!(wallet_b.balance().amount, 1000);
}

/// Test mesh with max peers and balance limits
#[tokio::test]
async fn test_mesh_and_balance_limits() {
    let mut mesh = GossipMesh::with_max_peers(4242, 3);
    mesh.start().await.unwrap();

    let mut wallet = OfflineWallet::new(500);

    for _ in 0..3 {
        mesh.join(NodeId::generate()).await.unwrap();
    }

    let extra = NodeId::generate();
    assert!(mesh.join(extra).await.is_err());

    let result = wallet.create_cct(600);
    assert!(result.is_err());

    mesh.stop().await.unwrap();
}

/// Test chain validation with multiple blocks
#[test]
fn test_chain_validation_full() {
    let mut dag = MerkleDAG::new();
    let mut parent = dag.append_genesis(b"block 0".to_vec());

    for i in 1..10 {
        parent = dag.append(parent, format!("block {}", i).into_bytes());
    }

    assert!(dag.validate_chain(parent).is_ok());
    assert_eq!(dag.height(), 10);
    assert_eq!(dag.chain_hashes().len(), 10);
}

/// Test PoW difficulty cap enforcement
#[test]
fn test_pow_difficulty_cap_enforcement() {
    let result = ProofOfWork::mine(b"test", 17);
    assert!(result.is_err());
}

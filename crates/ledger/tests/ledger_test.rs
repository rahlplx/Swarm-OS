use swarm_ledger::{Block, BlockHash, MerkleDAG, ProofOfWork};

#[test]
fn test_block_creation() {
    let block = Block::new([0u8; 32], b"test data".to_vec());
    assert_ne!(block.hash, [0u8; 32]);
    assert_eq!(block.parent_hash, [0u8; 32]);
    assert_eq!(block.data, b"test data");
    assert_eq!(block.nonce, 0);
}

#[test]
fn test_block_hash_deterministic() {
    let block1 = Block::new([1u8; 32], b"hello".to_vec());
    let block2 = Block::new([1u8; 32], b"hello".to_vec());
    assert_eq!(block1.hash, block2.hash);
}

#[test]
fn test_block_hash_different_data() {
    let block1 = Block::new([0u8; 32], b"hello".to_vec());
    let block2 = Block::new([0u8; 32], b"world".to_vec());
    assert_ne!(block1.hash, block2.hash);
}

#[test]
fn test_merkle_dag_genesis() {
    let dag = MerkleDAG::new();
    assert_eq!(dag.height(), 0);
    assert!(dag.genesis_hash().is_none());
}

#[test]
fn test_merkle_dag_append() {
    let mut dag = MerkleDAG::new();
    let genesis = dag.append_genesis(b"genesis block".to_vec());
    assert_eq!(dag.height(), 1);
    assert_eq!(dag.genesis_hash(), Some(&genesis));

    let second = dag.append(genesis, b"second block".to_vec());
    assert_eq!(dag.height(), 2);
    assert_ne!(genesis, second);
}

#[test]
fn test_merkle_dag_validate_chain() {
    let mut dag = MerkleDAG::new();
    let h1 = dag.append_genesis(b"block 1".to_vec());
    let h2 = dag.append(h1, b"block 2".to_vec());
    let h3 = dag.append(h2, b"block 3".to_vec());

    assert!(dag.validate_chain(h3).is_ok());
}

#[test]
fn test_merkle_dag_invalid_chain() {
    let mut dag = MerkleDAG::new();
    let h1 = dag.append_genesis(b"block 1".to_vec());
    let _h2 = dag.append(h1, b"block 2".to_vec());

    // Try to validate with wrong hash
    let wrong_hash = [99u8; 32];
    assert!(dag.validate_chain(wrong_hash).is_err());
}

#[test]
fn test_pow_mine_and_verify() {
    let data = b"test data for mining";
    let nonce = ProofOfWork::mine(data, 2); // difficulty 2 = 2 leading zeros
    assert!(ProofOfWork::verify(data, nonce, 2));
}

#[test]
fn test_pow_invalid_nonce() {
    let data = b"test data";
    assert!(!ProofOfWork::verify(data, 0, 2));
}

#[test]
fn test_pow_higher_difficulty() {
    let data = b"harder puzzle";
    let nonce = ProofOfWork::mine(data, 4); // difficulty 4
    assert!(ProofOfWork::verify(data, nonce, 4));
}

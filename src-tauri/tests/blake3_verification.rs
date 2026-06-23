use std::io::Write;
use swarm_os_lib::inference::model_manager::ModelManager;
use tempfile::TempDir;

#[test]
fn blake3_hash_and_verify_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.gguf");
    std::fs::write(&path, b"test model content for blake3 verification").unwrap();

    let hash = ModelManager::compute_blake3(&path).unwrap();
    assert_eq!(hash.len(), 64);

    assert!(ModelManager::verify_file(&path, &hash).unwrap());
}

#[test]
fn blake3_detects_corruption() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.gguf");
    std::fs::write(&path, b"original content").unwrap();

    let hash = ModelManager::compute_blake3(&path).unwrap();

    let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    file.write_all(b"corrupted!").unwrap();
    drop(file);

    assert!(!ModelManager::verify_file(&path, &hash).unwrap());
}

#[test]
fn blake3_missing_file_returns_error() {
    let result = ModelManager::compute_blake3(std::path::Path::new("/nonexistent.gguf"));
    assert!(result.is_err());
}

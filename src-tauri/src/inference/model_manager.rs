use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("model file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub blake3_hash: Option<String>,
    pub status: ModelStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    Available,
    Downloading { progress: f32 },
    Verified,
    Corrupted,
}

pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    pub fn compute_blake3(path: &Path) -> Result<String, ModelError> {
        if !path.exists() {
            return Err(ModelError::FileNotFound(path.to_path_buf()));
        }
        let mut file = std::fs::File::open(path)?;
        let mut hasher = blake3::Hasher::new();
        std::io::copy(&mut file, &mut hasher)?;
        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn verify_file(path: &Path, expected_hash: &str) -> Result<bool, ModelError> {
        let actual = Self::compute_blake3(path)?;
        Ok(actual == expected_hash)
    }

    pub fn list_models(&self) -> Result<Vec<ModelInfo>, ModelError> {
        let mut models = Vec::new();

        if !self.models_dir.exists() {
            return Ok(models);
        }

        let entries = std::fs::read_dir(&self.models_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                let metadata = std::fs::metadata(&path)?;
                models.push(ModelInfo {
                    name: path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path,
                    size_bytes: metadata.len(),
                    blake3_hash: None,
                    status: ModelStatus::Available,
                });
            }
        }
        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_temp_model() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let model_path = dir.path().join("test-model.gguf");
        let mut file = std::fs::File::create(&model_path).unwrap();
        file.write_all(b"fake gguf content for testing").unwrap();
        (dir, model_path)
    }

    #[test]
    fn blake3_hash_of_valid_file() {
        let (_dir, path) = setup_temp_model();
        let hash = ModelManager::compute_blake3(&path).unwrap();
        assert_eq!(hash.len(), 64); // BLAKE3 hex is 64 chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn blake3_hash_of_missing_file() {
        let result = ModelManager::compute_blake3(Path::new("/nonexistent/model.gguf"));
        assert!(matches!(result, Err(ModelError::FileNotFound(_))));
    }

    #[test]
    fn verify_file_correct_hash() {
        let (_dir, path) = setup_temp_model();
        let hash = ModelManager::compute_blake3(&path).unwrap();
        assert!(ModelManager::verify_file(&path, &hash).unwrap());
    }

    #[test]
    fn verify_file_wrong_hash() {
        let (_dir, path) = setup_temp_model();
        let result = ModelManager::verify_file(
            &path,
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        assert!(!result);
    }

    #[test]
    fn list_models_empty_dir() {
        let dir = TempDir::new().unwrap();
        let manager = ModelManager::new(dir.path().to_path_buf());
        let models = manager.list_models().unwrap();
        assert!(models.is_empty());
    }

    #[test]
    fn list_models_finds_gguf_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("model-a.gguf"), b"fake").unwrap();
        std::fs::write(dir.path().join("model-b.gguf"), b"fake2").unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"not a model").unwrap();

        let manager = ModelManager::new(dir.path().to_path_buf());
        let models = manager.list_models().unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.iter().all(|m| m.status == ModelStatus::Available));
    }

    #[test]
    fn list_models_nonexistent_dir() {
        let manager = ModelManager::new(PathBuf::from("/nonexistent/models"));
        let models = manager.list_models().unwrap();
        assert!(models.is_empty());
    }
}

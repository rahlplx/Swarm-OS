use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("model not found: {0}")]
    ModelNotFound(String),
    #[error("insufficient VRAM: need {need_bytes} bytes, have {have_bytes} bytes")]
    InsufficientVram { need_bytes: u64, have_bytes: u64 },
    #[error("model already loaded: {0}")]
    AlreadyLoaded(String),
    #[error("no model loaded")]
    NoModelLoaded,
    #[error("inference failed: {0}")]
    InferenceFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
}

impl Default for InferenceRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub text: String,
    pub tokens_generated: u32,
    pub tokens_per_second: f64,
}

pub trait InferenceEngine: Send + Sync {
    fn load_model(&mut self, path: &str) -> Result<(), InferenceError>;
    fn unload_model(&mut self) -> Result<(), InferenceError>;
    fn is_loaded(&self) -> bool;
    fn complete(&self, request: &InferenceRequest) -> Result<InferenceResponse, InferenceError>;
}

pub struct PlaceholderEngine {
    loaded_model: Option<String>,
}

impl PlaceholderEngine {
    pub fn new() -> Self {
        Self { loaded_model: None }
    }
}

impl Default for PlaceholderEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine for PlaceholderEngine {
    fn load_model(&mut self, path: &str) -> Result<(), InferenceError> {
        if self.loaded_model.is_some() {
            return Err(InferenceError::AlreadyLoaded(path.to_string()));
        }
        if !std::path::Path::new(path).exists() {
            return Err(InferenceError::ModelNotFound(path.to_string()));
        }
        self.loaded_model = Some(path.to_string());
        Ok(())
    }

    fn unload_model(&mut self) -> Result<(), InferenceError> {
        if self.loaded_model.is_none() {
            return Err(InferenceError::NoModelLoaded);
        }
        self.loaded_model = None;
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.loaded_model.is_some()
    }

    fn complete(&self, _request: &InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        if !self.is_loaded() {
            return Err(InferenceError::NoModelLoaded);
        }
        Ok(InferenceResponse {
            text: "placeholder response".to_string(),
            tokens_generated: 2,
            tokens_per_second: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEngine {
        loaded: bool,
    }

    impl MockEngine {
        fn new() -> Self {
            Self { loaded: false }
        }
    }

    impl InferenceEngine for MockEngine {
        fn load_model(&mut self, path: &str) -> Result<(), InferenceError> {
            if path == "missing.gguf" {
                return Err(InferenceError::ModelNotFound(path.to_string()));
            }
            self.loaded = true;
            Ok(())
        }

        fn unload_model(&mut self) -> Result<(), InferenceError> {
            if !self.loaded {
                return Err(InferenceError::NoModelLoaded);
            }
            self.loaded = false;
            Ok(())
        }

        fn is_loaded(&self) -> bool {
            self.loaded
        }

        fn complete(
            &self,
            request: &InferenceRequest,
        ) -> Result<InferenceResponse, InferenceError> {
            if !self.loaded {
                return Err(InferenceError::NoModelLoaded);
            }
            Ok(InferenceResponse {
                text: format!("response to: {}", request.prompt),
                tokens_generated: 5,
                tokens_per_second: 42.0,
            })
        }
    }

    #[test]
    fn mock_engine_load_and_complete() {
        let mut engine = MockEngine::new();
        engine.load_model("test.gguf").unwrap();
        assert!(engine.is_loaded());

        let request = InferenceRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        };
        let response = engine.complete(&request).unwrap();
        assert_eq!(response.text, "response to: hello");
        assert_eq!(response.tokens_generated, 5);
    }

    #[test]
    fn mock_engine_model_not_found() {
        let mut engine = MockEngine::new();
        let result = engine.load_model("missing.gguf");
        assert!(matches!(result, Err(InferenceError::ModelNotFound(_))));
    }

    #[test]
    fn mock_engine_complete_without_load() {
        let engine = MockEngine::new();
        let request = InferenceRequest::default();
        let result = engine.complete(&request);
        assert!(matches!(result, Err(InferenceError::NoModelLoaded)));
    }

    #[test]
    fn mock_engine_double_unload() {
        let mut engine = MockEngine::new();
        let result = engine.unload_model();
        assert!(matches!(result, Err(InferenceError::NoModelLoaded)));
    }

    #[test]
    fn inference_request_default_values() {
        let req = InferenceRequest::default();
        assert_eq!(req.max_tokens, 256);
        assert!((req.temperature - 0.7).abs() < f32::EPSILON);
        assert!((req.top_p - 0.9).abs() < f32::EPSILON);
    }
}

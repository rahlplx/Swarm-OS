use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteLLMConfig {
    pub host: String,
    pub port: u16,
    pub llama_server_url: String,
}

impl Default for LiteLLMConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 4000,
            llama_server_url: "http://127.0.0.1:8080".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = LiteLLMConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 4000);
        assert_eq!(config.llama_server_url, "http://127.0.0.1:8080");
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = LiteLLMConfig {
            host: "0.0.0.0".to_string(),
            port: 5000,
            llama_server_url: "http://localhost:9090".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: LiteLLMConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port, 5000);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamToken {
    pub token: String,
    pub done: bool,
    pub tokens_per_sec: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_token_serde_roundtrip() {
        let token = StreamToken {
            token: "hello".to_string(),
            done: false,
            tokens_per_sec: 42.5,
        };
        let json = serde_json::to_string(&token).unwrap();
        let deserialized: StreamToken = serde_json::from_str(&json).unwrap();
        assert_eq!(token, deserialized);
    }

    #[test]
    fn stream_token_done_marker() {
        let token = StreamToken {
            token: String::new(),
            done: true,
            tokens_per_sec: 38.2,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"done\":true"));
    }
}

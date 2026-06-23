use crate::inference::StreamToken;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    Token(StreamToken),
    HardwareUpdate(crate::hardware::HardwareProfile),
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::profiler::HardwareProfile;
    use crate::inference::streaming::StreamToken;

    #[test]
    fn ipc_message_token_roundtrip() {
        let msg = IpcMessage::Token(StreamToken {
            token: "world".to_string(),
            done: false,
            tokens_per_sec: 55.3,
        });
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: IpcMessage = serde_json::from_str(&json).unwrap();
        if let IpcMessage::Token(t) = deserialized {
            assert_eq!(t.token, "world");
            assert!(!t.done);
        } else {
            panic!("expected Token variant");
        }
    }

    #[test]
    fn ipc_message_hardware_roundtrip() {
        let msg = IpcMessage::HardwareUpdate(HardwareProfile {
            cpu_cores: 4,
            cpu_name: "Test".to_string(),
            ram_total_bytes: 8 * 1024 * 1024 * 1024,
            ram_available_bytes: 4 * 1024 * 1024 * 1024,
            gpus: vec![],
            os: "Linux".to_string(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: IpcMessage = serde_json::from_str(&json).unwrap();
        if let IpcMessage::HardwareUpdate(h) = deserialized {
            assert_eq!(h.cpu_cores, 4);
        } else {
            panic!("expected HardwareUpdate variant");
        }
    }

    #[test]
    fn ipc_message_error_roundtrip() {
        let msg = IpcMessage::Error("something broke".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: IpcMessage = serde_json::from_str(&json).unwrap();
        if let IpcMessage::Error(e) = deserialized {
            assert_eq!(e, "something broke");
        } else {
            panic!("expected Error variant");
        }
    }
}

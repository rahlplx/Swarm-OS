use super::bridge::LiteLLMConfig;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LiteLLMError {
    #[error("failed to start LiteLLM process: {0}")]
    StartFailed(String),
    #[error("LiteLLM process not running")]
    NotRunning,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct LiteLLMProcess {
    config: LiteLLMConfig,
    child: Option<std::process::Child>,
}

impl LiteLLMProcess {
    pub fn new(config: LiteLLMConfig) -> Self {
        Self {
            config,
            child: None,
        }
    }

    pub fn config(&self) -> &LiteLLMConfig {
        &self.config
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    pub fn start(&mut self) -> Result<(), LiteLLMError> {
        if self.child.is_some() {
            return Ok(());
        }

        let child = std::process::Command::new("litellm")
            .args([
                "--host",
                &self.config.host,
                "--port",
                &self.config.port.to_string(),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| LiteLLMError::StartFailed(e.to_string()))?;

        self.child = Some(child);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), LiteLLMError> {
        match self.child.take() {
            Some(mut child) => {
                child.kill().map_err(LiteLLMError::Io)?;
                child.wait().map_err(LiteLLMError::Io)?;
                Ok(())
            }
            None => Err(LiteLLMError::NotRunning),
        }
    }
}

impl Drop for LiteLLMProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

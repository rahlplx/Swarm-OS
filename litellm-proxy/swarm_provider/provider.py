import httpx

from .config import SwarmProviderConfig


class SwarmProvider:
    def __init__(self, config: SwarmProviderConfig | None = None):
        self.config = config or SwarmProviderConfig()

    def completion(self, messages: list[dict], **kwargs) -> dict:
        url = f"{self.config.llama_server_url}/v1/chat/completions"
        payload = {
            "model": kwargs.get("model", self.config.model_name),
            "messages": messages,
            "max_tokens": kwargs.get("max_tokens", 256),
            "temperature": kwargs.get("temperature", 0.7),
            "stream": kwargs.get("stream", False),
        }

        with httpx.Client(timeout=self.config.timeout) as client:
            response = client.post(url, json=payload)
            response.raise_for_status()
            return response.json()

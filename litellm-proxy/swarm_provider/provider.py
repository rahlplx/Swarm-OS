"""Swarm-OS LiteLLM custom provider.

Routes chat completion requests to a local llama.cpp server.
"""

from __future__ import annotations

import httpx

from swarm_provider.config import SwarmProviderConfig


class SwarmProviderError(Exception):
    """Base error for all SwarmProvider failures."""


class SwarmProviderConnectionError(SwarmProviderError):
    """Failed to reach the llama.cpp server."""


class SwarmProviderResponseError(SwarmProviderError):
    """llama.cpp server returned a non-2xx response."""


class SwarmProvider:
    """Routes OpenAI-compatible chat completions to a local llama.cpp server."""

    def __init__(self, config: SwarmProviderConfig | None = None):
        self.config = config or SwarmProviderConfig()

    def completion(self, messages: list[dict], **kwargs) -> dict:
        """Send a chat completion request to the llama.cpp server.

        Args:
            messages: OpenAI-format message list.
            **kwargs: Override defaults: model, max_tokens, temperature, stream.

        Returns:
            The JSON response body from the llama.cpp server.

        Raises:
            SwarmProviderConnectionError: Network failure (DNS, refused, timeout).
            SwarmProviderResponseError: HTTP 4xx/5xx from the server.
        """
        url = f"{self.config.llama_server_url.rstrip('/')}/v1/chat/completions"
        payload = {
            "model": kwargs.get("model", self.config.model_name),
            "messages": messages,
            "max_tokens": kwargs.get("max_tokens", 256),
            "temperature": kwargs.get("temperature", 0.7),
            "stream": kwargs.get("stream", False),
        }

        try:
            with httpx.Client(timeout=self.config.timeout) as client:
                response = client.post(url, json=payload)
                response.raise_for_status()
                return response.json()
        except httpx.TimeoutException as exc:
            raise SwarmProviderConnectionError(
                f"Timed out connecting to llama.cpp server at {url}: {exc}"
            ) from exc
        except httpx.ConnectError as exc:
            raise SwarmProviderConnectionError(
                f"Cannot reach llama.cpp server at {url}: {exc}"
            ) from exc
        except httpx.HTTPStatusError as exc:
            raise SwarmProviderResponseError(
                f"llama.cpp server returned HTTP {exc.response.status_code}: "
                f"{exc.response.text[:200]}"
            ) from exc

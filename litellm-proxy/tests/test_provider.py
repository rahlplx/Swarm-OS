import httpx
import respx

from swarm_provider.config import SwarmProviderConfig
from swarm_provider.provider import SwarmProvider


@respx.mock
def test_provider_routes_to_llama_server(llama_completion_response):
    respx.post("http://127.0.0.1:8080/v1/chat/completions").mock(
        return_value=httpx.Response(200, json=llama_completion_response)
    )

    provider = SwarmProvider(SwarmProviderConfig())
    result = provider.completion([{"role": "user", "content": "hello"}])

    assert result["choices"][0]["message"]["content"] == "Hello! How can I help?"
    assert result["usage"]["total_tokens"] == 18


@respx.mock
def test_provider_sends_correct_payload(llama_completion_response):
    route = respx.post("http://127.0.0.1:8080/v1/chat/completions").mock(
        return_value=httpx.Response(200, json=llama_completion_response)
    )

    provider = SwarmProvider(SwarmProviderConfig())
    provider.completion(
        [{"role": "user", "content": "test"}],
        max_tokens=100,
        temperature=0.5,
    )

    request = route.calls[0].request
    import json

    body = json.loads(request.content)
    assert body["max_tokens"] == 100
    assert body["temperature"] == 0.5
    assert body["messages"][0]["content"] == "test"


def test_provider_default_config():
    config = SwarmProviderConfig()
    assert config.llama_server_url == "http://127.0.0.1:8080"
    assert config.timeout == 120.0

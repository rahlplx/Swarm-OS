"""Swarm-OS LiteLLM custom provider.

Routes OpenAI-compatible chat completion requests to a local llama.cpp
server (via llama-server). Used by the LiteLLM proxy gateway.
"""

from swarm_provider.callback import TokenCounter
from swarm_provider.config import SwarmProviderConfig
from swarm_provider.provider import SwarmProvider

__all__ = [
    "SwarmProvider",
    "SwarmProviderConfig",
    "TokenCounter",
]

__version__ = "0.1.0"

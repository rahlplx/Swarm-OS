"""
SwarmRouter: routes OpenAI-compatible requests to local llama.cpp.
Phase 0: single-device only. No swarm routing, no etcd, no ledger.

LiteLLM pinned to >=1.82.0 — CustomLogger hooks are unstable below this.
"""
from typing import AsyncIterator

LITELLM_MIN_VERSION = "1.82.0"

# Canonical backend bonus values from architecture.md §3:
# cuda=10 / metal=8 / vulkan=5 / cpu=0
BACKEND_BONUS = {"cuda": 10.0, "metal": 8.0, "vulkan": 5.0, "cpu": 0.0}


def capability_score(
    vram_gib: float,
    ram_gib: float,
    cpu_cores: int,
    backend: str,
) -> float:
    """Score formula: vram×4 + ram×0.5 + cpu×0.25 + backend_bonus (architecture.md §3)."""
    normalized = str(backend).strip().lower() if backend is not None else "cpu"
    bonus = BACKEND_BONUS.get(normalized, 0.0)
    return (vram_gib * 4.0) + (ram_gib * 0.5) + (cpu_cores * 0.25) + bonus


class SwarmRouter:
    """Phase 0: forwards requests to local llama.cpp server on localhost:8080."""

    def __init__(self, llama_url: str = "http://localhost:8080"):
        self.llama_url = llama_url

    async def chat_completions(
        self,
        model: str,
        messages: list[dict],
        max_tokens: int = 512,
        stream: bool = False,
        temperature: float = 0.7,
    ) -> dict | AsyncIterator[dict]:
        """Route OpenAI-compatible chat request to local llama.cpp inference engine."""
        raise NotImplementedError("Phase 0 implementation pending — wire to llama.cpp /v1/chat/completions")

import pytest
from swarm_gateway.router import SwarmRouter, capability_score


@pytest.fixture
def router():
    return SwarmRouter(llama_url="http://localhost:8080")


def test_router_instantiates(router):
    assert router.llama_url == "http://localhost:8080"


@pytest.mark.asyncio
async def test_chat_completions_not_yet_implemented(router):
    with pytest.raises(NotImplementedError):
        await router.chat_completions(
            model="llama-3.1-8b",
            messages=[{"role": "user", "content": "hello"}],
        )


def test_capability_score_rtx4090():
    # (24*4) + (32*0.5) + (8*0.25) + 20 = 96 + 16 + 2 + 20 = 134
    score = capability_score(vram_gib=24.0, ram_gib=32.0, cpu_cores=8, backend="cuda")
    assert score == 134.0, f"expected 134.0, got {score}"


def test_capability_score_cpu_only():
    # (0*4) + (8*0.5) + (4*0.25) + 0 = 0 + 4 + 1 + 0 = 5
    score = capability_score(vram_gib=0.0, ram_gib=8.0, cpu_cores=4, backend="cpu")
    assert score == 5.0, f"expected 5.0, got {score}"


def test_capability_score_metal():
    # M3 Max: (48*4) + (48*0.5) + (8*0.25) + 15 = 192 + 24 + 2 + 15 = 233
    score = capability_score(vram_gib=48.0, ram_gib=48.0, cpu_cores=8, backend="metal")
    assert score == 233.0, f"expected 233.0, got {score}"

def test_gateway_package_importable():
    import swarm_gateway  # noqa: F401


def test_router_importable():
    from swarm_gateway.router import SwarmRouter  # noqa: F401


def test_litellm_min_version_constant():
    from swarm_gateway.router import LITELLM_MIN_VERSION
    major, minor = [int(x) for x in LITELLM_MIN_VERSION.split(".")[:2]]
    assert (major, minor) >= (1, 82), f"LITELLM_MIN_VERSION {LITELLM_MIN_VERSION} < 1.82.0"

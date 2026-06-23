from dataclasses import dataclass


@dataclass
class SwarmProviderConfig:
    llama_server_url: str = "http://127.0.0.1:8080"
    timeout: float = 120.0
    model_name: str = "swarm/local"

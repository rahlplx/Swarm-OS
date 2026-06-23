from dataclasses import dataclass, field
from typing import TypedDict


class RequestLogEntry(TypedDict):
    """Record of a single completion request's token usage."""

    input_tokens: int
    output_tokens: int
    request_number: int


@dataclass
class TokenCounter:
    """Accumulates token usage across requests for billing/telemetry."""

    input_tokens: int = 0
    output_tokens: int = 0
    requests: int = 0
    _log: list[RequestLogEntry] = field(default_factory=list)

    def log_request(self, input_tokens: int, output_tokens: int) -> None:
        """Record a single request's token counts."""
        self.input_tokens += input_tokens
        self.output_tokens += output_tokens
        self.requests += 1
        self._log.append(
            {
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "request_number": self.requests,
            }
        )

    @property
    def total_tokens(self) -> int:
        """Total tokens consumed (input + output)."""
        return self.input_tokens + self.output_tokens

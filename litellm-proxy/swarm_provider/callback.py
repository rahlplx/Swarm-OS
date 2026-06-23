from dataclasses import dataclass, field


@dataclass
class TokenCounter:
    input_tokens: int = 0
    output_tokens: int = 0
    requests: int = 0
    _log: list[dict] = field(default_factory=list)

    def log_request(self, input_tokens: int, output_tokens: int) -> None:
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
        return self.input_tokens + self.output_tokens

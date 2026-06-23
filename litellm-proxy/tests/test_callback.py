from swarm_provider.callback import TokenCounter


def test_token_counter_initial_state():
    counter = TokenCounter()
    assert counter.input_tokens == 0
    assert counter.output_tokens == 0
    assert counter.requests == 0
    assert counter.total_tokens == 0


def test_token_counter_single_request():
    counter = TokenCounter()
    counter.log_request(input_tokens=10, output_tokens=20)
    assert counter.input_tokens == 10
    assert counter.output_tokens == 20
    assert counter.total_tokens == 30
    assert counter.requests == 1


def test_token_counter_multiple_requests():
    counter = TokenCounter()
    counter.log_request(10, 20)
    counter.log_request(15, 30)
    counter.log_request(5, 10)
    assert counter.input_tokens == 30
    assert counter.output_tokens == 60
    assert counter.total_tokens == 90
    assert counter.requests == 3
    assert len(counter._log) == 3

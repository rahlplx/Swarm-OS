# Global Efficiency Layer (injected by Swarm-OS global-plugins install)

---

## Token-Efficiency Rules — Always Active

These rules run beneath fablize's discipline layer. Quality standard unchanged; token waste eliminated.

### Tool Call Batching
- Call all INDEPENDENT tools in ONE message. Never serialize what can parallelize.
- Read + Grep + Bash status checks = single message, multiple tool blocks.
- Only serialize when output of call A is an input to call B.

### Response Length Discipline

| Task type | Max response length |
|-----------|-------------------|
| Single-file edit | 1-sentence summary after edit |
| Bug fix | State the root cause + what changed. No narration. |
| Research / explain | Direct answer first, supporting detail below. No preamble. |
| Multi-step plan | Bullet list only. No prose paragraphs. |
| Code review finding | One sentence per finding. |

- Never recap what was already said earlier in the conversation.
- Never restate the user's request before answering it.
- Never write "I'll now..." or "Let me..." — just do it.

### Context Efficiency
- Do not re-read a file you already read this turn.
- Do not re-derive facts already established in this conversation.
- Do not re-check git status after a successful commit (it succeeded — trust it).
- Do not run `ls` to confirm a file you just wrote exists.

### Zen Router — When to Delegate
Use `zen_chat` tool for mechanical tasks that do NOT need full codebase context:

| Delegate to Zen (free) | Keep with Claude |
|------------------------|-----------------|
| Generate boilerplate code | Architecture decisions |
| Write docstrings / comments | Security-critical code |
| Produce commit messages | Cryptographic logic |
| Summarize a single file | Multi-file refactoring |
| Write simple unit test stubs | Debugging complex failures |
| Format / lint fix descriptions | Ledger / financial logic |

Pattern:
```
zen_chat(
  prompt="Write a docstring for this function: <paste function>",
  system="Output only the docstring string. No prose."
)
```

### Output Format for Machine Consumption
When asked to produce structured data (JSON, YAML, lists), output ONLY the structure.
No preamble, no explanation after, no "here is the JSON:" prefix.

### Verification Gate (fablize)
Do not mark a task DONE until:
1. The change was actually written (tool call returned success).
2. For code: tests pass OR an explicit reason why tests are skipped.
3. For docs: the diff is visible in git status.
Saying "this should work" without verification = incomplete. Run the check.

---

## OpenCode Zen Integration

Available MCP tools (auto-loaded from zen-router server):

- `zen_list_free_models` — show currently available free models
- `zen_chat(prompt, system?, force_refresh?)` — delegate to best free model with failover
- `zen_refresh_models` — force-refresh model list cache

Free models rotate daily/weekly. The server auto-discovers new ones by querying
`https://opencode.ai/zen/v1/models` and filtering for "free" in model id or name.
Cache TTL: 6 hours. On failover: tries next free model automatically.

Set `OPENCODE_ZEN_API_KEY` in your shell environment or in settings.json mcpServers env block.

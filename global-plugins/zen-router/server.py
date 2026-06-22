#!/usr/bin/env python3
"""
OpenCode Zen Free-Model Router — MCP Server
Discovers free models from OpenCode Zen's /models endpoint,
routes prompts with automatic failover when a model is unavailable.
Model list is cached locally with a 6-hour TTL and refreshed on startup.
"""

import asyncio
import json
import os
import sys
import time
from pathlib import Path
from typing import Any

import httpx
import mcp.server.stdio
import mcp.types as types
from mcp.server import Server

# ── Constants ─────────────────────────────────────────────────────────────────

ZEN_BASE = "https://opencode.ai/zen/v1"
CACHE_PATH = Path.home() / ".cache" / "swarm-os" / "zen-free-models.json"
CACHE_TTL = 6 * 3600      # 6 hours
MAX_TOKENS = 4096
HTTP_TIMEOUT = 60.0

# ── Cache helpers ──────────────────────────────────────────────────────────────

def _load_cache() -> dict | None:
    try:
        if CACHE_PATH.exists():
            data = json.loads(CACHE_PATH.read_text())
            if time.time() - data.get("ts", 0) < CACHE_TTL:
                return data
    except Exception:
        pass
    return None

def _save_cache(models: list[str]) -> None:
    CACHE_PATH.parent.mkdir(parents=True, exist_ok=True)
    CACHE_PATH.write_text(json.dumps({"ts": time.time(), "models": models}))

# ── Model discovery ────────────────────────────────────────────────────────────

async def _fetch_free_models(api_key: str, force: bool = False) -> list[str]:
    if not force:
        cached = _load_cache()
        if cached:
            return cached["models"]

    async with httpx.AsyncClient() as client:
        resp = await client.get(
            f"{ZEN_BASE}/models",
            headers={"Authorization": f"Bearer {api_key}"},
            timeout=15.0,
        )
        resp.raise_for_status()
        data = resp.json().get("data", [])

    # A model is "free" if "free" appears anywhere in its id or display name
    free: list[str] = []
    for m in data:
        mid = m.get("id", "")
        mname = m.get("name", mid)
        if "free" in mid.lower() or "free" in mname.lower():
            free.append(mid)

    _save_cache(free)
    return free

# ── Chat with failover ─────────────────────────────────────────────────────────

async def _chat(
    prompt: str,
    api_key: str,
    system: str = "",
    force_refresh: bool = False,
) -> dict[str, Any]:
    try:
        models = await _fetch_free_models(api_key, force=force_refresh)
    except Exception as exc:
        return {"error": f"Failed to fetch free models: {exc}"}

    if not models:
        return {"error": "No free models available — check your API key or try again later."}

    messages: list[dict] = []
    if system:
        messages.append({"role": "system", "content": system})
    messages.append({"role": "user", "content": prompt})

    errors: list[str] = []
    async with httpx.AsyncClient() as client:
        for model in models:
            try:
                resp = await client.post(
                    f"{ZEN_BASE}/chat/completions",
                    headers={
                        "Authorization": f"Bearer {api_key}",
                        "Content-Type": "application/json",
                    },
                    json={"model": model, "messages": messages, "max_tokens": MAX_TOKENS},
                    timeout=HTTP_TIMEOUT,
                )
                resp.raise_for_status()
                body = resp.json()
                try:
                    content = body["choices"][0]["message"]["content"]
                except (KeyError, IndexError, TypeError) as exc:
                    errors.append(f"{model}: Malformed response body ({type(exc).__name__})")
                    continue
                usage = body.get("usage", {})
                return {"model": model, "content": content, "usage": usage}
            except httpx.HTTPStatusError as exc:
                code = exc.response.status_code
                errors.append(f"{model}: HTTP {code}")
                # 429 = rate limited; 503/502 = model down — both warrant trying next
                if code not in (429, 502, 503, 504):
                    break  # hard failure (400 bad request, 401 auth) — stop immediately
            except (httpx.TimeoutException, httpx.ConnectError) as exc:
                errors.append(f"{model}: {type(exc).__name__}")

    return {"error": f"All free models failed: {'; '.join(errors)}", "tried": models}

# ── MCP server ─────────────────────────────────────────────────────────────────

server = Server("zen-router")

@server.list_tools()
async def list_tools() -> list[types.Tool]:
    return [
        types.Tool(
            name="zen_list_free_models",
            description=(
                "Show which OpenCode Zen models are currently free. "
                "Result is cached (6h TTL). Use zen_refresh_models to force an update."
            ),
            inputSchema={"type": "object", "properties": {}, "required": []},
        ),
        types.Tool(
            name="zen_chat",
            description=(
                "Delegate a task to OpenCode Zen's free models. "
                "USE FOR: code generation, refactoring, test scaffolding, docstrings, "
                "commit messages, simple transformations, summarisation. "
                "DO NOT USE FOR: architectural decisions, security-critical code, "
                "tasks requiring full codebase context, cryptographic logic. "
                "Automatically picks the fastest available free model and fails over "
                "to the next if unavailable."
            ),
            inputSchema={
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "The full task prompt for the free model.",
                    },
                    "system": {
                        "type": "string",
                        "description": "Optional system prompt (role, constraints, output format).",
                    },
                    "force_refresh": {
                        "type": "boolean",
                        "description": "Force refresh the free-model cache before sending.",
                        "default": False,
                    },
                },
                "required": ["prompt"],
            },
        ),
        types.Tool(
            name="zen_refresh_models",
            description=(
                "Force-refresh the free model list from OpenCode Zen. "
                "Run this if zen_chat reports no models available."
            ),
            inputSchema={"type": "object", "properties": {}, "required": []},
        ),
    ]


@server.call_tool()
async def call_tool(name: str, arguments: dict) -> list[types.TextContent]:
    arguments = arguments or {}
    api_key = os.environ.get("OPENCODE_ZEN_API_KEY", "").strip().strip("'\"")

    if name == "zen_list_free_models":
        cached = _load_cache()
        if cached:
            age_min = int((time.time() - cached["ts"]) / 60)
            lines = "\n".join(f"  • {m}" for m in cached["models"])
            return [types.TextContent(
                type="text",
                text=f"Free models (cached {age_min}m ago, refreshes every 6h):\n{lines}",
            )]
        if not api_key:
            return [types.TextContent(
                type="text",
                text="OPENCODE_ZEN_API_KEY not set — cannot fetch model list.",
            )]
        models = await _fetch_free_models(api_key)
        lines = "\n".join(f"  • {m}" for m in models) or "  (none found)"
        return [types.TextContent(type="text", text=f"Free models:\n{lines}")]

    if name == "zen_refresh_models":
        if not api_key:
            return [types.TextContent(type="text", text="OPENCODE_ZEN_API_KEY not set.")]
        if CACHE_PATH.exists():
            CACHE_PATH.unlink()
        models = await _fetch_free_models(api_key, force=True)
        lines = "\n".join(f"  • {m}" for m in models) or "  (none found)"
        return [types.TextContent(type="text", text=f"Refreshed — {len(models)} free models:\n{lines}")]

    if name == "zen_chat":
        if not api_key:
            return [types.TextContent(
                type="text",
                text="OPENCODE_ZEN_API_KEY not set. Add it to your environment or settings.json env block.",
            )]
        result = await _chat(
            prompt=arguments["prompt"],
            api_key=api_key,
            system=arguments.get("system", ""),
            force_refresh=arguments.get("force_refresh", False),
        )
        if "error" in result:
            return [types.TextContent(type="text", text=f"zen_chat error: {result['error']}")]

        usage = result.get("usage", {})
        footer = (
            f"\n\n---\n_Routed via **{result['model']}** "
            f"(in={usage.get('prompt_tokens','?')} out={usage.get('completion_tokens','?')} tokens)_"
        )
        return [types.TextContent(type="text", text=result["content"] + footer)]

    return [types.TextContent(type="text", text=f"Unknown tool: {name}")]


async def main() -> None:
    async with mcp.server.stdio.stdio_server() as (read, write):
        await server.run(read, write, server.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())

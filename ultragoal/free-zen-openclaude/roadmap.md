# Roadmap: free-zen-openclaude

## Overall Goal

Convert the pi-opencode-zen pattern into a standalone local HTTP proxy that:
- Injects opencode-style headers into upstream requests
- Filters `/v1/models` responses to show only free models
- Streams chat completions with SSE
- Requires no API key, no config file modification, no openclaude dependency
- Runs as a pure daemon, compatible with any OpenAI client

## Constraints

| # | Constraint | Implementation |
|---|-----------|----------------|
| C1 | Dynamic fetch every run | `fetch_free_models()` fetches models.dev each invocation |
| C2 | Only free models | `cost.input === 0`, `status !== "deprecated"` filter |
| C3 | No manual env var setup | `--daemon` writes env file; `source /tmp/free-zen-*.env` |
| C4 | Minimal configuration | Default port=random, single `--daemon` flag |
| C5 | No OPENCODE_API_KEY | Empty `OPENAI_API_KEY` — zen accepts free models without auth |
| C6 | No .env file | Temp files only (`/tmp/free-zen-*.env`), never persistent |
| C7 | Best language | Python stdlib only (`http.server`, `urllib`, `os`, `json`) |
| C8 | No openclaude dependency | Proxy is standalone, works with any OpenAI-compatible client |
| C9 | Same HTTP headers as opencode | User-Agent + 4 x-opencode-* UUIDs per request |
| C10 | No config file modification | Env vars and temp files only |
| C11 | Install openclaude and test | Verified E2E — both foreground and daemon modes |

## All Stages (Completed)

### Stage 1: Implement `free-zen.py` as local HTTP proxy

| Aspect | Detail |
|--------|--------|
| **Status** | ✅ Complete |
| **Objective** | Python HTTP proxy that intercepts `/v1/models` and `/chat/completions` |
| **Output** | `free-zen.py` (313 lines) + `test_free_zen.py` (335 lines, 24 tests) |

### Stage 2: Install openclaude

| Aspect | Detail |
|--------|--------|
| **Status** | ✅ Complete |
| **Objective** | Install `@gitlawb/openclaude` for E2E testing |

### Stage 3: Test with free model

| Aspect | Detail |
|--------|--------|
| **Status** | ✅ Complete |
| **Objective** | Run openclaude through proxy, verify response |
| **Evidence** | `openclaude --print "Reply with exactly: PASS"` → "PASS" |
| **Evidence** | `api/models` returns 6 free models (not 55 total) |

## Model Details

| # | Model ID | Notes |
|---|----------|-------|
| 1 | `deepseek-v4-flash-free` | Best for coding (default) |
| 2 | `big-pickle` | Generic free |
| 3 | `hy3-free` | Generic free |
| 4 | `mimo-v2.5-free` | Xiaomi MiMo free |
| 5 | `nemotron-3-ultra-free` | NVIDIA Nemotron free |
| 6 | `north-mini-code-free` | North AI free |

## Usage

```bash
# Foreground (Ctrl+C to stop)
python3 free-zen.py --port 8765

# Daemon mode (background, write env to /tmp)
eval "$(python3 free-zen.py --daemon --port 8765 2>/dev/null)"
openclaude

# Or source the env file
source /tmp/free-zen-8765.env
openclaude
```

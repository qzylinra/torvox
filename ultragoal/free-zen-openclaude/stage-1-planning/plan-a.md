# Plan A: `free-zen.py` Design

## Overview

A single-file Python 3.12+ stdlib-only script that dynamically discovers free models from `models.dev`, generates openclaude-compatible `export` statements, and requires zero configuration, zero API keys, and zero file mutations.

---

## 1. Architecture

| Aspect | Decision |
|--------|----------|
| File | Single file: `free-zen.py` at repository root |
| Shebang | `#!/usr/bin/env python3` |
| Classes | None — top-level functions only |
| Entry | `if __name__ == "__main__": main()` |
| Imports | `urllib.request`, `json`, `uuid`, `sys`, `os`, `argparse`, `time` — all stdlib, zero pip deps |

---

## 2. Function Map

```
generate_opencode_headers()  → dict   Per-request UUID headers (session, project, request)
fetch_json(url, timeout, headers) → dict   HTTP GET + json.loads wrapper
fetch_models_dev_info()       → dict   models.dev/api.json → opencode.models.*
probe_zen_models()            → list|None   Optional GET /v1/models verification
filter_free_models(raw)       → dict    cost.input===0 AND status!=="deprecated"
select_best_model(free)       → str     Priority-pick from filtered free models
format_env_exports(model)     → str     Shell export statements
main()                        → int     argparser → orchestrate → print or list or json
```

### 2a. `generate_opencode_headers()` — Header Simulation (C9)

```python
import uuid

def generate_opencode_headers():
    """Mimic pi-opencode-zen per-request UUID headers."""
    def uuid26():
        return uuid.uuid4().hex[:26]  # strip dashes, truncate to 26 chars
    return {
        "User-Agent": "opencode/latest/1.3.15/cli",
        "x-opencode-client": "cli",
        "x-opencode-session": uuid26(),
        "x-opencode-project": uuid26(),
        "x-opencode-request": uuid26(),
    }
```

Used only for script's own HTTP probes (models.dev, zen /v1/models). NOT injected into openclaude's traffic — openclaude sends its own headers.

### 2b. `fetch_json()` — HTTP Fetch (C1)

```python
def fetch_json(url, timeout=30, headers=None):
    req = urllib.request.Request(url, headers=headers or {}, method="GET")
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read())  # 3MB is fine for stdlib json
```

- `urllib.Request` handles gzip/deflate transparently
- `timeout` param prevents hangs (default 30s)
- Returns parsed dict or raises on error

### 2c. `fetch_models_dev_info()` — Source of Truth (C1, C2)

Fetches `https://models.dev/api.json`. No auth needed. Uses `generate_opencode_headers()` for the User-Agent but anonymous otherwise.

Returns the raw `opencode.models` dict from the response, or raises.

### 2d. `probe_zen_models()` — Optional Verification

Optional flag `--probe`. Fetches `https://opencode.ai/zen/v1/models` with empty `Authorization: Bearer ` header + `generate_opencode_headers()`. 

Purpose: verify the zen endpoint is reachable and confirm the expected model IDs are present. If the endpoint is down, this produces a warning on stderr but does NOT block env var generation (the script already has the model list from models.dev).

Returns list of model IDs from the endpoint, or `None` on failure.

### 2e. `filter_free_models()` — Pricing Filter (C2)

```python
def filter_free_models(models_dev):
    free = {}
    for model_id, info in models_dev.items():
        if info.get("status") == "deprecated":
            continue
        cost = info.get("cost") or {}
        if cost.get("input", 0) != 0:
            continue
        free[model_id] = info
    return free
```

Logic:
1. Skip if `status == "deprecated"`
2. Skip if `cost` is absent or `cost.input` is absent/non-zero
3. Keep otherwise (zero input cost, not deprecated)

### 2f. `select_best_model()` — Priority Selection

```python
PRIORITY = [
    "deepseek-v4-flash-free",    # Best coding capability
    "nemotron-3-ultra-free",     # NVIDIA, solid general
    "mimo-v2.5-free",            # Xiaomi MiMo
    "north-mini-code-free",      # North AI coding
    "big-pickle",                # Generic free
    "hy3-free",                  # Generic free
]

def select_best_model(free_models):
    for model_id in PRIORITY:
        if model_id in free_models:
            return model_id
    # Fallback: first alphabetical
    return sorted(free_models.keys())[0]
```

### 2g. `format_env_exports()` — Output Generation (C3, C4, C5, C6, C10)

```python
def format_env_exports(model_id):
    return "\n".join([
        f"export CLAUDE_CODE_USE_OPENAI=1",
        f"export OPENAI_BASE_URL=https://opencode.ai/zen/v1",
        f"export OPENAI_MODEL={model_id}",
        "",
        f"# Free model: {model_id}",
        f"# Usage: eval $({__file__}) && openclaude",
    ])
```

Key design decisions:
- `OPENAI_API_KEY` is intentionally **omitted** — zen accepts free model requests with empty `Authorization: Bearer `, and openclaude sends `Bearer ${OPENAI_API_KEY}` which defaults to empty string when the env var is absent. Setting it explicitly to empty string could cause issues with some .env parsers. Omitting it is safer (it's already unset by default).
- `CLAUDE_CODE_USE_OPENAI=1` routes through the OpenAI-compatible shim
- `OPENAI_BASE_URL=https://opencode.ai/zen/v1` points at the zen gateway
- `OPENAI_MODEL=<model_id>` selects the best free model

### 2h. `main()` — Orchestrator

```python
def main():
    parser = argparse.ArgumentParser(description="...")
    parser.add_argument("--list", action="store_true", help="List all free models, one per line")
    parser.add_argument("--json", action="store_true", help="Output full model info as JSON")
    parser.add_argument("--probe", action="store_true", help="Also probe zen /v1/models for verification")
    parser.add_argument("--timeout", type=int, default=30, help="HTTP timeout (seconds)")
    args = parser.parse_args()

    headers = generate_opencode_headers()
    models_dev = fetch_models_dev_info(headers, args.timeout)
    free_models = filter_free_models(models_dev)

    if not free_models:
        print("error: no free models found", file=sys.stderr)
        return 1

    if args.list:
        for model_id in sorted(free_models):
            print(model_id)
        return 0

    if args.json:
        print(json.dumps({
            "free_models": list(free_models.keys()),
            "selected": select_best_model(free_models),
            "env": {
                "CLAUDE_CODE_USE_OPENAI": "1",
                "OPENAI_BASE_URL": "https://opencode.ai/zen/v1",
                "OPENAI_MODEL": select_best_model(free_models),
            }
        }, indent=2))
        return 0

    if args.probe:
        zen_models = probe_zen_models(headers, args.timeout)
        if zen_models is not None:
            selected = select_best_model(free_models)
            if selected not in zen_models:
                print(f"warning: selected model '{selected}' not in zen /v1/models", file=sys.stderr)

    model_id = select_best_model(free_models)
    print(format_env_exports(model_id))
    return 0
```

---

## 3. Error Handling

| Scenario | Behavior |
|----------|----------|
| models.dev unreachable | Print error to stderr, exit 1 |
| models.dev returns non-JSON | Print JSON decode error to stderr, exit 1 |
| models.dev malformed structure | KeyError with message to stderr, exit 1 |
| No free models found | Print message to stderr, exit 1 |
| zen /v1/models unreachable (--probe) | Print warning to stderr, continue |
| Network timeout | `urllib.error.URLError` caught, message to stderr, exit 1 |
| DNS failure | `urllib.error.URLError` caught, message to stderr, exit 1 |

---

## 4. CLI Interface

```text
usage: free-zen.py [-h] [--list] [--json] [--probe] [--timeout TIMEOUT]

Dynamically discover free models from opencode.ai zen gateway and
generate shell configuration for openclaude.

No API key needed — free models work with empty auth at the zen endpoint.

options:
  -h, --help         show this help message and exit
  --list             List free models (one per line) and exit
  --json             Output free model info as JSON and exit
  --probe            Also probe https://opencode.ai/zen/v1/models for verification
  --timeout TIMEOUT  HTTP timeout in seconds (default: 30)
```

---

## 5. Usage Examples

```bash
# Default: print shell export statements for eval
eval $(python free-zen.py) && openclaude --print "Say hello"

# List available free models
python free-zen.py --list

# View full info as JSON
python free-zen.py --json

# With optional zen endpoint probe
eval $(python free-zen.py --probe) && openclaude --print "Write a poem"

# Longer timeout for slow networks
eval $(python free-zen.py --timeout 60) && openclaude
```

---

## 6. Constraint Compliance Matrix

| # | Constraint | How Plan A Satisfies It |
|---|-----------|------------------------|
| C1 | Dynamic fetch every run | `fetch_models_dev_info()` called every invocation |
| C2 | Only free models | `filter_free_models()` checks cost.input===0, status!=="deprecated" |
| C3 | No manual env var setup | Script prints ready-to-`eval` export statements |
| C4 | Minimal config | No config files, no profiles, no settings |
| C5 | No OPENCODE_API_KEY | `OPENAI_API_KEY` is omitted — zen works with empty auth |
| C6 | No .env file | Only stdout output, never writes to filesystem |
| C7 | Python stdlib only | `urllib`, `json`, `uuid`, `sys`, `os`, `argparse`, `time` |
| C8 | No wrapping/proxy | Script only prints env vars — user invokes openclaude directly |
| C9 | Header simulation | `generate_opencode_headers()` for probe requests |
| C10 | No config file modify | Never touches `~/.openclaude.json`, `~/.openclaude-profile.json` |
| C11 | Must work with openclaude | Output maps to openclaude's documented `CLAUDE_CODE_USE_OPENAI=1` path |

---

## 7. Non-Goals (explicitly out of scope)

- Watching/restarting openclaude on model changes
- Profiling multiple models for benchmark comparison
- Caching models.dev results to disk (C1 forbids hardcoded lists)
- Setting `OPENAI_API_KEY` or any auth-related env vars
- Creating shell aliases or modifying shell rc files
- Automatic model fallback if the selected model fails at runtime

---

## 8. Files Modified by This Stage

| File | Action |
|------|--------|
| `free-zen.py` | **Create** — the script itself |

No other files are created or modified.

---

## 9. Acceptance Criteria

1. `python free-zen.py --list` exits 0 and prints ≥1 model ID (currently 6)
2. `python free-zen.py` exits 0 and prints valid shell exports containing `CLAUDE_CODE_USE_OPENAI=1`, `OPENAI_BASE_URL=`, `OPENAI_MODEL=`
3. `python free-zen.py --json` exits 0 and prints valid JSON with `free_models` array of ≥1 model
4. `eval "$(python free-zen.py)" && echo "$OPENAI_MODEL"` prints a non-empty model name
5. `python free-zen.py --probe` exits 0 without error (or produces a warning if zen is down)
6. Running without network: `python free-zen.py` exits 1 with error message on stderr
7. All functions documented with docstrings explaining parameters and return types

---

## 10. Implementation Order

1. Write `generate_opencode_headers()` → test UUID generation matches `[a-f0-9]{26}`
2. Write `fetch_json()` → test with models.dev
3. Write `fetch_models_dev_info()` → parse real response
4. Write `filter_free_models()` → verify returns exactly 6 models on live data
5. Write `select_best_model()` → verify returns `deepseek-v4-flash-free`
6. Write `format_env_exports()` → verify output format
7. Write `probe_zen_models()` → optional
8. Write `main()` with argparse → wire everything together
9. Verify all acceptance criteria pass

---

## 11. Test Data

When offline, the script can be tested against a local copy of models.dev by setting:

```bash
export FREE_ZEN_MODELS_DEV_URL="file:///tmp/models.dev.api.json"
```

But this is a development convenience only — in production the script always fetches from the real URL (C1).

# Plan B: `free-zen.py` — Implementation Plan

## 1. Architecture Decision: Path A (env vars via `eval`)

**Decision**: Generate shell `export` statements for the user to `eval`.

| Path | Approach | Verdict |
|------|----------|---------|
| A (env vars) | `CLAUDE_CODE_USE_OPENAI=1`, `OPENAI_BASE_URL`, `OPENAI_MODEL=<free>` | ✅ **Chosen** — zero files, zero config changes, works with C10 |
| B (temp config) | Write `.openclaude-profile.json` or `--provider-env-file` | ❌ C10 prohibits config file modifications; temp file is still a file modification |
| C (proxy) | Intercept `/v1/models` to filter model list | ❌ C8 prohibits wrapping/proxy |

**Why Path A is sufficient**: The goal is "only free models" in the sense that the script *configures* a free model by default and *never* requires an API key or payment. OpenClaude's model picker (`/model`) still lists all models discovered from `/v1/models` on the zen gateway, but:
- The default model is always free
- The user must *explicitly choose* a paid model — they won't accidentally incur costs
- The zen gateway enforces auth for paid models anyway (request will fail without a valid key)

This matches the practical UX of pi-opencode-zen's "public mode": you get free models by default, with the ability to add a key later.

### Why not restrict model discovery

OpenClaude's OpenAI shim (`openaiShim.ts`) fetches `GET /v1/models` from `OPENAI_BASE_URL` to build the model picker list. There is no env var to filter the response. To restrict the list we would need either a proxy between openclaude and the zen gateway (C8 violation) or a code change to openclaude itself (out of scope).

---

## 2. Script Design

### Entry Point

```
usage: free-zen.py [-h] [--launch] [--list] [--json] [--launch-cmd CMD]

Generate openclaude env vars for free OpenCode Zen models.

options:
  -h, --help            show this help message and exit
  --launch              Export env vars and exec openclaude
  --list                Print human-readable free model table
  --json                Print free model list as JSON array
  --launch-cmd CMD      Command to launch instead of "openclaude"
  --no-color            Disable ANSI color in --list output
  --timeout SECONDS     HTTP timeout for models.dev fetch (default: 10)
```

### Return Codes

| Code | Meaning |
|------|---------|
| 0 | Success (env vars printed, or launch succeeded) |
| 1 | Network/parse/no-models error |
| 2 | `--launch` mode but `openclaude` not found |

### Functions

```python
# --- ID generation ---
def create_id() -> str:
    """Generate a 32-char lowercase hex string (UUID4 without dashes)."""
    # uuid.uuid4().hex[:26] in pie — use uuid module

# --- Header simulation (C9) ---
def opencode_headers() -> dict[str, str]:
    """Build the 5 opencode-style headers for models.dev fetch."""

# --- Networking ---
def fetch_json(url: str, headers: dict | None = None, timeout: int = 10) -> dict:
    """GET request → JSON parse. Raises on HTTP error or parse failure.
       Uses urllib.request with opencode_headers()."""

# --- Model filtering (C2) ---
def find_free_models(raw: dict) -> list[dict]:
    """Filter models.dev JSON to only free, non-deprecated models."""

def pick_best_default(models: list[dict]) -> str | None:
    """Pick the best default model: prefer deepseek, then nemotron, then first."""

# --- Output generation ---
def generate_shell_exports(models: list[dict], default_id: str) -> str:
    """Return a shell-safe 'export KEY=VAL\\n' string."""

def print_model_table(models: list[dict], default_id: str, color: bool = True) -> None:
    """Pretty-print a table of free models to stdout (or stderr?)."""
    # Uses stderr so stdout stays clean for eval

# --- Launch ---
def launch_openclaude(env: dict, cmd: str = "openclaude") -> NoReturn:
    """Set env vars, exec openclaude."""

# --- Main ---
def main(argv: list[str]) -> int:
    """Parse args, fetch, filter, output."""
```

### Flow Diagram

```
main()
  ├─ parse args
  ├─ fetch_json("https://models.dev/api.json", headers=opencode_headers())
  ├─ find_free_models(response)
  │   └─ models["opencode"]["models"].items()
  │       ├─ skip if status == "deprecated"
  │       └─ keep if cost.input == 0
  ├─ if empty → exit 1
  ├─ pick_best_default(models)
  │
  ├─ branch: --json → json.dumps(models) → stdout → exit 0
  ├─ branch: --list → print_model_table() → stderr → exit 0
  ├─ branch: --launch → launch_openclaude(env=exports, cmd=launch_cmd)
  │                      └─ os.execvpe(cmd, [cmd], env)
  └─ default:
       ├─ print(generate_shell_exports(models, default))  # stdout
       └─ print("eval "$(<script>)" to apply", file=stderr)  # stderr
       └─ exit 0
```

---

## 3. models.dev Fetching/parsing

### Request

```python
req = urllib.request.Request(
    "https://models.dev/api.json",
    headers=opencode_headers(),
    method="GET",
)
with urllib.request.urlopen(req, timeout=10) as resp:
    data = json.loads(resp.read())
```

### Why `urllib.request` is preferred over alternatives

| Module | Stdlib? | Why not |
|--------|---------|---------|
| `urllib.request` | ✅ | Used — simple GET with custom headers and timeout |
| `http.client` | ✅ | Lower-level, more boilerplate for same result |
| `urllib.request` + `json` | ✅ | Minimal, idiomatic Python stdlib |

### Parsing

The JSON response from models.dev is typically < 500 KB. `json.loads` handles this in < 50ms. No streaming needed.

### Error resilience

```python
try:
    data = fetch_json(MODELS_DEV_URL, headers=opencode_headers(), timeout=args.timeout)
except (urllib.error.URLError, urllib.error.HTTPError) as e:
    print(f"free-zen: Network error fetching models.dev: {e}", file=sys.stderr)
    return 1
except json.JSONDecodeError as e:
    print(f"free-zen: Invalid JSON from models.dev: {e}", file=sys.stderr)
    return 1
```

No fallback hardcoded list (C1: dynamic fetch every run). If the network is down, the script exits with an error.

---

## 4. Model Filtering

### Algorithm

```python
def find_free_models(data: dict) -> list[dict]:
    models = data.get("opencode", {}).get("models", {})
    if not isinstance(models, dict):
        raise ValueError("models.dev: 'opencode.models' is not an object")

    free = []
    for model_id, info in models.items():
        if not isinstance(info, dict):
            continue  # skip malformed entries

        # Filter deprecated
        if info.get("status") == "deprecated":
            continue

        # Filter by cost - only free if cost.input == 0
        cost = info.get("cost")
        if cost is None:
            continue  # no cost info → assume paid (safe default)
        if not isinstance(cost, dict):
            continue
        if cost.get("input") != 0:
            continue

        free.append({
            "id": model_id,
            "name": info.get("name", model_id),
            "description": info.get("description", ""),
            "context_window": info.get("limit", {}).get("context", 128_000),
            "max_output": info.get("limit", {}).get("output", 64_000),
            "reasoning": bool(info.get("reasoning", False)),
            "vision": "image" in (info.get("modalities", {}).get("input", [])),
        })

    free.sort(key=lambda m: m["name"].lower())
    return free
```

### Default model selection

```python
def pick_best_default(models: list[dict]) -> str | None:
    if not models:
        return None
    # Preference order: deepseek-v4-flash-free, nemotron-*, any other
    for prefix in ["deepseek-v4-flash-free", "deepseek", "nemotron"]:
        for m in models:
            if m["id"] == prefix or m["id"].startswith(prefix):
                return m["id"]
    return models[0]["id"]
```

---

## 5. OpenClaude Integration

### Environment Variables Generated

```bash
export CLAUDE_CODE_USE_OPENAI=1
export OPENAI_BASE_URL=https://opencode.ai/zen/v1
export OPENAI_API_KEY=
export OPENAI_MODEL=deepseek-v4-flash-free
export OPENAI_API_FORMAT=chat_completions
```

### Why these values

| Env var | Value | Rationale |
|---------|-------|-----------|
| `CLAUDE_CODE_USE_OPENAI=1` | Enable OpenAI-compatible shim | Required for non-Anthropic base URLs |
| `OPENAI_BASE_URL` | `https://opencode.ai/zen/v1` | Zen gateway — no `/v1` suffix needed; zen handles routing |
| `OPENAI_API_KEY` | `""` (empty) | C5: no API key needed. Zen accepts free model requests with empty auth |
| `OPENAI_MODEL` | Best free model ID | Ensures default session uses a free model |
| `OPENAI_API_FORMAT` | `chat_completions` | Zen free models use `/v1/chat/completions` endpoint |

### Invocation patterns

**Recommended: eval + launch**
```bash
eval "$(python free-zen.py --launch)"
# Launches openclaude directly with env vars set in-process
```

**Recommended: source then run**
```bash
eval "$(python free-zen.py)"
openclaude
```

**Pipe to file (if users prefer)**
```bash
python free-zen.py > /tmp/free-zen.env
source /tmp/free-zen.env && openclaude
```

---

## 6. Header Simulation

### What the script sends to models.dev

```python
def opencode_headers() -> dict[str, str]:
    uid = uuid.uuid4().hex  # 32-char lowercase hex
    return {
        "User-Agent": "opencode/latest/1.3.15/cli",
        "x-opencode-client": "cli",
        "x-opencode-session": uid[:26],
        "x-opencode-project": uid[26:52] if len(uid) >= 52 else uid[:26],
        "x-opencode-request": uuid.uuid4().hex[:26],
    }
```

Note: Each header call generates **two** UUIDs (one shared for session+project truncated at different offsets, one unique for request). This avoids wasting entropy while preserving the structural pattern.

### Why headers are NOT injected into openclaude requests

OpenClaude's `openaiShim.ts` (`makeOpenAIRequest`) does not expose a mechanism to inject arbitrary HTTP headers for OpenAI-compatible requests. The variables that exist are:

- `OPENAI_AUTH_HEADER` / `OPENAI_AUTH_HEADER_VALUE` / `OPENAI_AUTH_SCHEME` — auth-only
- `ANTHROPIC_CUSTOM_HEADERS` — Anthropic-native mode only, not OpenAI shim

To inject opencode headers into the actual API requests, we would need to either:
- Modify openclaude source (out of scope)
- Use a proxy (C8 violation)
- Use the `ANTHROPIC_CUSTOM_HEADERS` path for Anthropic-native models (doesn't apply to free models)

Since the headers are non-functional (the zen gateway does not require them), this is acceptable.

---

## 7. Output/Runtime Behavior

### Default mode (`free-zen.py`, no flags)

```
# stdout (for eval):
export CLAUDE_CODE_USE_OPENAI=1
export OPENAI_BASE_URL=https://opencode.ai/zen/v1
export OPENAI_API_KEY=
export OPENAI_MODEL=deepseek-v4-flash-free
export OPENAI_API_FORMAT=chat_completions

# stderr:
free-zen: found 6 free models from opencode.ai/zen
free-zen: default model: deepseek-v4-flash-free
free-zen: run `eval "$(free-zen.py)" && openclaude`
```

### `--list` mode

```
# stderr:
free-zen: free OpenCode Zen models (6):
  1. big-pickle                 (ctx: 200k, max: 128k)
  2. deepseek-v4-flash-free     (ctx: 128k, max: 64k) ← default
  3. hy3-free                   (ctx: 128k, max: 64k)
  4. mimo-v2.5-free             (ctx: 128k, max: 64k)
  5. nemotron-3-ultra-free       (ctx: 204k, max: 128k)
  6. north-mini-code-free        (ctx: 128k, max: 64k)
```

### `--json` mode

```json
# stdout:
[
  {"id": "big-pickle", "name": "Big Pickle", ...},
  {"id": "deepseek-v4-flash-free", "name": "DeepSeek V4 Flash Free", ...},
  ...
]
```

### `--launch` mode

Prints nothing to stdout/stderr (except errors). Sets env vars in the current process, then `os.execvpe("openclaude", ["openclaude"], env=exports)`.

---

## 8. Error Handling

| Failure mode | Detection | Behavior | Exit code |
|---|---|---|---|
| No network | `urllib.error.URLError` | Print error to stderr, exit 1 | 1 |
| HTTP error (4xx/5xx) | `urllib.error.HTTPError` | Print status + body snippet to stderr, exit 1 | 1 |
| Invalid JSON | `json.JSONDecodeError` | Print error to stderr, exit 1 | 1 |
| Missing `opencode.models` | KeyError / TypeError | Print error to stderr, exit 1 | 1 |
| No free models | `find_free_models` returns `[]` | Print "no free models found" to stderr, exit 1 | 1 |
| `--launch` but openclaude not on PATH | `os.execvpe` raises `FileNotFoundError` | Print "openclaude not found" to stderr, exit 2 | 2 |
| `--launch` and openclaude fails | N/A (exec replaces process) | OpenClaude handles its own errors | N/A |
| Invalid CLI args | Argparse | Print help to stderr, exit 2 | 2 |

All error output goes to stderr, preserving stdout for `eval`.

---

## 9. Testing Plan

### 9.1 Unit tests (stdlib `unittest` + `unittest.mock`)

Create `test_free_zen.py` alongside the script:

| Test | What it verifies |
|---|---|
| `test_opencode_headers_structure` | Returns dict with 5 expected keys |
| `test_opencode_headers_format` | User-Agent matches `opencode/latest/*/cli`, x-opencode-* values are 26-char hex strings |
| `test_opencode_headers_uniqueness` | Two calls produce different session/project/request values |
| `test_find_free_models_empty` | Empty input returns `[]` |
| `test_find_free_models_deprecated` | Deprecated models are excluded |
| `test_find_free_models_paid` | Models with cost.input != 0 are excluded |
| `test_find_free_models_no_cost` | Models with no cost field are excluded |
| `test_find_free_models_free` | Models with cost.input == 0 are included |
| `test_find_free_models_mixed` | Mixed input correctly filters |
| `test_pick_best_default_deepseek` | deepseek-v4-flash-free is preferred |
| `test_pick_best_default_nemotron` | Falls back to nemotron if no deepseek |
| `test_pick_best_default_fallback` | Falls back to first model |
| `test_pick_best_default_empty` | Returns None for empty list |
| `test_generate_shell_exports` | Produces valid `KEY=VAL` (or `KEY=""`) for each expected var |
| `test_fetch_json_success` | Mocked urllib → returns parsed JSON |
| `test_fetch_json_http_error` | Mocked HTTPError → raises |
| `test_fetch_json_parse_error` | Mocked invalid body → raises |

### 9.2 Integration test (manual / scripted)

```bash
# 1. Verify script runs and outputs env vars
python free-zen.py > /tmp/free-zen-test.env
grep -q CLAUDE_CODE_USE_OPENAI=1 /tmp/free-zen-test.env
grep -q OPENAI_BASE_URL=https://opencode.ai/zen/v1 /tmp/free-zen-test.env
grep -q OPENAI_API_KEY= /tmp/free-zen-test.env
grep -q OPENAI_MODEL= /tmp/free-zen-test.env
grep -q OPENAI_API_FORMAT=chat_completions /tmp/free-zen-test.env
echo "PASS: env vars generated"

# 2. Verify --list mode (stderr contains table, stdout is empty)
python free-zen.py --list 2>/tmp/free-zen-list.txt
test "$(wc -l < /tmp/free-zen-list.txt)" -gt 3
echo "PASS: --list mode"

# 3. Verify --json mode (stdout is valid JSON array)
python free-zen.py --json | python -c "import sys,json; d=json.load(sys.stdin); assert isinstance(d, list) and len(d) > 0"
echo "PASS: --json mode"

# 4. Verify env vars take effect
eval "$(python free-zen.py)"
echo "$OPENAI_MODEL"
```

### 9.3 End-to-end (Stage 3 on roadmap)

```bash
# With openclaude installed:
eval "$(python free-zen.py)" && openclaude -p "Say hello in one word" --print
# Expected: free model responds (no API key, no payment)
```

---

## 10. File List

| File | Purpose |
|------|---------|
| `free-zen.py` | Main script (single file, ~250 lines) |
| `test_free_zen.py` | Unit tests (~200 lines) |
| `README.md` | One-page usage guide (only if requested) |
| `docs/` | Not needed — script is self-documenting via `--help` |

---

## 11. Implementation Order

1. Write `free_zen.py` skeleton: arg parsing, `main()`, `opencode_headers()`
2. Implement `fetch_json()` with error handling
3. Implement `find_free_models()` + `pick_best_default()`
4. Implement `generate_shell_exports()` — the core output
5. Implement `--list` and `--json` display modes
6. Implement `--launch` mode
7. Write `test_free_zen.py` — all unit tests
8. Run integration tests manually against live models.dev
9. Run E2E with openclaude (see roadmap Stage 3)
10. Polish: help text, error messages, edge cases

---

## 12. Edge Cases & Defensive Design

| Edge case | Handling |
|-----------|----------|
| models.dev returns models with `cost.input: 0` but `cost.output: > 0` | Still treated as free (matches pi-opencode-zen behavior) |
| models.dev has `cost: null` per model | Excluded (no cost info = assume paid) |
| models.dev structure changes | Script accesses `.get()` safely; exits with clear error on missing keys |
| models.dev is slow (>10s) | Timeout configurable via `--timeout`, default 10s |
| `OPENAI_API_FORMAT` changes per model | Zen free models all use `chat_completions`; single value is correct |
| User has existing `OPENAI_*` env vars | `eval` overrides them (expected behavior) |
| Multiple Python versions | 3.8+ compatible (no 3.10+ features used) |
| Windows | Script targets Linux (project context); Windows paths not needed |

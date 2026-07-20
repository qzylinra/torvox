# Integrated Plan: `free-zen.py`

## Overview

Single-file Python 3.8+ stdlib-only script that dynamically discovers free models from `models.dev`, generates openclaude-compatible shell environment variables, and optionally launches openclaude. Zero configuration, zero API keys, zero file mutations.

Derived from Plan A (features: `--probe`, 3 independent UUIDs, `FREE_ZEN_MODELS_DEV_URL`, usage comment, cost/status filtering) and Plan B (architecture: `--launch`/`--list`/`--json`/`--no-color`/`--launch-cmd`, 10s timeout, prefix-based matching, exit code 2, `free-zen:` prefix, rich metadata, test plan, Python 3.8+). Conflicts resolved by [planning-decision.md](planning-decision.md).

---

## 1. Architecture

| Aspect | Decision |
|--------|----------|
| File | Single file: `free-zen.py` at repository root |
| Shebang | `#!/usr/bin/env python3` |
| Classes | None — top-level functions only |
| Entry | `if __name__ == "__main__": sys.exit(main(sys.argv[1:]))` |
| Imports | `urllib.request`, `json`, `uuid`, `sys`, `os`, `argparse` — all stdlib, zero pip deps |
| Python target | 3.8+ (no f-string debugging, no `match`, no `|` union types at runtime) |
| Type hints | Optional 3.8+ compatible (`Dict[str, str]`, `List[Dict]`, `Optional[str]`, via `typing`) |

---

## 2. CLI Interface

```
usage: free-zen.py [-h] [--list] [--json] [--probe] [--launch]
                   [--launch-cmd CMD] [--no-color] [--timeout SECONDS]

Dynamically discover free models from opencode.ai zen gateway and
generate shell configuration for openclaude.

No API key needed — free models work with empty auth at the zen endpoint.

options:
  -h, --help            show this help message and exit
  --list                List free models (one per line, with metadata) and exit
  --json                Output free model info as JSON array and exit
  --probe               Also probe zen /v1/models for verification
  --launch              Export env vars and exec openclaude (or --launch-cmd)
  --launch-cmd CMD      Command to launch instead of "openclaude"
  --no-color            Disable ANSI color in --list output
  --timeout SECONDS     HTTP timeout for models.dev fetch (default: 10)
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (env vars printed, or launch succeeded) |
| 1 | Network error, parse error, no free models found |
| 2 | Usage error (invalid args) or `--launch` but binary not found |

---

## 3. Function Signatures, Docstrings, and Pseudo-code

### 3a. `generate_opencode_headers()`

```python
from typing import Dict
import uuid

def generate_opencode_headers() -> Dict[str, str]:
    """Build opencode-style HTTP headers for models.dev and zen probes.

    Returns a dict with:
      - User-Agent: "opencode/latest/1.3.15/cli"
      - x-opencode-client: "cli"
      - x-opencode-session: 26-char hex (first 26 chars of UUID4 hex)
      - x-opencode-project: 26-char hex (next 26 chars, or new UUID4)
      - x-opencode-request: 26-char hex (new UUID4)

    Three independent UUIDs per Plan A ruling (simpler, no offset edge cases).
    """
```

Pseudo-code:
```
session = uuid.uuid4().hex[:26]
project = uuid.uuid4().hex[:26]
request = uuid.uuid4().hex[:26]
return {
    "User-Agent": "opencode/latest/1.3.15/cli",
    "x-opencode-client": "cli",
    "x-opencode-session": session,
    "x-opencode-project": project,
    "x-opencode-request": request,
}
```

### 3b. `fetch_json(url, timeout, headers)`

```python
from typing import Dict, Optional
import urllib.request
import json

def fetch_json(url: str, timeout: int = 10,
               headers: Optional[Dict[str, str]] = None) -> dict:
    """Perform an HTTP GET and return parsed JSON.

    Args:
        url: Target URL.
        timeout: Request timeout in seconds (default 10).
        headers: Optional dict of HTTP headers.

    Returns:
        Parsed JSON as a Python dict.

    Raises:
        urllib.error.URLError: Network error (DNS, connection refused, timeout).
        urllib.error.HTTPError: Non-2xx status code.
        json.JSONDecodeError: Response body is not valid JSON.
    """
```

Pseudo-code:
```
req = urllib.request.Request(url, headers=headers or {}, method="GET")
with urllib.request.urlopen(req, timeout=timeout) as resp:
    return json.loads(resp.read())
```

### 3c. `fetch_models_dev_info(headers, timeout)`

```python
def fetch_models_dev_info(headers: Dict[str, str],
                          timeout: int = 10) -> dict:
    """Fetch model info from models.dev.

    URL source (in priority order):
      1. FREE_ZEN_MODELS_DEV_URL env var (for offline testing)
      2. https://models.dev/api.json (production)

    Returns the raw JSON dict from the response.

    Raises same exceptions as fetch_json.
    """
```

Pseudo-code:
```
url = os.environ.get("FREE_ZEN_MODELS_DEV_URL") or MODELS_DEV_URL
return fetch_json(url, timeout, headers)
```

### 3d. `filter_free_models(data)`

```python
from typing import Dict, List, Any

def filter_free_models(data: dict) -> List[Dict[str, Any]]:
    """Extract free, non-deprecated models from models.dev JSON.

    Filter criteria:
      1. status != "deprecated"
      2. cost.input == 0 (absent cost or absent cost.input → excluded)

    Returns sorted list of dicts with rich metadata per Plan B:
      - id: model identifier string
      - name: display name (or id if absent)
      - description: description string (or "")
      - context_window: context limit from info["limit"]["context"] (default 128000)
      - max_output: output limit from info["limit"]["output"] (default 64000)
      - reasoning: bool(info.get("reasoning", False))
      - vision: "image" in info["modalities"]["input"] (or False)

    Sorted alphabetically by name (case-insensitive).

    May raise ValueError if models.dev structure is unrecognizable.
    """
```

Pseudo-code:
```
models = data.get("opencode", {}).get("models", {})
if not isinstance(models, dict):
    raise ValueError("models.dev: 'opencode.models' is not an object")

free = []
for model_id, info in models.items():
    if not isinstance(info, dict):
        continue
    if info.get("status") == "deprecated":
        continue
    cost = info.get("cost")
    if cost is None or not isinstance(cost, dict):
        continue
    if cost.get("input") != 0:
        continue

    free.append({
        "id": model_id,
        "name": info.get("name", model_id),
        "description": info.get("description", ""),
        "context_window": info.get("limit", {}).get("context", 128000),
        "max_output": info.get("limit", {}).get("output", 64000),
        "reasoning": bool(info.get("reasoning", False)),
        "vision": "image" in (info.get("modalities", {}).get("input", [])),
    })

free.sort(key=lambda m: m["name"].lower())
return free
```

### 3e. `select_best_model(models)`

```python
from typing import List, Dict, Any, Optional

def select_best_model(
    models: List[Dict[str, Any]]
) -> Optional[str]:
    """Select the best default free model by prefix matching.

    Preference order (Plan B prefix matching):
      1. Exact match or prefix match on "deepseek-v4-flash-free"
      2. Prefix match on "deepseek"
      3. Prefix match on "nemotron"
      4. First model in the list (already sorted alphabetically)

    Returns model id string, or None if list is empty.
    """
```

Pseudo-code:
```
if not models:
    return None

for prefix in ["deepseek-v4-flash-free", "deepseek", "nemotron"]:
    for m in models:
        if m["id"] == prefix or m["id"].startswith(prefix):
            return m["id"]

return models[0]["id"]
```

### 3f. `format_env_exports(model_id, model_name)`

```python
def format_env_exports(model_id: str,
                        model_name: str = "") -> str:
    """Generate shell export statements for openclaude integration.

    Exports (from ruling):
      - CLAUDE_CODE_USE_OPENAI=1
      - OPENAI_BASE_URL=https://opencode.ai/zen/v1
      - OPENAI_API_KEY=""  (explicit empty — prevents key leakage)
      - OPENAI_MODEL=<model_id>
      - OPENAI_API_FORMAT=chat_completions

    Includes a usage comment per Plan A (helps users who capture output).
    """
```

Pseudo-code:
```
lines = []
lines.append("# Generated by free-zen.py — free OpenCode Zen models")
if model_name:
    lines.append(f"# Free model: {model_name} ({model_id})")
else:
    lines.append(f"# Free model: {model_id}")
lines.append("# Usage: eval \"$({})\" && openclaude".format(__file__))
lines.append("")
lines.append("export CLAUDE_CODE_USE_OPENAI=1")
lines.append("export OPENAI_BASE_URL=https://opencode.ai/zen/v1")
lines.append("export OPENAI_API_KEY=")
lines.append("export OPENAI_MODEL={}".format(model_id))
lines.append("export OPENAI_API_FORMAT=chat_completions")
return "\n".join(lines) + "\n"
```

### 3g. `print_model_table(models, default_id, color)`

```python
from typing import List, Dict, Any

def print_model_table(models: List[Dict[str, Any]],
                      default_id: str,
                      color: bool = True) -> None:
    """Print a human-readable table of free models to stderr.

    Args:
        models: List of model dicts from filter_free_models.
        default_id: Currently selected default model id.
        color: Enable ANSI color escape codes (default True).

    Output format:
      free-zen: free OpenCode Zen models (N):
        1. model-name-here         (ctx: 128k, max: 64k) ← default
        2. ...
    """
```

Pseudo-code:
```
lines = [f"free-zen: free OpenCode Zen models ({len(models)}):"]
for i, m in enumerate(models, 1):
    ctx = m["context_window"]
    max_out = m["max_output"]
    suffix = " ← default" if m["id"] == default_id else ""
    name_colored = color_name(m["name"], color)
    lines.append(f"  {i}. {name_colored:<30} (ctx: {ctx}, max: {max_out}){suffix}")

print("\n".join(lines), file=sys.stderr)
```

Color helper (internal): add ANSI green to the default model name when `color=True` and stderr is a TTY.

### 3h. `probe_zen_models(headers, timeout)`

```python
from typing import Optional, List

def probe_zen_models(headers: Dict[str, str],
                     timeout: int = 10) -> Optional[List[str]]:
    """Optionally probe the zen /v1/models endpoint for verification.

    Fetches https://opencode.ai/zen/v1/models with empty auth header.
    Returns list of model IDs from the endpoint, or None on failure.
    On failure, prints warning to stderr but does NOT exit.
    """
```

Pseudo-code:
```
try:
    data = fetch_json(ZEN_MODELS_URL, timeout, headers)
    # zen returns {"object": "list", "data": [{"id": "...", ...}, ...]}
    models = data.get("data", [])
    return [m["id"] for m in models if isinstance(m, dict)]
except Exception as e:
    print(f"free-zen: warning: zen /v1/models probe failed: {e}",
          file=sys.stderr)
    return None
```

### 3i. `launch_openclaude(env, cmd)`

```python
from typing import Dict, NoReturn

def launch_openclaude(env: Dict[str, str],
                      cmd: str = "openclaude") -> None:
    """Set environment variables and exec the target command.

    The new process replaces the current one (os.execvpe).
    If the command is not found, prints error and exits with code 2.
    """
```

Pseudo-code:
```
try:
    os.execvpe(cmd, [cmd], env)
except FileNotFoundError:
    print(f"free-zen: error: '{cmd}' not found on PATH", file=sys.stderr)
    sys.exit(2)
except PermissionError:
    print(f"free-zen: error: '{cmd}' is not executable", file=sys.stderr)
    sys.exit(2)
```

### 3j. `main(argv)`

```python
from typing import List

def main(argv: List[str]) -> int:
    """Parse arguments, orchestrate fetch/filter/output.

    Returns an exit code (0, 1, or 2).

    Flow:
      1. Parse args → exit 2 on invalid usage
      2. Generate headers
      3. Fetch models.dev info → exit 1 on network/parse failure
      4. Filter to free models → exit 1 if empty
      5. Select best default model
      6. If --probe: probe zen /v1/models (warn on failure, continue)
      7. Branch on mode:
         --json → print JSON array to stdout, exit 0
         --list → print table to stderr via print_model_table(), exit 0
         --launch → build env dict, call launch_openclaude()
         default → print shell exports to stdout, usage hint to stderr, exit 0
    """
```

Pseudo-code:
```
parser = argparse.ArgumentParser(...)  # all flags from §2
args = parser.parse_args(argv)

headers = generate_opencode_headers()

try:
    raw = fetch_models_dev_info(headers, args.timeout)
except (urllib.error.URLError, urllib.error.HTTPError) as e:
    # Try hardcoded fallback list
    models = FALLBACK_MODELS  # list of dicts with at least "id"
    print("free-zen: warning: using hardcoded fallback models "
          "(network unreachable)", file=sys.stderr)
    if not models:
        print(f"free-zen: error: {e}", file=sys.stderr)
        return 1
except json.JSONDecodeError as e:
    # Try hardcoded fallback list
    models = FALLBACK_MODELS
    print("free-zen: warning: using hardcoded fallback models "
          "(invalid JSON from models.dev)", file=sys.stderr)
    if not models:
        print(f"free-zen: error: {e}", file=sys.stderr)
        return 1
except ValueError as e:
    print(f"free-zen: error: {e}", file=sys.stderr)
    return 1

if raw was fetched successfully:
    try:
        models = filter_free_models(raw)
    except ValueError as e:
        print(f"free-zen: error: {e}", file=sys.stderr)
        return 1

if not models:
    print("free-zen: error: no free models found", file=sys.stderr)
    return 1

default_id = select_best_model(models)

if args.probe:
    zen_ids = probe_zen_models(headers, args.timeout)
    if zen_ids is not None and default_id not in zen_ids:
        print(f"free-zen: warning: selected model '{default_id}' "
              f"not in zen /v1/models", file=sys.stderr)

if args.json:
    print(json.dumps(models, indent=2))
    return 0

if args.list:
    print_model_table(models, default_id, color=not args.no_color)
    return 0

# Default mode: shell exports
exports = format_env_exports(default_id,
                              next(m["name"] for m in models
                                   if m["id"] == default_id))
print(exports, end="")  # stdout for eval
print(f"# Usage: eval \"$(python free-zen.py)\" && openclaude",
      file=sys.stderr)

if args.launch:
    env = os.environ.copy()
    env.update({
        "CLAUDE_CODE_USE_OPENAI": "1",
        "OPENAI_BASE_URL": "https://opencode.ai/zen/v1",
        "OPENAI_API_KEY": "",
        "OPENAI_MODEL": default_id,
        "OPENAI_API_FORMAT": "chat_completions",
    })
    launch_openclaude(env, args.launch_cmd or "openclaude")
    return 0  # unreachable if launch succeeds

return 0
```

---

## 4. Constants

```python
MODELS_DEV_URL = "https://models.dev/api.json"
ZEN_MODELS_URL = "https://opencode.ai/zen/v1/models"

FALLBACK_MODELS = [
    {"id": "deepseek-v4-flash-free", "name": "DeepSeek V4 Flash Free"},
    {"id": "nemotron-3-ultra-free", "name": "Nemotron 3 Ultra Free"},
    {"id": "mimo-v2.5-free", "name": "Mimo v2.5 Free"},
    {"id": "north-mini-code-free", "name": "North Mini Code Free"},
    {"id": "big-pickle", "name": "Big Pickle"},
    {"id": "hy3-free", "name": "Hy3 Free"},
]
```

Hardcoded fallback used only when models.dev is unreachable. Printed with a loud warning on stderr. IDs only — no pricing data (avoids staleness). Used as last-resort so `--list` / `--json` still function offline.

---

## 5. Env Var Reference

| Env Var | Value | Purpose |
|---------|-------|---------|
| `CLAUDE_CODE_USE_OPENAI` | `1` | Enable openclaude's OpenAI-compatible shim |
| `OPENAI_BASE_URL` | `https://opencode.ai/zen/v1` | Zen gateway endpoint |
| `OPENAI_API_KEY` | `""` (empty) | Prevent accidental key leakage; zen accepts empty auth for free models |
| `OPENAI_MODEL` | `<best-free-id>` | Default free model for the session |
| `OPENAI_API_FORMAT` | `chat_completions` | Zen free models use this API format |

### Dev / Test Env Var

| Env Var | Purpose |
|---------|---------|
| `FREE_ZEN_MODELS_DEV_URL` | Override models.dev URL for offline testing. Set to `file:///path/to/fixture.json` or a local HTTP server. |

---

## 6. Error Handling

| Scenario | Exit Code | Behavior |
|----------|-----------|----------|
| models.dev unreachable (network down) | 1 | Print `free-zen: Network error: ...` to stderr, try FALLBACK_MODELS with warning, exit 1 if fallback empty |
| models.dev returns non-JSON | 1 | Print `free-zen: Invalid JSON: ...` to stderr, try FALLBACK_MODELS with warning, exit 1 if fallback empty |
| models.dev structure unrecognized | 1 | Print `free-zen: ...` ValueError message to stderr, exit 1 (no fallback — data is wrong shape) |
| No free models found | 1 | Print `free-zen: error: no free models found` to stderr, exit 1 |
| zen /v1/models unreachable (`--probe`) | 0 | Print `free-zen: warning: ...` to stderr, continue (non-fatal) |
| `--launch` but binary not found | 2 | Print `free-zen: error: 'X' not found on PATH` to stderr, exit 2 |
| Invalid CLI args | 2 | Argparse prints help to stderr, exits 2 |
| DNS failure / timeout | 1 | Caught as `URLError`, message to stderr, fallback to FALLBACK_MODELS |
| HTTP 4xx/5xx | 1 | Caught as `HTTPError`, message to stderr, fallback to FALLBACK_MODELS |
| Hardcoded fallback in use | 0 | Print loud warning to stderr, use fallback, continue normally |

---

## 7. Implementation Order

1. **Skeleton + headers**: Write `free_zen.py` with shebang, imports, `generate_opencode_headers()`, constants, `main()` stub with argparse
2. **Networking + parsing**: Implement `fetch_json()`, `fetch_models_dev_info()`
3. **Filtering + selection**: Implement `filter_free_models()`, `select_best_model()`
4. **Core output**: Implement `format_env_exports()` — the default mode
5. **Display modes**: Implement `print_model_table()` for `--list`, JSON dump for `--json`
6. **Launch mode**: Implement `launch_openclaude()` with `os.execvpe`, wire `--launch-cmd`
7. **Probe mode**: Implement `probe_zen_models()` and wire `--probe`
8. **Hardcoded fallback**: Wire FALLBACK_MODELS into error handling path
9. **Test suite**: Write `test_free_zen.py` with all unit tests (§8)
10. **Polish**: Help text, error messages, edge cases, color detection, `--no-color`
11. **Integration test**: Run acceptance criteria (§10)

---

## 8. Test Structure (`test_free_zen.py`)

Using stdlib `unittest` + `unittest.mock`. No external test deps.

### Headers tests

| Test | What it verifies |
|------|------------------|
| `test_generate_opencode_headers_structure` | Returns dict with exactly 5 expected keys |
| `test_generate_opencode_headers_format` | User-Agent matches `opencode/latest/*/cli`, x-opencode-* values are 26-char `[a-f0-9]{26}` strings |
| `test_generate_opencode_headers_uniqueness` | Two calls produce different values for all 3 UUID fields |

### Filtering tests

| Test | What it verifies |
|------|------------------|
| `test_filter_free_models_empty` | Empty dict returns `[]` |
| `test_filter_free_models_deprecated` | Models with `status: "deprecated"` excluded |
| `test_filter_free_models_paid` | Models with `cost.input != 0` excluded |
| `test_filter_free_models_no_cost` | Models with no cost field excluded |
| `test_filter_free_models_free` | Models with `cost.input == 0` included |
| `test_filter_free_models_mixed` | Mixed input correctly filters |
| `test_filter_free_models_malformed` | Non-dict model entries skipped gracefully |
| `test_filter_free_models_bad_structure` | Non-dict `opencode.models` raises ValueError |

### Selection tests

| Test | What it verifies |
|------|------------------|
| `test_select_best_model_deepseek_v4` | Exact match `deepseek-v4-flash-free` preferred |
| `test_select_best_model_deepseek` | Prefix match `deepseek` works |
| `test_select_best_model_nemotron` | Falls back to `nemotron` prefix |
| `test_select_best_model_fallback` | Falls back to first model |
| `test_select_best_model_empty` | Returns None for empty list |

### Output tests

| Test | What it verifies |
|------|------------------|
| `test_format_env_exports` | Produces all 5 expected `KEY=VAL` lines |
| `test_format_env_exports_empty_key` | `OPENAI_API_KEY` exported as `=` (no value) |
| `test_format_env_exports_model_id` | `OPENAI_MODEL=` contains the given model_id |
| `test_format_env_exports_comment` | Output contains usage comment |

### Network tests (mocked)

| Test | What it verifies |
|------|------------------|
| `test_fetch_json_success` | Mocked `urlopen` returns parsed JSON |
| `test_fetch_json_http_error` | Mocked `HTTPError` raises |
| `test_fetch_json_urlerror` | Mocked `URLError` raises |
| `test_fetch_json_parse_error` | Mocked invalid JSON body raises `json.JSONDecodeError` |
| `test_fetch_json_headers` | Headers dict passed through to Request |

### Main / integration tests (mocked)

| Test | What it verifies |
|------|------------------|
| `test_main_default_output` | `main([])` returns 0, prints exports to stdout |
| `test_main_list` | `main(["--list"])` returns 0, prints to stderr |
| `test_main_json` | `main(["--json"])` returns 0, valid JSON to stdout |
| `test_main_network_error` | `main([])` returns 1 when fetch raises URLError and no fallback models exist |
| `test_main_no_free_models` | `main([])` returns 1 when filter returns empty |
| `test_main_launch_not_found` | `main(["--launch"])` returns 2 when binary missing |
| `test_main_probe_success` | `main(["--probe"])` returns 0, calls probe |
| `test_main_fallback_used` | `main([])` returns 0 using FALLBACK_MODELS when network fails |
| `test_dev_url_env_override` | `FREE_ZEN_MODELS_DEV_URL` env var overrides production URL |

### Test file structure

```python
import unittest
from unittest.mock import patch, MagicMock
import sys, json, os

# Import the module under test
sys.path.insert(0, ".")
import free_zen  # or exec(open("free_zen.py").read()) for flat script
```

---

## 9. Files

| File | Action | Lines |
|------|--------|-------|
| `free-zen.py` | **Create** | ~350 lines |
| `test_free_zen.py` | **Create** | ~300 lines |

No other files modified. No config files. No docs (script is self-documenting via `--help`).

---

## 10. Acceptance Criteria

1. `python free-zen.py` exits 0, prints to stdout: `CLAUDE_CODE_USE_OPENAI=1`, `OPENAI_BASE_URL=https://opencode.ai/zen/v1`, `OPENAI_API_KEY=`, `OPENAI_MODEL=<model>`, `OPENAI_API_FORMAT=chat_completions`. Stderr contains usage hint.

2. `python free-zen.py --list` exits 0. Stdout is empty. Stderr contains a table with ≥1 model row and a `← default` indicator.

3. `python free-zen.py --json` exits 0. Stdout is valid JSON array with ≥1 element, each containing `id`, `name`, `context_window`, `max_output`, `reasoning`, `vision` fields.

4. `eval "$(python free-zen.py)" && echo "$OPENAI_MODEL"` prints a non-empty model name.

5. `python free-zen.py --probe` exits 0. If zen is reachable, it may produce a warning on stderr. If zen is down, a warning is printed but exit code is still 0.

6. `python free-zen.py --launch --launch-cmd /bin/sh -c 'echo $OPENAI_MODEL'` launches the shell and prints the model name. (Requires exec semantics.)

7. `python free-zen.py --no-color --list` exits 0 with no ANSI escape codes in stderr output.

8. `python free-zen.py --help` prints help text listing all options.

9. Running without network (`unshare -n python free-zen.py` or `FREE_ZEN_MODELS_DEV_URL=/dev/null`): exits 1 with error on stderr. If FALLBACK_MODELS is non-empty, exits 0 with a warning on stderr.

10. `FREE_ZEN_MODELS_DEV_URL=file:///path/to/fixture.json python free-zen.py` uses the local file.

11. `python -m unittest test_free_zen.py -v` passes all unit tests.

12. `python free-zen.py --invalid-flag` exits 2 with argparser error.

---

## 11. Edge Cases & Defensive Design

| Edge case | Handling |
|-----------|----------|
| models.dev returns models with `cost.input: 0` but `cost.output: > 0` | Still treated as free (matches pi-opencode-zen behavior) |
| models.dev has `cost: null` per model | Excluded (no cost info = assume paid) |
| models.dev structure changes | `.get()` with defaults; explicit ValueError on bad shape |
| models.dev is slow (>10s) | Configurable timeout, default 10s |
| `OPENAI_API_FORMAT` varies per model | Zen free models all use `chat_completions`; single value correct |
| User has existing `OPENAI_*` env vars | `eval` overrides them (expected behavior) |
| Multiple Python versions | 3.8+ syntax only |
| No network in CI | `FREE_ZEN_MODELS_DEV_URL` for fixture; FALLBACK_MODELS as last resort |
| Hardcoded fallback staleness | IDs change rarely; pricing not included; loud warning on stderr |
| `--launch` with relative path `./my-binary` | `os.execvpe` resolves via PATH — `--launch-cmd ./my-binary` works if `./` is on PATH |

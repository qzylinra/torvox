# Stage 3 — Plan B: Runner Script Approach

## Approach Overview

Create a two-file Node.js solution:

1. **`setup.js`** — Discovers free models, writes `~/.openclaude.json`, generates the runner script
2. **`opencode-zen-free`** (generated script) — Node.js spawner that launches `openclaude` with env vars injected programmatically via `child_process.spawn()`, passes through all CLI args

No `.env` file. No shell `export`. No `OPENCODE_API_KEY`. Env vars exist only in the child process address space.

---

## Model Discovery

### Step 1: Fetch pricing + status from models.dev

**Endpoint**: `GET https://models.dev/api.json`

**Response shape** (3MB+ JSON, extract `opencode.models` key):

```json
{
  "opencode": {
    "models": {
      "<model-id>": {
        "id": "deepseek-v4-flash-free",
        "name": "DeepSeek V4 Flash Free",
        "status": null | "deprecated",
        "cost": {
          "input": 0,
          "output": 0,
          "cache_read": 0
        },
        "limit": { "context": 200000, "output": 128000 },
        "tool_call": true,
        ...
      }
    }
  }
}
```

**Filter rule**: `cost.input === 0 && status !== "deprecated"`

As of Jul 2026, 6 non-deprecated free models exist:
- `big-pickle`, `deepseek-v4-flash-free`, `hy3-free`, `mimo-v2.5-free`, `nemotron-3-ultra-free`, `north-mini-code-free`

### Step 2: Fetch visible models from OpenCode Zen

**Endpoint**: `GET https://opencode.ai/zen/v1/models`

Works **without authentication** (returns all models). With auth header it would scope to the user's plan; our approach works without any key.

**Response shape**:

```json
{
  "object": "list",
  "data": [
    { "id": "deepseek-v4-flash-free", "object": "model", "created": 1784528582, "owned_by": "opencode" },
    ...
  ]
}
```

**Parse**: `response.data.map(m => m.id)` → `Set<string>`

### Step 3: Intersect + filter

```
free_models = models_dev.models.entries()
  .filter(m => m.cost.input === 0 && m.status !== "deprecated")
  .filter(m => visible_models_set.has(m.id))
```

### Step 4: Sort for deterministic default

Sort alphabetically by model ID. First entry becomes the default model for the runner. Pick `deepseek-v4-flash-free` if present (it's the first free one in zen's response).

---

## Config Storage Strategy — `~/.openclaude.json`

Write a config that works with `agentModels` + `agentRouting` (the only way to pin api_key without env/.env).

### Structure

```json
{
  "agentModels": {
    "opencode-zen": {
      "base_url": "https://opencode.ai/zen/v1",
      "api_key": "public"
    },
    "opencode-zen-big-pickle": {
      "model": "big-pickle",
      "base_url": "https://opencode.ai/zen/v1",
      "api_key": "public"
    },
    "opencode-zen-deepseek-v4-flash-free": {
      "model": "deepseek-v4-flash-free",
      "base_url": "https://opencode.ai/zen/v1",
      "api_key": "public"
    },
    ...
  },
  "agentRouting": {
    "default": "opencode-zen-deepseek-v4-flash-free"
  }
}
```

Key design decisions:
- **`api_key: "public"`** — OpenCode Zen free models accept `OPENAI_API_KEY=public` as auth. Stored in `agentModels.<key>.api_key`, openclaude passes it as `OPENAI_API_KEY` to the OpenAI shim.
- **One entry per model** — each with explicit `model` field so openclaude sends the right model name.
- **Separate `opencode-zen` entry** — serves as a base config entry (model-agnostic with only base_url + api_key), usable when the user wants to override model at runtime.
- **Default routing** — points to the first discovered free model.

### Runtime override via `agentRouting`

The config also sets up `opencode-zen` as the base profile entry without a `model` field. The generated runner will use `--model <model-id>` to override.

**Important**: openclaude has NO `availableModels` field (that is an Anthropic Claude Code feature, not ported). The `agentModels` mechanism in openclaude provides per-agent provider overrides — it's the correct vehicle for this.

---

## Runner Design — `opencode-zen-free`

A standalone Node.js script placed at a system PATH location (e.g., `/usr/local/bin/opencode-zen-free`).

### Mechanism

```js
#!/usr/bin/env node
const { spawn } = require('child_process');

const env = {
  ...process.env,
  CLAUDE_CODE_USE_OPENAI: '1',
  OPENAI_API_KEY: 'public',
  OPENAI_BASE_URL: 'https://opencode.ai/zen/v1',
  OPENAI_MODEL: '<default-free-model>',
};

// All CLI args passed through
const proc = spawn('openclaude', process.argv.slice(2), {
  env,
  stdio: 'inherit',
});

proc.on('exit', (code) => process.exit(code ?? 1));
```

### What it does

1. **`env` option** in `spawn()` — merges current env + our overrides. These vars are set only in the child process, never in the parent shell. No `export`, no `.env` file.
2. **`process.argv.slice(2)`** — passes through ALL CLI flags/args to openclaude (e.g., `--print`, `--model`, `-p`).
3. **`stdio: 'inherit'`** — full TTY interaction preserved.

### Why `CLAUDE_CODE_USE_OPENAI=1` is required

Without this flag, openclaude uses its built-in Anthropic route and ignores `OPENAI_BASE_URL` / `OPENAI_MODEL`. The `agentModels` config only works WITHIN an OpenAI-compatible session — it does NOT switch the provider protocol. The env var is the guard that activates the OpenAI shim path.

### Why `spawn()` not `exec()` or `execSync()`

- `spawn()` with `stdio: 'inherit'` preserves interactive terminal behavior (PTY, SIGINT, colors)
- `exec()` buffers output (bad for streaming)
- `execSync()` blocks the event loop

### Args passthrough

User runs: `opencode-zen-free --model big-pickle -p "hello"` → spawns `openclaude --model big-pickle -p "hello"` with env vars.

The `--model` flag overrides the default `OPENAI_MODEL` in env (openclaude CLI `--model` takes precedence over `OPENAI_MODEL` env var).

---

## What Happens When the User Types `openclaude` vs the Runner

| Scenario | Env vars | Result |
|---|---|---|
| `openclaude` | None | Default provider (Gitlawb Opengateway or Anthropic). No OpenCode Zen. |
| `openclaude --provider-env-file .env` | Loaded from .env file | Works, but requires .env file (forbidden). |
| `opencode-zen-free` | Injected by spawner | OpenCode Zen with default free model. No .env. No export. |
| `opencode-zen-free --model big-pickle` | Injected by spawner | OpenCode Zen with `big-pickle` free model. |
| `opencode-zen-free -p "explain git"` | Injected by spawner | Non-interactive OpenCode Zen. |

Without the runner, typing `openclaude` alone gives zero free models configured. The runner is the only entry point that provides the env vars.

---

## Files to Create

### 1. `setup.js` — standalone Node.js script (zero npm deps)

Location: any directory (user runs `node setup.js`), suggested `/usr/local/lib/opencode-zen/setup.js`

Function:
- Fetch both APIs in parallel (`Promise.all`)
- Parse, intersect, filter
- Write `~/.openclaude.json` with config
- Write `opencode-zen-free` runner script to a configurable PATH (default `/usr/local/bin/opencode-zen-free`)
- Print summary of discovered free models

Error handling:
- If `models.dev/api.json` fails → hard error (no pricing data = cannot determine free models)
- If `/zen/v1/models` fails → still proceed using only pricing data (fallback: all free non-deprecated models)
- If both fail → exit with clear error message
- If `~/.openclaude.json` write fails → print path and error
- If runner script write fails → print path and error

### 2. `opencode-zen-free` (generated by setup.js)

Location: `/usr/local/bin/opencode-zen-free` (or `~/.local/bin/opencode-zen-free`)

Generated content — the spawner script described above. The model ID is baked in at generation time.

### 3. Optionally: `package.json` at `/usr/local/lib/opencode-zen/package.json`

Only if needed for `node` shebang discovery. Minimal: `{ "name": "opencode-zen-free", "type": "commonjs" }`.

---

## Test Strategy

Tests use Node.js built-in `node:test` (available since Node 18, stable in 22+) — zero deps.

### Test 1: API fetching + filtering (mocked HTTP)

Use `node:test` + `node:mock` to intercept `fetch()` calls:

```js
mock.method(globalThis, 'fetch', async (url) => {
  if (url === 'https://models.dev/api.json') {
    return { ok: true, json: async () => ({ opencode: { models: { /* test data */ } } }) };
  }
  if (url === 'https://opencode.ai/zen/v1/models') {
    return { ok: true, json: async () => ({ object: 'list', data: [/* test data */] }) };
  }
});
```

Cover:
- Happy path: both APIs succeed, 3 free models found, correct intersection
- One model in zen list but not free → excluded
- One model free but deprecated → excluded
- One model free + non-deprecated + visible → included
- Zen API fails → falls back to pricing-only
- Both APIs fail → error

### Test 3: Config generation

- Verify `~/.openclaude.json` output structure matches expected schema
- Verify `agentModels` contains one entry per free model
- Verify `agentRouting.default` points to first free model
- Verify all entries have `api_key: "public"` and `base_url: "https://opencode.ai/zen/v1"`
- Verify each model entry has explicit `model` field

### Test 4: Runner script generation

- Verify shebang line: `#!/usr/bin/env node`
- Verify `spawn()` is called with `openclaude` as command
- Verify `process.argv.slice(2)` is passed as args
- Verify env contains: `CLAUDE_CODE_USE_OPENAI=1`, `OPENAI_API_KEY=public`, `OPENAI_BASE_URL=https://opencode.ai/zen/v1`, `OPENAI_MODEL=<default>`
- Verify `stdio: 'inherit'` is set

### Test 5: Edge cases

- Empty free model list (no free models matching criteria) → empty config, warn user
- Model ID with special characters (unlikely since all are kebab-case)
- Very long model list (performance — should be fine, <100 models)
- `~/.openclaude.json` already exists — we OVERWRITE with a warning message (backup is user's responsibility)
- Runner script path not writable → print clear instruction with suggested paths

### Test 6: No-op safety

- Verify setup.js does NOT create any `.env` file
- Verify setup.js does NOT read `OPENCODE_API_KEY` from env (it doesn't need it)
- Verify runner script does NOT reference `OPENCODE_API_KEY`

---

## API Summary

| What | Endpoint | Auth | Used For |
|---|---|---|---|
| Model pricing + status | `GET https://models.dev/api.json` | None | Extract `opencode.models` map, filter by `cost.input === 0 && status !== "deprecated"` |
| Visible models | `GET https://opencode.ai/zen/v1/models` | None (Bearer optional) | Get model IDs the user can access; without auth returns all models |
| Runtime inference | `POST https://opencode.ai/zen/v1/chat/completions` | `Authorization: Bearer public` | Used by openclaude via OpenAI-compatible shim — no direct call from our code |

---

## Error Handling Matrix

| Failure | Behaviour |
|---|---|
| `models.dev/api.json` fetch fails (network error) | `setup.js` exits with error: "Cannot fetch model pricing data" |
| `models.dev/api.json` returns non-200 | Same as above |
| `/zen/v1/models` fetch fails | Continue with pricing-only filter (no intersection) |
| `/zen/v1/models` returns non-200 | Same as above |
| Both fetches fail | Exit with error: "Cannot reach OpenCode Zen or model pricing APIs" |
| `models.dev` JSON missing `opencode.models` key | Exit with error: "Unexpected pricing data format" |
| `opencode.models` is empty | Exit with error: "No models found in pricing data" |
| Zero free models after filtering | Write empty config, warn user "No free models available" |
| Cannot write `~/.openclaude.json` | Print path + OS error, suggest checking permissions |
| Cannot write runner script | Print path + OS error, suggest checking permissions |
| Runner script has no execute bit | `setup.js` must `chmod +x` the generated file |
| User removes `~/.openclaude.json` after setup | Runner still works (env vars + default model) but `agentRouting` won't apply; user must re-run setup |

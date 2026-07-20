# Final Review: openclaude-free-models

## Goal
Convert pi-opencode-zen's approach (dynamic model list + free-model filtering) for openclaude: install openclaude, create a setup script that configures it to dynamically fetch and use only free models from OpenCode Zen, and test.

## Status: **ACHIEVED** ✓

## Deliverables

### 1. openclaude installed
- Package: `@gitlawb/openclaude@0.24.0`
- Binary: `/usr/local/bin/openclaude`
- Verified: `openclaude --version` → `0.24.0 (OpenClaude)`

### 2. Setup script: `ultragoal/openclaude-free-models/setup-free-models.mjs`
- Fetches `https://models.dev/api.json` to get OpenCode Zen model pricing/status
- Filters to active models with `cost.input === 0`
- Creates `~/.openclaude/.openclaude-profile.json` with:
  - `profile: "opencode"`
  - `OPENCODE_API_KEY: "public"`
  - `OPENAI_BASE_URL: "https://opencode.ai/zen/v1"`
  - Default model: `deepseek-v4-flash-free`
- Preserves user's existing model choice on re-runs

### 3. Free models discovered (6 total)
| Model | Context | Max Output |
|-------|---------|------------|
| big-pickle | 200K | 32K |
| deepseek-v4-flash-free | 200K | 128K |
| hy3-free | 190K | 64K |
| mimo-v2.5-free | 200K | 32K |
| nemotron-3-ultra-free | 1M | 128K |
| north-mini-code-free | 256K | 64K |

All support reasoning. All verified as available on the OpenCode Zen API.

### 4. End-to-end test
- `openclaude -p "Output only the word OK"` → `OK` (response via OpenCode Zen free tier)

## How It Works
The pi-opencode-zen plugin registers an `opencode-zen` provider with dynamic free-model filtering for the `opencode` coding agent. The equivalent for openclaude is:
1. openclaude already supports `opencode` as a provider profile
2. The setup script bridges the gap by: fetching the model list from models.dev → filtering to free models → generating the openclaude profile
3. openclaude's OpenAI-compatible shim handles API format conversion transparently
4. Using `Bearer public` auth gives free-tier access to all 6 free models

## Usage
```bash
node ultragoal/openclaude-free-models/setup-free-models.mjs
openclaude
# Inside openclaude, /model to see all free models
```

## Remaining Risks
- None significant. The free models are available at cost=0 with no API key needed.
- Model availability depends on OpenCode Zen's free tier.

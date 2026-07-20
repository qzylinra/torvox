# Roadmap: opencode-zen-adapter

## Goal

Adapt pi-opencode-zen's dynamic free-model discovery for openclaude. Create a setup tool that:
- Discovers free OpenCode Zen models dynamically
- Configures openclaude with zero env vars, zero .env files, zero OPENCODE_API_KEY
- Only free models (no paid leaks)
- Minimal, best-language implementation

## Stages

### Stage 1: Goal Decomposition ✅
Exploration reports, roadmap.

### Stage 2: Install openclaude
Reinstall openclaude v0.24.0 cleanly.

### Stage 3: Build adapter
Create `setup.js` — a Node.js script that:
- Fetches models from `models.dev/api.json` for pricing
- Fetches visible models from OpenCode Zen API
- Filters to free models (cost.input === 0, not deprecated)
- Writes `~/.openclaude.json` with allowlist + agent model config
- Generates a runner script (`opencode-zen-free`) that wraps openclaude with inline env vars
- No `.env` file created anywhere
- No `OPENCODE_API_KEY` required
- Minimum external deps (Node.js built-ins only)

**AC**: 20+ unit tests, no .env files generated, no OPENCODE_API_KEY referenced.

### Stage 4: E2E testing
Test with real OpenCode Zen API:
- Default free model works
- At least 2 other free models work
- Clean env confirmed (no leak)
- Paid models not accessible
- Runner script works without any shell env vars

**AC**: 3 models tested, clean env, no paid access.

### Stage 5: Acceptance & commit
Review, issues, final disposition.

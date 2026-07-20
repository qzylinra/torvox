# Roadmap: free-zen-openclaude

## Overall Goal

Convert the pi-opencode-zen pattern (dynamic model fetching from models.dev, free-model-only filtering, simulated opencode client headers) into an openclaude-compatible form: a lightweight Python script that dynamically discovers free models from the opencode.ai zen gateway, configures openclaude via environment variables to use only free models, and requires no manual setup, no API key, no config file changes, and no wrapper/proxy.

## Constraints

| # | Constraint | Source |
|---|-----------|--------|
| C1 | Dynamic fetch every run — no hardcoded model list | models.dev endpoint |
| C2 | Only free models (cost.input === 0, not deprecated) | models.dev filtering |
| C3 | No manual env var setup — script handles it | User requirement |
| C4 | Minimal configuration | User requirement |
| C5 | No OPENCODE_API_KEY needed | Confirmed: zen endpoint accepts free model requests without auth |
| C6 | No .env file | User requirement |
| C7 | Best language (Python — stdlib only, no deps) | User requirement |
| C8 | No wrapping/proxy/dependency on other software | User requirement |
| C9 | Simulated opencode client headers for probes | Original pi-opencode-zen pattern |
| C10 | Must not modify openclaude config files | User requirement |
| C11 | Install openclaude and test with actual API call | User requirement |

## Stages

### Stage 1: Write `free-zen.py`

| Aspect | Detail |
|--------|--------|
| **Objective** | Python script that fetches models.dev, filters to free models, generates openclaude env config with opencode-style headers |
| **Acceptance** | Script runs, prints valid env vars for openclaude, uses only stdlib |
| **Depends on** | — |

### Stage 2: Install openclaude

| Aspect | Detail |
|--------|--------|
| **Objective** | `npm install -g @gitlawb/openclaude@latest` and verify it's installed |
| **Acceptance** | `openclaude --version` exits 0 |
| **Depends on** | — |

### Stage 3: Test with a free model

| Aspect | Detail |
|--------|--------|
| **Objective** | Run openclaude with free-zen.py generated config, send a real prompt, verify response |
| **Acceptance** | openclaude starts, processes a prompt, returns output from a free model (no payment, no API key) |
| **Depends on** | Stage 1, Stage 2 |

## Model Details

### Active free models (from models.dev, cost.input === 0, not deprecated)

| # | Model ID | Notes |
|---|----------|-------|
| 1 | `deepseek-v4-flash-free` | Best for coding (current model) |
| 2 | `big-pickle` | Generic free model |
| 3 | `hy3-free` | Generic free model |
| 4 | `mimo-v2.5-free` | Xiaomi MiMo free tier |
| 5 | `nemotron-3-ultra-free` | NVIDIA Nemotron free |
| 6 | `north-mini-code-free` | North AI free |

### Verified: zen endpoint accepts free model requests with empty auth header

Tested: `curl -H "Authorization: Bearer " -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Say hello"}],"max_tokens":10}' https://opencode.ai/zen/v1/chat/completions` → HTTP 200 with response

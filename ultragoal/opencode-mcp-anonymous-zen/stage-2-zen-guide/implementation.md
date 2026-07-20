# Implementation: Stage 2 — Zen Anonymous Mode Guide

## Concept

OpenCode Zen exposes free models when no valid API key is provided. Setting the key to `"public"` or leaving it empty enables anonymous mode, filtering to only `cost.input === 0` models.

## 4 Setup Methods

### 1. TUI (Easiest)
Run `/connect` in OpenCode, select OpenCode Zen, press Enter at API key prompt.

### 2. Environment Variable
```bash
export OPENCODE_API_KEY=public
opencode --model opencode/big-pickle
```

### 3. Config File
```json
{
  "model": "opencode/deepseek-v4-flash-free"
}
```
Model ID format: `opencode/<model-id>`.

### 4. Automated Setup
```bash
npx my-opencode-config
```
Auto-configures free Zen + Gemini free tier.

## Free Models

| Model ID | Name |
|---|---|
| opencode/big-pickle | Big Pickle |
| opencode/deepseek-v4-flash-free | DeepSeek V4 Flash Free |
| opencode/mimo-v2.5-free | MiMo-V2.5 Free |
| opencode/north-mini-code-free | North Mini Code Free |
| opencode/nemotron-3-ultra-free | Nemotron 3 Ultra Free |

## Zen API Endpoints
- GET `https://opencode.ai/zen/v1/models` — list models (Auth: Bearer public)
- POST `https://opencode.ai/zen/v1/chat/completions` — OpenAI-compatible
- POST `https://opencode.ai/zen/v1/messages` — Anthropic-compatible
- POST `https://opencode.ai/zen/v1/responses` — Responses API
- GET `https://models.dev/api.json` — model metadata

## Privacy
Free models may log/use data for improvement during the free period.

## pi-opencode-zen Architecture
- `isPublicMode(key)`: returns true if key is undefined or "public"
- `isFreeModel(model)`: checks `cost.input === 0`
- `opencodeHeaders()`: mimics CLI User-Agent for compatibility

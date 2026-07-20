# Exploration: pi-opencode-zen/index.ts

## Source
`https://raw.githubusercontent.com/ravshansbox/pi-opencode-zen/refs/heads/main/index.ts`

## Structure
A plugin for `@earendil-works/pi-coding-agent` that registers an `opencode-zen` provider.

## Key Mechanisms

### 1. Static Model Definition
Defines `allModels` array with 35+ models from multiple families (Claude, GPT, Gemini, GLM, MiniMax, Kimi, Qwen, etc.). Each model has: id, name, reasoning, input types, cost, contextWindow, maxTokens.

### 2. Endpoint Routing
Maps each model to an API backend via `endpoints` record:
- `anthropic-messages` — Claude models
- `google-generative-ai` — Gemini models
- `openai-responses` — GPT models (Responses API)
- `openai-completions` — Everything else (GLM, MiniMax, Kimi, etc.)

### 3. API Key Resolution
- Reads `OPENCODE_API_KEY` env var
- Falls back to `~/.pi/agent/auth.json` → `opencode-zen` key
- Returns `undefined` if neither available

### 4. Dynamic Model Discovery
- `fetchVisibleModelIds(apiKey)`: Calls `GET https://opencode.ai/zen/v1/models` with Bearer auth → extracts visible model IDs from response
- `fetchModelsDevInfo()`: Calls `https://models.dev/api.json` → extracts `opencode.models` section with pricing/status

### 5. Free-Model Filtering
- `isPublicMode(apiKey)`: Returns true if API key is falsy or `"public"`
- `isFreeModel(model)`: Returns true if `cost.input === 0`
- In public mode: only models with `cost.input === 0` are included
- Deprecated models are filtered out when models.dev info is available

### 6. Provider Registration
- `pi.registerProvider("opencode-zen", {...})` with:
  - `baseUrl`: `https://opencode.ai/zen/v1`
  - `apiKey`: `OPENCODE_API_KEY`
  - `api`: `openai-completions` (fallback)
  - `streamSimple`: Custom routing function that dispatches to the correct backend stream function
  - `models`: Filtered list from `getVisibleModels()`

## Key Differences vs OpenClaude
- pi ecosystem uses a plugin/provider registration model
- pi has separate stream functions per API backend (anthropic, google, openai-completions, openai-responses)
- OpenClaude uses an OpenAI-compatible shim for all non-native providers
- OpenClaude has a complex catalog/registry system with vendor, gateway, brand, and model descriptors

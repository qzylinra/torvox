# Exploration: pi-opencode-zen

**Source**: https://raw.githubusercontent.com/ravshansbox/pi-opencode-zen/refs/heads/main/index.ts

## Architecture

A provider plugin for `@earendil-works/pi-coding-agent` that registers models under the `"opencode-zen"` provider name.

## Key mechanisms

1. **Static model list**: Hardcoded `allModels` array (~40 models) with id, name, reasoning, input types, cost, context window, maxTokens.
2. **Static endpoint routing**: Hardcoded `endpoints` map mapping model IDs to backend API type (anthropic-messages, google-generative-ai, openai-responses, openai-completions) and base URL.
3. **Dynamic filtering**: Two fetches happen at startup:
   - `fetchVisibleModelIds(apiKey)` — calls `GET {BASE_URL}/models` with `Authorization: Bearer {apiKey}` to get accessible model IDs.
   - `fetchModelsDevInfo()` — fetches `https://models.dev/api.json` and extracts `opencode.models` for pricing + status.
4. **Free model filter**: `getVisibleModels()` filters to models where `cost.input === 0` and `status !== "deprecated"`. In public mode (no API key or key="public"), only free models are returned.
5. **API key resolution**: `getConfiguredApiKey()` checks `OPENCODE_API_KEY` env var first, then falls back to `~/.pi/agent/auth.json`.

## Limitations for our use case

- Tightly coupled to `@earendil-works/pi-coding-agent` extension API — not portable.
- Hardcoded model list and endpoint routing (needs manual updates when new models appear).
- Uses `OPENCODE_API_KEY` which we must avoid.
- Not usable with openclaude as-is.

# Exploration: OpenClaude Provider System

## Source
`https://github.com/Gitlawb/openclaude` (v0.24.0)

## Architecture Overview

OpenClaude supports 30+ providers through a layered system:

### 1. Provider Profile System (`src/utils/providerProfile.ts`)
- Stored as `.openclaude-profile.json`
- `ProviderProfile` union type includes `'opencode'`
- `OPENCODE_API_KEY` is a managed env var
- `buildLaunchEnv()` maps profile to env vars

### 2. Route Resolution (`src/integrations/routeMetadata.ts`)
- `resolveActiveRouteIdFromEnv()` checks env vars → base URL → profile
- OpenCode Zen detected via `CLAUDE_CODE_USE_OPENAI` + base URL matching
- `resolveEnvOnlyProviderRouteId()` for env-var-only providers

### 3. API Transport (`src/services/api/`)
- `openaiShim.ts`: Main OpenAI-compatible shim — translates Anthropic SDK calls to OpenAI format
- `codexShim.ts`: OpenAI Responses API format
- `client.ts`: Routes between Anthropic native, Bedrock, Vertex, OpenAI-compatible, etc.

### 4. Model Catalog (`src/integrations/`)
- Registry with 5 descriptor types: Vendor, Gateway, AnthropicProxy, Brand, Model
- Auto-generated from `src/integrations/generated/integrationArtifacts.generated.ts`
- Each route has a `ModelCatalogConfig` with `source: 'static' | 'dynamic' | 'hybrid'`
- Dynamic discovery via `discoveryService.ts` calling `/v1/models` or custom endpoints
- Caching with TTL and refresh modes

### 5. Model Picker (`src/utils/model/modelOptions.ts`)
- Builds options from: user tier × active provider × route catalog × profile models × dynamic fetches
- `getActiveOpenAIRouteCatalogOptions()` fetches route catalog entries
- `getScopedAdditionalModelOptions()` includes bootstrap-discovered models

### 6. OpenCode Zen Integration (Current State)
- `opencode` is a recognized `ProviderProfile`
- `DEFAULT_OPENCODE_BASE_URL = 'https://opencode.ai/zen/v1'`
- Routes through OpenAI-compatible shim
- Model list is NOT dynamically fetched — uses whatever the route catalog provides
- No free-model filtering
- No public mode support

## What's Missing for the Goal
1. No dynamic model discovery for OpenCode Zen's free models
2. No free-model-only filtering
3. No public mode (no-API-key usage)

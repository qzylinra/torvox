# Exploration: pi-opencode-zen

**Source:** `https://raw.githubusercontent.com/ravshansbox/pi-opencode-zen/refs/heads/main/index.ts`

## Overview

`pi-opencode-zen` is a **pi-ai / pi-coding-agent extension** that proxies LLM requests through `opencode.ai/zen/v1` — a free API tier from the opencode.ai platform. It registers a custom provider called `"opencode-zen"` with the pi-ai framework and transparently routes streaming API calls for ~45 models through the opencode.ai zen endpoint.

---

## 1. Model List Fetching (Two Sources)

The extension uses **two independent sources** to build its visible model list:

### 1a. Static Hardcoded List (`allModels`)

A constant array of 43 model definitions is compiled directly into the source. Each entry includes:

```ts
{
  id: "gpt-5.4-nano",
  name: "GPT-5.4 nano",
  reasoning: true,
  input: ["text", "image"],
  cost: { input: 0.2, output: 1.25, cacheRead: 0.02, cacheWrite: 0 },
  contextWindow: 400000,
  maxTokens: 128000
}
```

**All models with `cost.input === 0` are free.** Paid models have non-zero input costs (e.g., `claude-opus-4-1` has `input: 15`).

Models span: GPT-5.x variants (15 models), Claude (8 models), Gemini (3 models), GLM (3 models), MiniMax (3 models), Kimi (3 models), plus singleton models: `big-pickle`, `trinity-large-preview-free`, `qwen3.6-plus-free`, `nemotron-3-super-free`.

### 1b. Live `models.dev` API (`fetchModelsDevInfo`)

- **URL:** `https://models.dev/api.json`
- **Method:** GET (no auth, no custom headers)
- **Response shape (expected):**
  ```json
  {
    "opencode": {
      "models": {
        "gpt-5.4-nano": {
          "status": "active" | "deprecated",
          "cost": { "input": 0, "output": 1.25, "cache_read": 0.02, "cache_write": 0 }
        }
      }
    }
  }
  ```
- **Purpose:** Overlays the hardcoded list with live status and cost data. Models marked `status: "deprecated"` are filtered out. In public mode, only models where `cost.input === 0` are included.

### 1c. opencode.ai Model Visibility Check (`fetchVisibleModelIds`)

- **URL:** `https://opencode.ai/zen/v1/models`
- **Method:** GET
- **Auth:** `Authorization: Bearer <apiKey>`
- **Response shape (expected):**
  ```json
  {
    "data": [{ "id": "gpt-5.4-nano" }, { "id": "claude-sonnet-4" }, ...]
  }
  ```
- **Purpose:** Cross-references the hardcoded `allModels` list against the server's visible model IDs. If this fetch fails or returns non-OK, all models are shown. If the server returns a subset, only matching models are included.
- **Only called when a real API key is configured** (skipped for public mode or missing key).

---

## 2. Provider Registration with pi-ai Framework

Registration happens in the default export function:

```ts
export default async function (pi: ExtensionAPI): Promise<void> {
  const apiKey = getConfiguredApiKey();
  const [visibleIds, modelsDevInfo] = await Promise.all([
    apiKey ? fetchVisibleModelIds(apiKey) : Promise.resolve(undefined),
    fetchModelsDevInfo(),
  ]);

  pi.registerProvider("opencode-zen", {
    baseUrl: BASE_URL,
    apiKey: API_KEY,
    api: "openai-completions",   // fallback API type
    streamSimple: streamOpencodeZen,
    models: getVisibleModels(visibleIds, modelsDevInfo, isPublicMode(apiKey)),
  });
}
```

The provider name is **`"opencode-zen"`**. It registers:
- **`baseUrl`:** `"https://opencode.ai/zen/v1"` — the root endpoint for all proxied API calls.
- **`apiKey`:** The *env var name* `"OPENCODE_API_KEY"` (not the key value itself — pi-ai likely interpolates the actual key from this name).
- **`api`:** `"openai-completions"` — the fallback streaming backend for models without an explicit endpoint mapping.
- **`streamSimple`:** Custom `streamOpencodeZen` function that routes per-model to the correct backend.
- **`models`:** The filtered model list (see section 5).

---

## 3. Headers Sent

### opencode.ai Custom Headers (`opencodeHeaders()`)

Sent on **every** request to `opencode.ai` (both the model list fetch and the actual LLM streaming requests):

| Header | Value | Purpose |
|--------|-------|---------|
| `User-Agent` | `"opencode/latest/1.3.15/cli"` | Spoofs the official opencode CLI client |
| `x-opencode-client` | `"cli"` | Identifies client type |
| `x-opencode-session` | 26-char hex string (UUID without dashes) | Session tracking |
| `x-opencode-project` | 26-char hex string (UUID without dashes) | Project tracking |
| `x-opencode-request` | 26-char hex string (UUID without dashes) | Per-request tracking |

The UUID generation:
```ts
const id = () => crypto.randomUUID().replace(/-/g, "").slice(0, 26);
```

### Auth Header

- `Authorization: Bearer <apiKey>` — sent only on the `GET /zen/v1/models` endpoint.

### models.dev Request

The `fetchModelsDevInfo()` call to `https://models.dev/api.json` sends **no custom headers** and **no auth** — it is a plain GET.

### Streaming Requests

The custom headers from `opencodeHeaders()` are **merged** (`...opencodeHeaders(), ...options?.headers`) into the streaming request headers via `SimpleStreamOptions`.

---

## 4. Authentication with opencode.ai API

### Key Resolution (`getConfiguredApiKey()`)

The API key is resolved in two layers:

1. **Environment variable:** `process.env["OPENCODE_API_KEY"]` (trimmed).
2. **File-based fallback:** Reads `~/.pi/agent/auth.json`, looks up the key under `auth["opencode-zen"]?.key`:
   ```json
   { "opencode-zen": { "key": "sk-..." } }
   ```

Returns `undefined` if neither source has a key.

### When is auth sent?

- The `Authorization: Bearer <key>` header is sent **only** to `GET /zen/v1/models` to determine which models the user can see.
- The key is **not directly sent** in streaming requests — the `apiKey` property registered with the provider is the env var name `"OPENCODE_API_KEY"`, and pi-ai's runtime presumably injects it into requests to the provider's `baseUrl`.

---

## 5. Free Model Filtering

### Tier 1: Public Mode (no API key)

When `isPublicMode(apiKey)` returns `true` (apiKey is falsy or literally `"public"`):
- The `getVisibleModels` function calls `isFreeModel(modelsDevInfo[m.id])` for each model.
- `isFreeModel`: returns `true` only if the model's `cost.input` is **exactly 0** (from `models.dev` data).
- **Result:** Only models with `cost.input === 0` are shown to the user.

### Tier 2: Authenticated Mode (with API key)

- Models marked `status: "deprecated"` in `models.dev` are removed.
- No cost filtering is applied — all non-deprecated models are shown.

### Tier 3: No models.dev Data Available

- The `modelsDevInfo` filter is skipped entirely — the full `allModels` list (or the visibility-filtered subset) is returned with no cost-based filtering.

### Summary of filtering priority:

| Has API Key? | models.dev available? | Filtering applied |
|---|---|---|
| No / "public" | Yes | Only models with `cost.input === 0` |
| No / "public" | No | All hardcoded models |
| Yes | Yes | Remove deprecated, show all costs |
| Yes | No | Show all models visible to API key |

---

## 6. Public Mode

### Detection

```ts
function isPublicMode(apiKey?: string): boolean {
  return !apiKey || apiKey === "public";
}
```

Public mode is activated when:
- **No API key is configured** (env var not set and auth file not present / has no key), **OR**
- **The API key value is literally the string `"public"`.**

### Behavior Changes in Public Mode

1. **`fetchVisibleModelIds` is skipped** — the `GET /zen/v1/models` call is not made because `apiKey` is falsy.
2. **Free model filtering is applied** — via `isFreeModel(modelsDevInfo[m.id])`, which checks `cost.input === 0`.
3. **The API key sent to the provider** is `"OPENCODE_API_KEY"` (the env var name). In public mode this env var is undefined, so pi-ai will send no auth header to the streaming endpoints.

### Intended Effect

Public mode gives users access to **only free models** (those with zero input cost) without requiring an opencode.ai API key. The `opencode.ai/zen/v1` endpoint apparently serves some models for free even without authentication.

---

## 7. Exact URL Scheme for opencode.ai API

| Purpose | URL | Method | Auth | Headers |
|---------|-----|--------|------|---------|
| Model visibility check | `https://opencode.ai/zen/v1/models` | GET | Bearer token | All `opencodeHeaders()` |
| GPT-5.x / Responses models | `https://opencode.ai/zen/v1` (base) | POST (implied) | Via env var injection | `opencodeHeaders()` merged |
| Claude / Anthropic models | `https://opencode.ai/zen/v1` (base) | POST (implied) | Via env var injection | `opencodeHeaders()` merged |
| Gemini / Google models | `https://opencode.ai/zen/v1` (base) | POST (implied) | Via env var injection | `opencodeHeaders()` merged |
| GLM / MiniMax / Kimi / others | `https://opencode.ai/zen/v1` (base) | POST (implied) | Via env var injection | `opencodeHeaders()` merged |

The base URL `https://opencode.ai/zen/v1` is the single entry point. The actual per-model API paths are determined by the `pi-ai` library's streaming functions (`streamSimpleAnthropic`, `streamSimpleGoogle`, `streamSimpleOpenAIResponses`, `streamSimpleOpenAICompletions`) which append paths like `/messages`, `/chat/completions`, etc. relative to `baseUrl`.

### Model-to-Backend Routing (the `endpoints` map)

| Backend Type (`api` field) | pi-ai stream function | Models |
|---|---|---|
| `openai-responses` | `streamSimpleOpenAIResponses` | All GPT-5.x variants (15 models) |
| `anthropic-messages` | `streamSimpleAnthropic` | `claude-*` (8 models) |
| `google-generative-ai` | `streamSimpleGoogle` | `gemini-*` (3 models) |
| `openai-completions` | `streamSimpleOpenAICompletions` | `glm-*`, `minimax-*`, `kimi-*`, `big-pickle`, `trinity-large-preview-free`, `qwen3.6-plus-free`, `nemotron-3-super-free` |

The fallback is `openai-completions` — if a model is not in the `endpoints` map, the `streamOpencodeZen` function checks `model.provider !== "opencode-zen"` and falls through to `streamSimpleOpenAICompletions`. (For provider `"opencode-zen"` models, a missing endpoint entry would cause an error since the switch has no default.)

---

## Key Design Insights

1. **The `zen/v1` path is a proxy/reverse tunnel** — it speaks multiple API protocols (Anthropic Messages, Google Generative AI, OpenAI Responses, OpenAI Completions) on a single base URL, routing based on the request body format.

2. **`models.dev` is the canonical free-model data source** — it acts as an off-chain registry. The extension caches nothing; it fetches fresh on every load.

3. **The 26-char session/project/request IDs** (UUID without dashes, truncated to 26 chars) are primarily for telemetry/tracking, not security.

4. **The `User-Agent` header impersonates the opencode CLI** (`"opencode/latest/1.3.15/cli"`), likely to pass the server's client version checks.

5. **The `"public"` magic string** as an API key is a deliberate bypass mechanism — the server apparently treats `Bearer public` as an unauthenticated free-tier request.

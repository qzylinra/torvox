# Exploration: `pi-opencode-zen` Plugin

Source: https://raw.githubusercontent.com/ravshansbox/pi-opencode-zen/refs/heads/main/index.ts

---

## 1. Initialization Sequence (`export default async function`)

The plugin entry point is a single default async export that takes an `ExtensionAPI` object. The startup runs these steps in order:

1. **API Key Resolution** — calls `getConfiguredApiKey()`, which tries env var first, then falls back to a JSON config file.
2. **Parallel fetch** — fires two requests concurrently via `Promise.all`:
   - `fetchVisibleModelIds(apiKey)` — only if an API key was found (otherwise resolves to `undefined`)
   - `fetchModelsDevInfo()` — always fires, no auth needed
3. **Provider registration** — calls `pi.registerProvider("opencode-zen", ...)` with:
   - `baseUrl`: `https://opencode.ai/zen/v1`
   - `apiKey`: the env-var name string `"OPENCODE_API_KEY"` (not the resolved value!)
   - `api`: `"openai-completions"` (fallback streaming type)
   - `streamSimple`: the custom `streamOpencodeZen` router function
   - `models`: result of `getVisibleModels(visibleIds, modelsDevInfo, isPublicMode(apiKey))`

---

## 2. Fetching Models from `models.dev`

**URL:** `https://models.dev/api.json`

**Request format:** Plain GET, no auth, no custom headers.

**Response shape expected:**
```ts
{
  opencode?: {
    models?: Record<string, ModelsDevModelInfo>
  }
}
```

Where each model entry has:
```ts
{
  status?: string | null;           // e.g. "deprecated"
  cost?: {
    input?: number | null;          // price per token (dollar)
    output?: number | null;
    cache_read?: number | null;
    cache_write?: number | null;
  } | null;
}
```

**What it extracts:** Only two fields per model — `status` (to filter deprecated) and `cost` (to determine free vs paid). The model names/IDs themselves come from the hardcoded `allModels` array, NOT from this endpoint.

---

## 3. Free vs Paid Model Determination

**Rule (in `isFreeModel`):** A model is **free** if its `cost.input` is `0`. If `cost` is missing entirely, it defaults to **paid** (returns `false`).

```ts
function isFreeModel(model: ModelsDevModelInfo | undefined): boolean {
  const cost = model?.cost;
  if (!cost) return false;
  return (cost.input ?? 0) === 0;
}
```

The `cost` is a record from `models.dev/api.json` nested under `opencode.models.<modelId>.cost`. If the cost object doesn't exist or `input` is null/undefined, the `?? 0` makes it return `true` (free) — so a missing `cost.input` is treated as free.

---

## 4. Public Mode (No API Key)

**Detection:** `isPublicMode(apiKey)` returns `true` when:
- `apiKey` is `undefined` (no key found anywhere)
- `apiKey` is the literal string `"public"`

**Behavior changes in public mode:**

1. `fetchVisibleModelIds` is **skipped entirely** (never called) — the user has no API key with which to authenticate the `/models` endpoint, so there's no point.
2. In `getVisibleModels`, the `visibleIds` filter is `undefined`, so **all models** from `allModels` survive the first filter.
3. The `publicMode` parameter (derived from `isPublicMode`) causes an **additional filter**: only models where `modelsDevInfo` reports `cost.input === 0` (free models) are included. Models not present in `modelsDevInfo` at all are excluded.

**Result:** A user with no key (or `"public"`) sees only the subset of hardcoded models that are also listed in `models.dev` with zero input cost.

---

## 5. Model Registration with the Provider

`getVisibleModels()` returns `ProviderModelConfig[]`. The pipeline:

1. **Filter by visible IDs** (if `visibleIds` is set): intersect `allModels` with the set of model IDs returned from `GET /zen/v1/models` (authenticated). This lets the backend control which models the user can access.
2. **Filter by status** (if `modelsDevInfo` is present): remove models whose `status === "deprecated"`.
3. **Filter by free** (if `publicMode` is `true`): keep only models where `isFreeModel` returns `true`.
4. **Map** each remaining model to `ProviderModelConfig`:
   - `id`, `name`, `reasoning`, `cost`, `contextWindow`, `maxTokens` — copied verbatim from `allModels`
   - `input` — filtered to only `"text"` or `"image"` (drops `"video"`, `"audio"`, `"pdf"` as the provider model config type only accepts "text" | "image")

The registered provider is named `"opencode-zen"`. The `apiKey` field passed to `registerProvider` is set to the env-var name string `"OPENCODE_API_KEY"` (not the resolved value) — the framework presumably reads the env var itself at request time.

---

## 6. Model ID → API Backend Mapping

The `endpoints` record maps each model ID string to a `{ api: Backend, baseUrl: string }`:

| Backend | Models |
|---|---|
| `anthropic-messages` | `claude-opus-4-6`, `claude-opus-4-5`, `claude-opus-4-1`, `claude-sonnet-4-6`, `claude-sonnet-4-5`, `claude-sonnet-4`, `claude-haiku-4-5`, `claude-3-5-haiku` |
| `google-generative-ai` | `gemini-3.1-pro`, `gemini-3-pro`, `gemini-3-flash` |
| `openai-responses` | All `gpt-5.*` variants (14 models) |
| `openai-completions` | GLM, MiniMax, Kimi, big-pickle, trinity, qwen, nemotron (14 models) |

Every endpoint uses the same `baseUrl`: `https://opencode.ai/zen/v1`. The backend just selects which protocol format the proxy expects (Anthropic-style, Google-style, OpenAI Responses API, or OpenAI Completions API).

If a model ID is NOT found in the `endpoints` map, and the model's provider is `"opencode-zen"`, the fallback in `streamOpencodeZen` is `streamSimpleOpenAICompletions` (with the original un-wrapped model).

---

## 7. Custom Headers (`opencodeHeaders` function)

Sends 5 headers on every request:

```
User-Agent: opencode/latest/1.3.15/cli
x-opencode-client: cli
x-opencode-session: <26-char hex string>       # crypto.randomUUID() with dashes stripped, truncated
x-opencode-project: <26-char hex string>       # same generation
x-opencode-request: <26-char hex string>       # same generation
```

**Generation:** Each call to `opencodeHeaders()` generates **three new unique IDs** (session, project, request) using `crypto.randomUUID()` → strip dashes → first 26 chars. This means every request gets fresh identifiers — they are NOT reused across calls.

These headers are merged into the streaming request options:
```ts
const wrappedOptions = {
  ...options,
  headers: { ...opencodeHeaders(), ...options?.headers },
};
```

Note: `options.headers` takes precedence over the auto-generated ones, so callers can override.

---

## 8. Streaming Routing (`streamOpencodeZen`)

This is the `streamSimple` callback that the pi-coding-agent framework calls for `"opencode-zen"` provider models.

**Decision logic:**

```
1. Look up model.id in the `endpoints` map
2. If NOT found OR model.provider !== "opencode-zen":
   → Fallback: streamSimpleOpenAICompletions (original model, no wrapping)
3. If found:
   a. Create a `wrappedModel` with the model's props overridden:
      - api → endpoint.api
      - baseUrl → endpoint.baseUrl
   b. Create `wrappedOptions` merging opencodeHeaders with options.headers
   c. Switch on endpoint.api:
      - "anthropic-messages"   → streamSimpleAnthropic(wrappedModel, context, wrappedOptions)
      - "google-generative-ai" → streamSimpleGoogle(wrappedModel, context, wrappedOptions)
      - "openai-responses"     → streamSimpleOpenAIResponses(wrappedModel, context, wrappedOptions)
      - "openai-completions"   → streamSimpleOpenAICompletions(wrappedModel, context, wrappedOptions)
```

The framework's `streamSimple*` functions handle the actual SSE parsing and convert to the unified `AssistantMessageEventStream` type. The plugin's job is purely to select the correct streaming parser for the backend, inject custom headers, and point it at the shared `baseUrl`.

---

## Summary of Key Patterns for Conversion

| Aspect | pi-opencode-zen Pattern |
|---|---|
| Model source | Hardcoded `allModels` array + `models.dev` for status/pricing |
| Auth key sources | `OPENCODE_API_KEY` env var → `~/.pi/agent/auth.json` `["opencode-zen"].key` |
| Public mode | No key or `"public"` → only free models (cost.input === 0) |
| Backend routing | Static `endpoints` record, keyed by model ID string |
| Custom headers | Per-request UUIDs (session, project, request) + fixed User-Agent |
| Streaming dispatch | Wraps model with backend type, delegates to framework's `streamSimple*` |
| API key in registration | Passes env-var NAME string, not resolved value |
| Model filtering order | visibleIds (server-whitelist) → deprecated (status field) → free-only in public mode |

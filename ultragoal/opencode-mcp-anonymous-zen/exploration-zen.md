# Exploration: OpenCode Zen Anonymous Mode (Free Models Without API Key)

## Conclusion

OpenCode Zen supports anonymous/public mode by setting the API key to `"public"`. This is an officially supported approach — when no valid API key is provided, Zen only exposes free models (`cost.input === 0`). The pi-opencode-zen extension demonstrates this pattern, and the same approach works natively in OpenCode.

## Sources

- https://opencode.ai/docs/zen/ — Official Zen documentation
- https://github.com/ravshansbox/pi-opencode-zen/blob/main/index.ts — pi-opencode-zen source code
- https://github.com/VcDoc/zen-free-models — Community free model sync tool
- Project `my-opencode-config` — Auto-configuration wizard
- Community Discord: using API key `"public"` for anonymous access

## How pi-opencode-zen Implements Anonymous Mode

The extension source (`index.ts`) reveals the key architecture:

### Authentication

```typescript
const API_KEY = "OPENCODE_API_KEY";

function getConfiguredApiKey(): string | undefined {
  const env = process.env[API_KEY]?.trim();
  if (env) return env;
  // Falls back to reading ~/.pi/agent/auth.json
}

function isPublicMode(apiKey?: string): boolean {
  return !apiKey || apiKey === "public";
}
```

### Free Model Filtering

```typescript
function isFreeModel(model: ModelsDevModelInfo | undefined): boolean {
  const cost = model?.cost;
  if (!cost) return false;
  return (cost.input ?? 0) === 0;
}

function getVisibleModels(visibleIds, modelsDevInfo, publicMode = false) {
  // Filters out deprecated models
  // In public mode: only keeps models where cost.input === 0
}
```

The code fetches two data sources:
1. `https://opencode.ai/zen/v1/models` (with `Authorization: Bearer public`) — visible model IDs
2. `https://models.dev/api.json` — pricing/deprecation metadata

### Mimicking CLI Behavior

```typescript
function opencodeHeaders(): Record<string, string> {
  return {
    "User-Agent": "opencode/latest/1.3.15/cli",
    "x-opencode-client": "cli",
    "x-opencode-session": id(),
    "x-opencode-project": id(),
    "x-opencode-request": id(),
  };
}
```

## Current Free Models (from official pricing page)

| Model ID | Name |
|---|---|
| `big-pickle` | Big Pickle (stealth model) |
| `deepseek-v4-flash-free` | DeepSeek V4 Flash Free |
| `mimo-v2.5-free` | MiMo-V2.5 Free |
| `north-mini-code-free` | North Mini Code Free |
| `nemotron-3-ultra-free` | Nemotron 3 Ultra Free |

Note: Free models are "available for a limited time" for feedback collection.

## How to Use in OpenCode

### Method 1: TUI /connect (easiest)

Run `/connect` in TUI, select OpenCode Zen, and when prompted for API key, just press Enter (empty key). OpenCode will automatically use anonymous mode.

### Method 2: Environment Variable

```bash
export OPENCODE_API_KEY=public
opencode
```

### Method 3: Config File

```json
{
  "provider": {},
  "model": "opencode/big-pickle"
}
```

The model ID format is `opencode/<model-id>`.

### Method 4: Automated Setup

```bash
npx my-opencode-config
```

This wizard auto-configures OpenCode with free Zen models + Gemini free tier.

## Zen API Endpoints

- Model list: `GET https://opencode.ai/zen/v1/models` (Authorization: Bearer public)
- Chat completions: `POST https://opencode.ai/zen/v1/chat/completions` (OpenAI-compatible)
- Responses API: `POST https://opencode.ai/zen/v1/responses`
- Anthropic messages: `POST https://opencode.ai/zen/v1/messages`
- Model metadata: `GET https://models.dev/api.json`

## Community Confirmation

Per VcDoc/zen-free-models README:
> "if you do not use a Zen API key and are not logged into the Zen provider, OpenCode only surfaces the free Zen models"

Per Discord community:
> "choose opencode zen as provider, set api key to 'public', enjoy free glm5 and minimax2.5"

## Privacy Note

Free models may have different data retention policies:
- Big Pickle: data used to improve model during free period
- DeepSeek V4 Flash Free: data used to improve model
- North Mini Code Free: data retained and used to improve model
- Nemotron 3 Ultra Free: trial use only, logged for security

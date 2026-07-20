# openclaude Configuration Analysis

**Source**: https://github.com/Gitlawb/openclaude (MIT license, derived from Claude Code)

---

## 1. CLI Flags

Defined in `src/main.tsx` (Commander.js) and `src/cli/`.

| Flag | Description |
|------|-------------|
| `[prompt]` | Positional: prompt string |
| `-p, --print` | Non-interactive mode, print response and exit |
| `-d, --debug [filter]` | Enable debug logging, optional category filter |
| `-d2e, --debug-to-stderr` | Debug mode to stderr (hidden) |
| `--debug-file <path>` | Write debug logs to file |
| `--verbose` | Override verbose mode from config |
| `--heartbeat <duration>` | Liveness heartbeat for `--print` (e.g. `30s`, `2m`) |
| `-c, --continue` | Resume most recent conversation in current dir |
| `--resume <session-id>` | Resume specific conversation by session ID |
| `--fork-session` | Branch history into new session ID |
| `--bg <prompt>` | Run non-interactive background session |
| `--bg --name <name> <prompt>` | Named background session |
| `--model <model>` | Override model (provider-scoped) |
| `--provider-env-file <path>` | Load provider env vars from file |
| `--provider <provider>` | Select provider (ollama, openai, gemini, codex, etc.) |
| `--settings <file>` | Load settings from file |
| `--setting <key=value>` | Inline setting override |
| `--allowed-tools <tools>` | Comma-separated allowlisted tool names |
| `--dangerously-skip-permissions` | Skip permission prompts |
| `--permission-mode <mode>` | `default`, `auto`, `bypass`, `restricted` |
| `--version` | Print version |
| Subcommands: `mcp`, `plugin`, `doctor`, `auth`, `open` | Various subcommand trees |

Defined in `src/cli/bg.ts`:
| Flag | Description |
|------|-------------|
| `ps` | List background sessions |
| `logs <id-or-name>` | Show logs for session |
| `logs <id-or-name> -f` | Follow logs |
| `kill <id-or-name>` | Terminate session |
| `attach <id-or-name>` | Point to logs (no full reattach yet) |

---

## 2. Provider Selection Flags

### `CLAUDE_CODE_USE_OPENAI=1`

Set this env var to route through the OpenAI-compatible shim (`src/services/api/openaiShim.ts`). Reads `OPENAI_*` env vars. The shim translates Anthropic SDK calls into OpenAI chat completions (`/v1/chat/completions`) or Responses API (`/v1/responses`).

### `CLAUDE_CODE_USE_GEMINI=1`

Routes to Gemini via the OpenAI shim but with Gemini-specific auth (`x-goog-api-key` or Bearer token from `GEMINI_API_KEY`). Reads `GEMINI_API_KEY` / `GOOGLE_API_KEY`, `GEMINI_MODEL`, `GEMINI_BASE_URL`.

### `CLAUDE_CODE_USE_MISTRAL=1`

Routes to Mistral AI via OpenAI shim. Reads `MISTRAL_API_KEY`, `MISTRAL_MODEL`, `MISTRAL_BASE_URL`.

### `CLAUDE_CODE_USE_GITHUB=1`

Routes to GitHub Copilot API or GitHub Models. Reads `GITHUB_TOKEN`/`GH_TOKEN`, `GITHUB_ENTERPRISE_URL`.

### `CLAUDE_CODE_USE_BEDROCK=1`

AWS Bedrock (Anthropic-native). Uses `@anthropic-ai/bedrock-sdk`.

### `CLAUDE_CODE_USE_VERTEX=1`

Claude on Vertex AI. Uses `google-auth-library` ADC.

### `CLAUDE_CODE_USE_FOUNDRY=1`

Azure Foundry via `@anthropic-ai/foundry-sdk`.

### No env var (default)

Anthropic first-party API with `ANTHROPIC_API_KEY`.

---

## 3. Provider Env Vars

### OpenAI-compatible (all require `CLAUDE_CODE_USE_OPENAI=1`)

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | API key (optional for local models; comma-separated enables rotation) |
| `OPENAI_API_KEYS` | Comma-separated key pool, takes precedence over `OPENAI_API_KEY` |
| `OPENAI_BASE_URL` | Base URL (default: `https://api.openai.com/v1`) |
| `OPENAI_API_BASE` | Compatibility alias for `OPENAI_BASE_URL` |
| `OPENAI_MODEL` | Model name |
| `OPENAI_AUTH_HEADER` | Custom auth header name (e.g. `api-key`) |
| `OPENAI_AUTH_HEADER_VALUE` | Custom auth header value |
| `OPENAI_AUTH_SCHEME` | Auth scheme: `bearer` (default) or `raw` |
| `OPENAI_API_FORMAT` | Request format: `chat_completions` or `responses` |
| `OPENAI_AZURE_STYLE` | Force Azure deployment URL + `api-key` header |
| `AZURE_OPENAI_API_VERSION` | API version for Azure (default: `2024-12-01-preview`) |
| `API_TIMEOUT_MS` | Response headers deadline (default: 600000 = 10 min) |

### Per-provider credential env vars (auto-mapped to `OPENAI_API_KEY`)

| Variable | Provider |
|----------|----------|
| `GROQ_API_KEY` | Groq |
| `FIREWORKS_API_KEY` | Fireworks AI |
| `MIMO_API_KEY` | Xiaomi MiMo |
| `OPENCODE_API_KEY` | OpenCode Zen / Go |
| `OPENGATEWAY_API_KEY` | Gitlawb Opengateway |
| `CLINE_API_KEY` | ClinePass |
| `NEARAI_API_KEY` | NEAR AI |
| `CLOUDFLARE_API_TOKEN` | Cloudflare Workers AI |
| `TOGETHER_API_KEY` | Together AI |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `OPENROUTER_API_KEY` | OpenRouter |
| `BANKR_API_KEY` / `BNKR_API_KEY` | Bankr.bot |
| `HICAP_API_KEY` | Hicap |
| `AIMLAPI_API_KEY` | AI/ML API |

### Special env vars

| Variable | Purpose |
|----------|---------|
| `CLAUDE_CODE_OPENAI_CONTEXT_WINDOWS` | JSON map: model → context window |
| `CLAUDE_CODE_OPENAI_MAX_OUTPUT_TOKENS` | JSON map: model → max output tokens |
| `OPENCLAUDE_OLLAMA_NUM_CTX` | Ollama context window override (default: 32768) |
| `OLLAMA_CONTEXT_LENGTH` | Fallback Ollama context length |
| `OPENCLAUDE_LOCAL_FAST_PATH` | Force local fast-path optimizations (`1`/`0`) |
| `OPENCLAUDE_MAX_RETRIES` | Max retry attempts (default: 10, cap: 100) |
| `OPENCLAUDE_RETRY_DELAY_MS` | Base retry delay (default: 500) |
| `OPENCLAUDE_QUERY_HARD_MAX_MS` | Foreground query hard max (default: 30 min) |
| `OPENCLAUDE_SAFETY_LEVEL` | `strict`, `balanced`, `permissive` |
| `ANTHROPIC_BASE_URL` | Custom Anthropic-compatible base URL |
| `ANTHROPIC_AUTH_TOKEN` | Bearer token for custom Anthropic endpoint |
| `ANTHROPIC_CUSTOM_HEADERS` | Extra headers for custom Anthropic endpoint |

---

## 4. Model Discovery

### Process (in `src/services/api/providerConfig.ts` and `src/integrations/`):

1. **Built-in catalog**: Route-specific model catalogs registered in `integrations/registry.ts`. Each route (OpenAI, DeepSeek, Groq, Fireworks, etc.) has a list of known models with capabilities.

2. **`GET /v1/models` discovery**: For OpenAI-compatible providers, the shim fetches `GET {BASE_URL}/models` to list available models (unauthenticated for some providers like Ollama, authenticated via Bearer for others). The response is cached.

3. **Route alias resolution**: Model names like `codexplan` / `codexspark` are aliases defined in `providerConfig.ts` that resolve to real model IDs (e.g. `codexplan` → `gpt-5.5` with `reasoningEffort: high`). `?reasoning=` and `?thinking=` query params further modify behavior.

4. **Catalog + discovery merge**: Discovery results are merged with catalog entries. Built-in catalog entries take precedence over discovery for known models (the catalog has richer metadata like context window, max output tokens, capability flags).

5. **`modelLimits` override**: Users can pin context window / max output tokens via `CLAUDE_CODE_OPENAI_CONTEXT_WINDOWS`/`CLAUDE_CODE_OPENAI_MAX_OUTPUT_TOKENS` env vars or `modelLimits` in `settings.json`. Precedence: exact env var override > built-in catalog > discovery cache > prefix env var > `modelLimits` > descriptor default.

### For Ollama specifically:

OpenClaude uses Ollama's native chat API (`/api/chat`) rather than `/v1/chat/completions` when it detects an Ollama endpoint (port `11434`, hostname contains `ollama`). This allows sending `options.num_ctx` with each request (default 32768 tokens). The native response is translated to the OpenAI streaming format.

### For local providers:

The shim detects local providers (loopback `127.0.0.0/8`, RFC1918, `.local`, ULA/LLA) and enables fast-path optimizations: skips stable serialization, strict tool-schema normalization, and tool-result compression.

---

## 5. `--provider-env-file`

### How it works (src/utils/envFile.ts):

1. **Parsing**: `parseProviderEnvFileArgs(argv)` extracts `--provider-env-file <path>` from raw CLI args (repeatable).

2. **Loading**: `loadEnvFile(filePath)` reads a `.env`-format file, parses `KEY=VALUE` (with `"`, `'` quote support, `export` prefix, `#` comments).

3. **Security**: Only ~100+ pre-approved env keys are allowed (`ALLOWED_ENV_FILE_KEYS` set in `envFile.ts`). If an unsupported variable is present, the file load fails with an error.

4. **Existing-process-env wins**: File values only apply when `process.env[key] === undefined`. Existing environment takes precedence.

5. **Reapplication**: `rememberLoadedEnvFileValues()` stashes the loaded values. `reapplyRememberedEnvFileValues()` re-applies them after every settings/profile env merge, so explicit CLI input is never overwritten by settings.

6. **Integration with provider flag**: `reapplyRememberedProviderFlag()` runs after env file reapplication, keeping `--provider <name>` as the highest-precedence selection.

### Allowed keys include:

Provider selection: `CLAUDE_CODE_USE_OPENAI`, `CLAUDE_CODE_USE_GEMINI`, `CLAUDE_CODE_USE_MISTRAL`, `CLAUDE_CODE_USE_GITHUB`, `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `CLAUDE_CODE_USE_FOUNDRY`.

Auth keys: `OPENAI_API_KEY`, `OPENAI_API_KEYS`, `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `GEMINI_API_KEY`, `GOOGLE_API_KEY`, `MISTRAL_API_KEY`, `GITHUB_TOKEN`, `GH_TOKEN`, `GROQ_API_KEY`, `FIREWORKS_API_KEY`, `MIMO_API_KEY`, `OPENCODE_API_KEY`, `OPENGATEWAY_API_KEY`, `CLINE_API_KEY`, `NEARAI_API_KEY`, `CLOUDFLARE_API_TOKEN`, `TOGETHER_API_KEY`, `DEEPSEEK_API_KEY`, `OPENROUTER_API_KEY`, `HICAP_API_KEY`, `AIMLAPI_API_KEY`, `AZURE_OPENAI_API_KEY`, `CODEX_API_KEY`, `XAI_API_KEY`.

URL/format: `OPENAI_BASE_URL`, `OPENAI_API_BASE`, `OPENAI_AUTH_HEADER`, `OPENAI_AUTH_HEADER_VALUE`, `OPENAI_AUTH_SCHEME`, `OPENAI_API_FORMAT`, `OPENAI_AZURE_STYLE`, `AZURE_OPENAI_API_VERSION`, `ANTHROPIC_BASE_URL`, `ANTHROPIC_VERTEX_*`.

Model: `OPENAI_MODEL`, `MISTRAL_MODEL`, `GEMINI_MODEL`, `ANTHROPIC_MODEL`, `CLAUDE_CODE_OPENAI_CONTEXT_WINDOWS`, `CLAUDE_CODE_OPENAI_MAX_OUTPUT_TOKENS`, `OPENCLAUDE_OLLAMA_NUM_CTX`.

---

## 6. Model List UI

### `/model` command (interactive):

- Shows a picker of available models from the provider's catalog + discovered models.
- `providerProfileModelPickerMode` setting controls behavior:
  - `auto` (default): Single-model profiles show provider catalog; multi-model profiles show profile list; native vendor routes keep full catalog.
  - `provider`: Show provider catalog/discovery first, append profile-only custom model IDs.
  - `profile`: Show only explicitly configured profile models.
- When multiple provider profiles exist, `/model` also lists models from inactive profiles, grouped under profile name. Selecting one activates that profile + switches model in one step.
- Cross-profile entries only appear in interactive `/model` — not in SDK/automation callers.

### `/provider` command:

- Guided provider setup wizard
- Saves provider profiles to `.openclaude-profile.json` (user-level config directory)
- Supports Ollama, OpenAI, Gemini, Codex, Atomic Chat, and others

### Profile launchers (bun scripts):

```bash
bun run dev:profile      # Launch with saved profile
bun run dev:openai       # OpenAI profile
bun run dev:ollama       # Ollama profile (localhost:11434, llama3.1:8b)
bun run dev:gemini       # Gemini profile
bun run dev:codex        # Codex profile
bun run dev:atomic-chat  # Atomic Chat (Apple Silicon local LLMs)
bun run profile:init -- --provider ollama --model llama3.1:8b
```

---

## 7. Auth Headers

### In `openaiShim.ts` (the `makeOpenAIRequest` function):

1. **Standard**: `Authorization: Bearer {OPENAI_API_KEY}`

2. **Custom auth**: If `OPENAI_AUTH_HEADER` is set, uses that header name instead of `Authorization`. If `OPENAI_AUTH_SCHEME` is `raw`, sends `{header}: {value}` directly. If `bearer`, sends `{header}: Bearer {value}`.

3. **Azure**: When `isAzureStyleBaseUrl()` returns true (hostname matches `*.openai.azure.com`, `*.cognitiveservices.azure.com`, `*.services.ai.azure.com`, `*.inference.ml.azure.com`, or `OPENAI_AZURE_STYLE=1`), sends `api-key: {OPENAI_API_KEY}` header and uses deployment URL: `{base}/openai/deployments/{model}/chat/completions?api-version={version}`.

4. **GitHub Copilot**: Sends `Authorization: Bearer {GITHUB_TOKEN|GH_TOKEN}` with additional `Copilot-Integration-Id`, `Editor-Version`, `User-Agent` headers. On 401, refreshes Copilot token.

5. **Gemini**: Sends `x-goog-api-key: {GEMINI_API_KEY}` or `Authorization: Bearer {access_token}` depending on auth mode.

6. **CredentialPool**: `OPENAI_API_KEYS` (comma-separated) creates a key pool. On auth/quota/rate-limit failures, the shim rotates to the next key with a 30-second cooldown. Handles up to 100 retries (configurable).

7. **Codex**: Reads credentials from `CODEX_API_KEY` env var, `~/.codex/auth.json`, or secure storage. Routes to `chatgpt.com/backend-api/codex`. Supports token refresh on expiry.

8. **Route credential resolution**: `resolveRouteCredentialValue()` maps base URL patterns to credential env vars (e.g., `api.xiaomimimo.com` → `MIMO_API_KEY`, `api.together.xyz` → `TOGETHER_API_KEY`). If the mapped var exists and `OPENAI_API_KEY` is unset, it's auto-assigned.

9. **Anthropic custom endpoint** (non-OpenAI): Sent as `Authorization: Bearer {ANTHROPIC_AUTH_TOKEN}`. Uses `x-api-key` header when `ANTHROPIC_API_KEY` is set instead.

10. **Custom extra headers**: `ANTHROPIC_CUSTOM_HEADERS` allows injecting arbitrary HTTP headers. Anthropic-specific headers (`x-anthropic-*`, `anthropic-*`) are filtered out in the OpenAI shim. The shim also strips `authorization`, `x-api-key`, and `api-key` from forwarded custom headers.

### Replay/non-replay marking:

Transport errors are classified (`classifyOpenAINetworkFailure`) into retryable and non-retryable. Non-retryable errors (4xx auth failures, invalid URLs) are marked with `markOpenAIRequestNonReplayable()`. Retryable errors go through the retry loop with exponential backoff + `Retry-After` header support.

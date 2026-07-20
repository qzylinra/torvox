# OpenClaude Provider Profile System — Exploration Report

**Source**: `/usr/local/lib/node_modules/@gitlawb/openclaude/`
**Analyzed**: 2026-07-20
**Version**: from installed npm package `@gitlawb/openclaude`

---

## 1. Binary Structure

The entry point `/usr/local/bin/openclaude` is a **symbolic link** to:

```
/usr/local/lib/node_modules/@gitlawb/openclaude/bin/openclaude
```

### Launcher (`bin/openclaude` — 3,922 bytes)

A thin Node.js launcher (124 lines) that:

1. **Reads the `--max-memory=<MB>` flag** and stores it in `OPENCLAUDE_NODE_MAX_OLD_SPACE_SIZE_MB` / `OPENCLAUDE_MAX_MEMORY_MB`.
2. **Relaunches itself** with `--max-old-space-size=<N>` (default 8192 MB) and `--expose-gc` if not already present, to support long interactive sessions.
3. **Imports `dist/cli.mjs`** (the real application) via dynamic `import()`.
4. Uses `OPENCLAUDE_HEAP_RELAUNCHED=1` to detect/avoid infinite relaunch loops.

### Built Bundle (`dist/cli.mjs` — 21,963,512 bytes / ~22 MB)

This is the bundled production application. All provider profile logic lives here.

---

## 2. Provider Profile System Overview

The provider profile system is responsible for resolving which LLM provider to use and what environment variables to set. It has three main sources of configuration, in **descending priority**:

| Priority | Source | Description |
|----------|--------|-------------|
| 1 (highest) | **Process env vars** | `CLAUDE_CODE_USE_OPENAI=1`, `OPENAI_BASE_URL=...`, etc. |
| 2 | **Profile file** | `~/.openclaude/.openclaude-profile.json` (persisted via `/provider`) |
| 3 (lowest) | **Default fallback** | `gitlawb-opengateway` gateway as startup default |

---

## 3. Profile File Structure

**Filename**: `.openclaude-profile.json` (constant `PROFILE_FILE_NAME`)

### JSON Schema (validated by `readProfileFile()`):

```json
{
  "profile": "opencode",
  "env": {
    "OPENAI_BASE_URL": "https://opencode.ai/zen/v1",
    "OPENAI_MODEL": "gpt-5.4",
    "OPENCODE_API_KEY": "sk-..."
  },
  "createdAt": "2026-07-20T12:00:00.000Z"
}
```

### Validation rules:
- `profile` must be one of the valid provider names (checked by `isProviderProfile()`)
- `env` must exist and be an object
- If `createdAt` is missing, it defaults to `new Date().toISOString()`

### Valid profile names (`isProviderProfile()`):

```
anthropic, openai, ollama, codex, gemini, atomic-chat, nvidia-nim,
minimax, mistral, github, github-enterprise, bedrock, vertex, xai, opencode
```

---

## 4. Config Directory Resolution

### `OPENCLAUDE_CONFIG_DIR` env var

The config directory is resolved via `getClaudeConfigHomeDir()`:

```
getClaudeConfigHomeDir()
  ├── if claudeConfigHomeDirOverride is set → use it directly
  ├── if $OPENCLAUDE_CONFIG_DIR is set → use resolveClaudeConfigHomeDir({configDirEnv})
  └── else → use getDefaultClaudeConfigHomeDir() → ~/.openclaude/
```

`resolveConfigDirEnv()` simply returns `options.openClaudeConfigDir || void 0` (the second and third parameters `legacyConfigDir` and `warn` are read but their values are discarded — only `openClaudeConfigDir` matters).

The legacy `CLAUDE_CONFIG_DIR` env var is also passed in some call sites but is **no longer used** in `resolveConfigDirEnv()` — only `OPENCLAUDE_CONFIG_DIR` takes effect.

`resolveClaudeConfigHomeDir()`:
- If `configDirEnv` is provided, uses it directly (normalized)
- Otherwise, joins `homeDir` + `.openclaude`

---

## 5. Profile File Path Resolution

`resolveProfileFileReadPaths(options)` determines where to read the profile from:

```
resolveProfileFilePaths(options):
  ├── if options.filePath is set → use it directly
  ├── if options.cwd is set AND options.configDir is NOT set → cwd/.openclaude-profile.json
  └── else → (options.configDir ?? getClaudeConfigHomeDir())/.openclaude-profile.json
```

Then `resolveProfileFileReadPaths`:
1. Gets the primary path from `resolveProfileFilePath(options)`
2. If `filePath` or `cwd`-based (without configDir) → returns just `[primary]`
3. If primary exists → returns `[primary]`
4. Otherwise → also checks `cwd/.openclaude-profile.json` as legacy fallback → returns `[primary, legacy]`

### Summary of path resolution behavior:

| Scenario | Primary Path | Fallback |
|----------|-------------|----------|
| No env, no profile file | `~/.openclaude/.openclaude-profile.json` | `cwd/.openclaude-profile.json` |
| `OPENCLAUDE_CONFIG_DIR=/custom/path` | `/custom/path/.openclaude-profile.json` | `cwd/.openclaude-profile.json` |
| Explicit `filePath` | `filePath` (exact) | none |
| `cwd`-based call | `cwd/.openclaude-profile.json` | none |

---

## 6. `readProfileFile()` Implementation

```javascript
function readProfileFile(filePath) {
  if (!existsSync4(filePath)) return null;
  try {
    let parsed = JSON.parse(readFileSync8(filePath, "utf8"));
    if (!isProviderProfile(parsed.profile) || !parsed.env || typeof parsed.env !== "object")
      return null;
    return {
      profile: parsed.profile,
      env: parsed.env,
      createdAt: typeof parsed.createdAt === "string" ? parsed.createdAt : new Date().toISOString()
    };
  } catch {
    return null;
  }
}
```

Returns `null` (silently) if:
- File does not exist
- JSON parse fails
- `profile` is not a recognized provider name
- `env` is missing or not an object

---

## 7. The "opencode" Profile — In Detail

### Two Gateway Variants

| Variant | Gateway ID | Base URL | Default Model | Description |
|---------|-----------|----------|---------------|-------------|
| `opencode` | `opencode` | `https://opencode.ai/zen/v1` | `gpt-5.4` | OpenCode Zen — pay-as-you-go, 48 models |
| `opencode-go` | `opencode-go` | `https://opencode.ai/zen/go/v1` | `glm-5.1` | OpenCode Go — $10/mo subscription, 13 models |

Both are defined as **OpenAI-compatible gateway routes** with:
- `vendorId: "openai"`
- API key env var: `OPENCODE_API_KEY`
- Model env var: `OPENAI_MODEL`
- Credential fallback: also accepts `OPENAI_API_KEYS` / `OPENAI_API_KEY`

### Env Building for `selectedProfile === "opencode"`

When the "opencode" profile is selected, the following env is built:

```javascript
{
  OPENAI_BASE_URL: processEnv.OPENAI_BASE_URL
               || persistedEnv.OPENAI_BASE_URL
               || "https://opencode.ai/zen/v1",

  OPENAI_MODEL: shellOpenAIModel
             || persistedOpenAIModel
             || "gpt-5.4",

  // If OPENCODE_API_KEY is set:
  OPENAI_API_KEY: processEnv.OPENCODE_API_KEY
               || persistedEnv.OPENCODE_API_KEY
}
```

If no `OPENCODE_API_KEY` is found, falls back to the general OpenAI credential resolver (`resolveOpenAICredentialEnvOverride`), which checks `OPENAI_API_KEYS` and `OPENAI_API_KEY`.

### Profile Application via `buildCompatibilityProcessEnv()`

```javascript
function buildCompatibilityProcessEnv(options) {
  let env = { ...options.processEnv ?? process.env };
  let nextEnv = { ...options.profileEnv };
  let flag = getCompatibilityProfileFlag(options.compatibilityMode); // "openai" → "CLAUDE_CODE_USE_OPENAI"
  if (flag) nextEnv[flag] = "1";
  return applyProfileEnvToProcessEnv(env, nextEnv), env;
}
```

This:
1. Clones the current process env
2. Merges profile-specific env vars on top
3. Adds `CLAUDE_CODE_USE_OPENAI=1`
4. Calls `applyProfileEnvToProcessEnv()` which:
   - **Clears** all `PROFILE_ENV_KEYS` from the target
   - **Assigns** all new env vars from `nextEnv`

### `applyProfileEnvToProcessEnv()`

```javascript
function applyProfileEnvToProcessEnv(targetEnv, nextEnv) {
  clearManagedProfileEnv(targetEnv);  // deletes all PROFILE_ENV_KEYS
  Object.assign(targetEnv, nextEnv);  // applies new values
}
```

This is the key function. When a profile is applied:
1. All previously managed env vars are **deleted** (to avoid stale values)
2. New values are **assigned** onto the process environment

---

## 8. Env Var Profiles — Explicit Selection via Process Env

### Support env vars for explicit provider selection:

| Env Var | Profile Name | Compat Mode |
|---------|-------------|-------------|
| `CLAUDE_CODE_USE_OPENAI` | `openai` | `openai` |
| `CLAUDE_CODE_USE_GITHUB` | `github` | `github` |
| `CLAUDE_CODE_USE_BEDROCK` | `bedrock` | `bedrock` |
| `CLAUDE_CODE_USE_VERTEX` | `vertex` | `vertex` |
| `CLAUDE_CODE_USE_MISTRAL` | `mistral` | `mistral` |
| `CLAUDE_CODE_USE_GEMINI` | `gemini` | `gemini` |

When any of these is set to a truthy value, `hasExplicitProviderSelection()` returns `true`. The `explicitProfileOverrides` loop iterates through them in order, and the **first match wins** (with a special exception for codex+openai OAuth overlap).

### Markers for "profile already applied":

| Marker | Meaning |
|--------|---------|
| `CLAUDE_CODE_PROVIDER_PROFILE_ENV_APPLIED=1` | Profile has been applied to process env |
| `CLAUDE_CODE_PROVIDER_PROFILE_ENV_APPLIED_ID={id}` | Which profile was applied |

These prevent double-application and allow the system to detect that env vars came from a profile rather than from the user's shell.

### Default startup provider:

```javascript
DEFAULT_STARTUP_PROVIDER_ENV_VAR = "CLAUDE_CODE_DEFAULT_STARTUP_PROVIDER"
```

When no profile is persisted and no explicit provider selection is detected:
1. Check if `OPENCLAUDE_PROFILE_GOAL` hints at Ollama (recommends "ollama")
2. Otherwise, use the `gitlawb-opengateway` default gateway (`https://opengateway.gitlawb.com/v1`, model `mimo-v2.5-pro`)

This default is indicated by setting `CLAUDE_CODE_DEFAULT_STARTUP_PROVIDER=gitlawb-opengateway` in the env.

---

## 9. Complete Env Var Priority for Each Source

For the **opencode** profile specifically, env var resolution priority is:

| Environment Variable | Priority 1 (shell env) | Priority 2 (profile file) | Priority 3 (hardcoded default) |
|---------------------|----------------------|--------------------------|-------------------------------|
| `OPENAI_BASE_URL` | `$OPENAI_BASE_URL` | `persisted.OPENAI_BASE_URL` | `https://opencode.ai/zen/v1` |
| `OPENAI_MODEL` | `$OPENAI_MODEL` | `persisted.OPENAI_MODEL` | `gpt-5.4` |
| `OPENCODE_API_KEY` | `$OPENCODE_API_KEY` | `persisted.OPENCODE_API_KEY` | (none) |
| `OPENAI_API_KEY` | `$OPENAI_API_KEY` | `persisted.OPENAI_API_KEY` | (none) |
| `OPENAI_API_KEYS` | `$OPENAI_API_KEYS` | `persisted.OPENAI_API_KEYS` | (none) |

---

## 10. Profile Application Flow

### `buildStartupEnvFromProfile()` (called at startup):

```
buildStartupEnvFromProfile()
  ├── If CLAUDE_CODE_PROVIDER_PROFILE_ENV_APPLIED=1 already → return processEnv (no-op)
  ├── If nvidia-nim detected via OPENAI_BASE_URL → build openai profile with nvidia route
  ├── If concrete provider selection exists (env vars set) → return processEnv
  ├── If no persisted profile:
  │     ├── If explicit opt-out of OpenAI compat → return processEnv
  │     └── Else → build default gitlawb-opengateway env
  ├── Else (persisted profile exists):
  │     └── buildLaunchEnv({profile: persisted.profile, persisted, ...})
  │         ├── Calls the specific profile builder (e.g., opencode case)
  │         └── Returns merged env with CLAUDE_CODE_USE_* flag set
  │
  └── Final: applyProfileEnvToProcessEnv(processEnv, startupEnv)
      ├── Clears all PROFILE_ENV_KEYS from processEnv
      └── Assigns startupEnv values onto processEnv
```

### `applySavedProfileToCurrentSession()` (called when user switches via `/provider`):

```
applySavedProfileToCurrentSession()
  ├── If codex OAuth + explicit selection → handle special case
  ├── If already managed → update in place
  └── Else → clear all CLAUDE_CODE_USE_* flags, apply profile env
```

---

## 11. Managed Env Vars (`PROFILE_ENV_KEYS`)

The full list of env vars that are **cleared and managed** by the profile system:

```javascript
PROFILE_ENV_KEYS = [
  "CLAUDE_CODE_USE_OPENAI",
  "CLAUDE_CODE_USE_GITHUB",
  "CLAUDE_CODE_USE_GEMINI",
  "CLAUDE_CODE_USE_MISTRAL",
  "CLAUDE_CODE_USE_BEDROCK",
  "CLAUDE_CODE_USE_VERTEX",
  "CLAUDE_CODE_USE_FOUNDRY",
  "ANTHROPIC_BASE_URL",
  "ANTHROPIC_MODEL",
  "ANTHROPIC_API_KEY",
  "ANTHROPIC_CUSTOM_HEADERS",
  "ANTHROPIC_BEDROCK_BASE_URL",
  "ANTHROPIC_VERTEX_BASE_URL",
  "OPENAI_BASE_URL",
  "OPENAI_API_BASE",
  "OPENAI_MODEL",
  "OPENAI_API_FORMAT",
  "OPENAI_AUTH_HEADER",
  "OPENAI_AUTH_SCHEME",
  "OPENAI_AUTH_HEADER_VALUE",
  "OPENAI_API_KEYS",
  "OPENAI_API_KEY",
  "GITHUB_COPILOT_KEY",
  "GITHUB_ENTERPRISE_URL",
  "CLAUDE_CODE_OPENAI_CONTEXT_WINDOWS",
  "CODEX_API_KEY",
  "CODEX_CREDENTIAL_SOURCE",
  "CHATGPT_ACCOUNT_ID",
  "CODEX_ACCOUNT_ID",
  "GEMINI_API_KEY",
  "GEMINI_AUTH_MODE",
  "GEMINI_ACCESS_TOKEN",
  "GEMINI_MODEL",
  "GEMINI_BASE_URL",
  "GOOGLE_API_KEY",
  "NVIDIA_NIM",
  "NVIDIA_API_KEY",
  "NVIDIA_MODEL",
  "MINIMAX_API_KEY",
  "MINIMAX_BASE_URL",
  "MINIMAX_MODEL",
  "MISTRAL_BASE_URL",
  "MISTRAL_API_KEY",
  "MISTRAL_MODEL",
  "BANKR_BASE_URL",
  "BNKR_API_KEY",
  "BANKR_MODEL",
  "XAI_API_KEY",
  "XAI_CREDENTIAL_SOURCE",
  "AIMLAPI_API_KEY",
  "VENICE_API_KEY",
  "MIMO_API_KEY",
  "ATLAS_CLOUD_API_KEY",
  "NEARAI_API_KEY",
  "FIREWORKS_API_KEY",
  "CLINE_API_KEY",
  "OPENCODE_API_KEY",
  "CLAUDE_CODE_PROVIDER_ROUTE_ID",
  "CLOUDFLARE_API_TOKEN",
  "CLAUDE_CODE_DEFAULT_STARTUP_PROVIDER",  // DEFAULT_STARTUP_PROVIDER_ENV_VAR
]
```

---

## 12. How `CLAUDE_CODE_USE_OPENAI` Interacts with the System

Setting `CLAUDE_CODE_USE_OPENAI=1` is the primary mechanism to **explicitly select an OpenAI-compatible provider** at the shell level. The flow:

1. `hasExplicitProviderSelection()` detects `CLAUDE_CODE_USE_OPENAI=1` and returns `true`
2. The `explicitProfileOverrides` loop matches `CLAUDE_CODE_USE_OPENAI` → `"openai"` profile
3. The openai profile builder checks `OPENAI_BASE_URL` to auto-detect which route to use:
   - If base URL matches a known route (e.g., `opencode.ai` → opencode gateway), that route is auto-selected
   - If base URL points to a local server, no API key is needed
   - Otherwise, `OPENAI_API_KEY` or `OPENAI_API_KEYS` must be provided
4. The `hasConcreteProviderSelection()` function validates that sufficient config exists

**Important edge case**: `CLAUDE_CODE_USE_OPENAI=1` but no `OPENAI_BASE_URL` or API key will trigger a validation error:
> "OPENAI_API_KEYS or OPENAI_API_KEY is required when CLAUDE_CODE_USE_OPENAI=1 and OPENAI_BASE_URL is not local."

---

## 13. Summary: Switching Profile via Env Var

**Yes, you can switch the provider profile entirely via env vars** without any profile file:

| Action | Command |
|--------|---------|
| Use OpenAI-compatible provider | `CLAUDE_CODE_USE_OPENAI=1 openclaude` |
| Use GitHub Copilot | `CLAUDE_CODE_USE_GITHUB=1 openclaude` |
| Use Gemini | `CLAUDE_CODE_USE_GEMINI=1 openclaude` |
| Use Anthropic | Set `ANTHROPIC_API_KEY` + `ANTHROPIC_MODEL` (no `CLAUDE_CODE_USE_*` needed) |
| Use OpenCode Zen specifically | `CLAUDE_CODE_USE_OPENAI=1 OPENAI_BASE_URL=https://opencode.ai/zen/v1 OPENCODE_API_KEY=... openclaude` |
| Use OpenCode Go | `CLAUDE_CODE_USE_OPENAI=1 OPENAI_BASE_URL=https://opencode.ai/zen/go/v1 OPENCODE_API_KEY=... openclaude` |

When env vars are set, the profile file is **ignored** (because `hasConcreteProviderSelection()` / `hasExplicitProviderSelection()` returns `true`, short-circuiting the profile file read).

---

## 14. Key Observations

1. **OpenClaude is a fork/derivative of Claude Code** that adds the OpenCode gateways and supports arbitrary OpenAI-compatible providers. The code structure (env var names, provider profile system, config dir conventions) is directly inherited from Claude Code.

2. **The `CLAUDE_CODE_USE_OPENAI` env var is the universal switch** for OpenAI-compatible mode. All OpenAI-compatible providers (OpenCode, Ollama, Atomic Chat, NVIDIA NIM, etc.) are routed through this single flag, differentiated by `OPENAI_BASE_URL`.

3. **`OPENCODE_API_KEY` is separate from `OPENAI_API_KEY`** but both are accepted. The opencode profile checks `OPENCODE_API_KEY` first, then falls back to `OPENAI_API_KEYS` / `OPENAI_API_KEY`.

4. **The profile file is only consulted when no explicit env var selection is made**. At startup, if `CLAUDE_CODE_USE_OPENAI` (or any `CLAUDE_CODE_USE_*`) is already set, the profile file is skipped entirely.

5. **`OPENCLAUDE_CONFIG_DIR` fully replaces `~/.openclaude/`** as the base config directory. When set, ALL config files (profile, settings, credentials, teams, etc.) are resolved relative to that directory.

6. **The profile file format is simple JSON** with three fields: `profile`, `env`, and `createdAt`. The `env` object is a flat map of env var names to values.

7. **Clearing and re-applying**: Every time a profile is applied, all managed env vars are first deleted from the process env, then the new ones are set. This prevents stale values from persisting across profile switches.

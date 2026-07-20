# Exploration: openclaude

**Version**: 0.24.0
**Source**: https://github.com/Gitlawb/openclaude

## Architecture

OpenClaude is a CLI tool (Node.js, built with Bun) for coding-agent workflows with multiple LLM providers. It uses an OpenAI-compatible shim (`openaiShim.ts`) to translate Anthropic SDK calls into OpenAI chat completions.

## Provider Configuration

Three mechanisms exist:

1. **Shell env vars** (avoid): `export CLAUDE_CODE_USE_OPENAI=1 OPENAI_API_KEY=... OPENAI_BASE_URL=... OPENAI_MODEL=...`
2. **`--provider-env-file` flag** (the issue): Loads a `.env`-format file with allowed keys (~85). File specified via `--provider-env-file path/to/file`.
3. **`~/.openclaude.json` config**: Supports `providerProfiles`, `agentModels`, `agentRouting`. The `agentModels` mechanism supports per-agent `model`, `base_url`, `api_key` overrides.

## CLI Flags

```
--provider             Set provider (interactive)
--model                Override model name
--provider-env-file    Load .env file for provider setup (repeatable)
--print / -p           Non-interactive mode
--version              Print version
```

No `--api-key` or `--base-url` CLI flags exist.

## Key constraint: CLAUDE_CODE_USE_OPENAI

The provider resolution checks `CLAUDE_CODE_USE_OPENAI=1` before reading `OPENAI_BASE_URL` / `OPENAI_MODEL`. Without this flag, openclaude uses its built-in Anthropic path.

## Config file (`~/.openclaude.json`)

```typescript
type GlobalConfig = {
  providerProfiles?: ProviderProfile[]
  activeProviderProfileId?: string
  agentModels?: Record<string, { model?: string; base_url?: string; api_key?: string }>
  agentRouting?: Record<string, string>
}
```

No `availableModels` field — this is an Anthropic Claude Code feature not ported to openclaude.

## "OpenCode Zen" in openclaude

Not a dedicated provider plugin. It's just the generic OpenAI-compatible shim pointed at `https://opencode.ai/zen/v1` with `OPENCODE_API_KEY`. The provider name "opencode-zen" in `--provider opencode-zen` refers to the built-in profile path using `OPENCODE_API_KEY`.

## Key insight for adapter

Since openclaude has no `availableModels` and no direct `--api-key`/`--base-url` flags, the adapter must either:
- Use `--provider-env-file` (but constraint says no .env)
- Use `agentModels` in `~/.openclaude.json` with inline `base_url` + `api_key`
- Or create a wrapper script that injects env vars at process level

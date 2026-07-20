# Stage 2: OpenCode Free-Models Setup Script — Implementation

## Script: `setup-free-models.mjs`

### What it does
1. Fetches `https://models.dev/api.json` to get pricing/status for all OpenCode Zen models
2. Filters to active (non-deprecated) models with `cost.input === 0`
3. Lists the free models in a table
4. Creates/updates `.openclaude-profile.json` with OpenCode Zen provider configuration
5. Preserves user's previously selected model if one exists

### Key Design Decisions
- Uses `deepseek-v4-flash-free` as the default model (well-known, fast, 200K context)
- API key defaults to `"public"` (free-tier access)
- Refetches model list each run (models.dev data changes as models are added/deprecated)
- Preserves existing model choice so re-running the script doesn't reset the user's preference

### Test Results
- Models discovered: 6 free models
- Profile created at `~/.openclaude/.openclaude-profile.json`
- Verified openclaude loads the profile and responds via `openclaude -p "Output only the word OK"` → `OK`
- Verified the OpenCode Zen API accepts requests with `Authorization: Bearer public` and standard OpenAI-compatible format

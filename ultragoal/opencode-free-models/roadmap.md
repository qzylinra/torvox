# Roadmap: Convert pi-opencode-zen to openclaude with Dynamic Free Models

## Overall Goal
Convert the pi-opencode-zen VS Code extension provider code to work with openclaude CLI, dynamically fetching the model list from the API and only showing free models (cost.input === 0).

## Source Analysis
- **pi-opencode-zen**: VS Code extension that registers a provider with pi-ai
  - Hardcoded model list with cost information
  - Fetches visible models from `/zen/v1/models` API
  - Fetches model info from `https://models.dev/api.json`
  - Filters models based on whether they're free (cost.input === 0)
  - Routes models to different backends (anthropic-messages, google-generative-ai, openai-responses, openai-completions)

- **openclaude**: CLI tool for coding agents
  - Already has OpenCode Zen/Go support via PR #1350
  - Uses `defineGateway` for gateway descriptors
  - Uses `defineModel` for model descriptors
  - Per-model endpoint routing via `transportOverrides.openaiShim.endpointPath`
  - Static model catalogs (currently)

## Stages

### Stage 1: Create Dynamic Free Models Gateway
**Objective**: Create a new openclaude gateway that dynamically fetches models from the OpenCode Zen API and filters to only show free models.

**Acceptance Criteria**:
1. New gateway file created at `src/integrations/gateways/opencode-free.ts`
2. Gateway dynamically fetches models from `/zen/v1/models` API
3. Gateway fetches cost info from `https://models.dev/api.json`
4. Only models with `cost.input === 0` are included
5. Per-model endpoint routing is preserved (GPT→/responses, Claude→/messages, etc.)
6. All existing tests pass

### Stage 2: Add Model Descriptors for Free Models
**Objective**: Create model descriptors for the free models that will be dynamically discovered.

**Acceptance Criteria**:
1. Model descriptors created for known free models
2. Descriptors include proper context windows, capabilities, and classifications
3. Model descriptors are compatible with the gateway's provider model map

### Stage 3: Integration and Testing
**Objective**: Integrate the new gateway into openclaude and verify it works correctly.

**Acceptance Criteria**:
1. Gateway is registered in the integration artifacts
2. Gateway appears in the `/provider` preset picker
3. Dynamic model fetching works correctly
4. Free model filtering works correctly
5. All existing tests pass

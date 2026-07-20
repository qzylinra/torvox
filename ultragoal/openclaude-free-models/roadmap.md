# Roadmap: openclaude-free-models

## Goal
Convert pi-opencode-zen's approach (dynamic model list fetching + free-model filtering) for openclaude: install openclaude, create a setup script that configures it to dynamically fetch and use only free models from OpenCode Zen, and test.

## Stages

### Stage 1: Install openclaude
**Objective**: Install openclaude CLI globally and verify it works.
**Acceptance Criteria**:
- `openclaude --version` returns a version number
- `openclaude --help` shows help text

### Stage 2: Create OpenCode free-models setup script
**Objective**: Create a script that:
1. Fetches models from OpenCode Zen API (`/models` endpoint + `models.dev/api.json`)
2. Filters to free models (input cost = 0)
3. Generates an openclaude provider configuration for OpenCode Zen
4. Configures openclaude to use free models only
**Acceptance Criteria**:
- Script runs successfully
- Produces valid openclaude configuration
- Only free models are included

### Stage 3: Test end-to-end
**Objective**: Test that openclaude starts with the OpenCode Zen free-models configuration and can access the free model list.
**Acceptance Criteria**:
- openclaude starts with the configuration
- Models are discoverable
- No errors during initialization

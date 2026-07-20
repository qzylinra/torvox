# Implementation Plan: OpenCode Free Models Gateway

## Overview
Create a new openclaude gateway that dynamically fetches models from the OpenCode Zen API and filters to only show free models (cost.input === 0).

## Architecture

### Gateway Structure
The new gateway will follow the same pattern as the existing `opencode.ts` gateway but with:
1. Dynamic model fetching from `/zen/v1/models` API
2. Cost information from `https://models.dev/api.json`
3. Filtering to only include free models (cost.input === 0)
4. Same endpoint routing logic (GPT→/responses, Claude→/messages, etc.)

### Key Components

1. **Model Fetcher**: Fetches available models from the API
2. **Cost Fetcher**: Fetches cost information from models.dev
3. **Free Model Filter**: Filters models where cost.input === 0
4. **Endpoint Router**: Maps models to their correct API endpoints

### Implementation Steps

1. Create `src/integrations/gateways/opencode-free.ts` with:
   - Dynamic model fetching logic
   - Cost information fetching
   - Free model filtering
   - Same endpoint routing as existing gateway

2. Create `src/integrations/models/opencode-free.ts` with:
   - Model descriptors for known free models
   - Proper context windows and capabilities
   - Provider model map for the gateway

3. Update integration artifacts to include the new gateway

### Testing Strategy
- Unit tests for model fetching and filtering
- Integration tests for gateway registration
- Verify existing tests still pass

### Risk Assessment
- **Low Risk**: Building on proven patterns from existing gateway
- **Medium Risk**: Dynamic fetching may fail if API is unavailable (fallback to static list)
- **Low Risk**: Cost filtering logic is straightforward (cost.input === 0)

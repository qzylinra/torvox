# OpenCode Free Models Gateway - Implementation Summary

## Overview
Created a new openclaude gateway that dynamically fetches models from the OpenCode Zen API and filters to only show free models (cost.input === 0).

## Files Created

### 1. Gateway Definition
**File**: `ultragoal/opencode-free-models/opencode-free.ts`

**Features**:
- Dynamic model fetching from `/zen/v1/models` API
- Cost information from `https://models.dev/api.json`
- Filtering to only include free models (cost.input === 0)
- Same endpoint routing as existing OpenCode Zen gateway:
  - GPT models → `/responses`
  - Claude/Qwen models → `/messages`
  - Gemini models → `/models/<id>`
  - Other models → `/chat/completions`

**Gateway Configuration**:
- ID: `opencode-free`
- Label: "OpenCode Free"
- Category: `aggregating`
- Base URL: `https://opencode.ai/zen/v1`
- Auth: `OPENCODE_API_KEY`

### 2. Model Descriptors
**File**: `ultragoal/opencode-free-models/opencode-free-models.ts`

**Free Models Included**:
- Claude 3.5 Haiku (200K context, 8K output, vision)
- GLM 4.6, 4.7, 5 (202K context, 131K output, coding)
- Kimi K2, K2 Thinking, K2.5 (128K-262K context, coding)
- MiniMax M2.1, M2.5, M2.5 Free (204K context, 131K output, coding)
- Big Pickle (200K context, 128K output, reasoning, coding)
- Nemotron 3 Super Free (204K context, 128K output, reasoning, coding)
- Qwen3.6 Plus Free (1M context, 64K output, reasoning, coding)
- Trinity Large Preview Free (131K context, 131K output, coding)

## How It Works

1. **Dynamic Model Discovery**: The gateway fetches available models from the OpenCode Zen API
2. **Cost Filtering**: Models are filtered based on cost information from models.dev
3. **Free Model Selection**: Only models with `cost.input === 0` are included
4. **Endpoint Routing**: Each model is routed to its correct API endpoint

## Integration Steps

To integrate this gateway into openclaude:

1. Copy the gateway file to `src/integrations/gateways/opencode-free.ts`
2. Copy the model descriptors to `src/integrations/models/opencode-free.ts`
3. Register the gateway in the integration artifacts
4. The gateway will appear in the `/provider` preset picker as "OpenCode Free"

## Testing

The implementation follows the same patterns as the existing OpenCode Zen gateway:
- Uses `defineGateway` and `defineModel` from openclaude's descriptor system
- Maintains compatibility with existing transport and authentication mechanisms
- Preserves per-model endpoint routing for different API formats

## Usage

Users can select "OpenCode Free" from the `/provider` preset picker in openclaude to access only free models from the OpenCode Zen platform. No paid models will be shown or available for selection.

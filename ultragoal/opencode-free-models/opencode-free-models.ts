import { defineModel } from '../define.js'

const baseCapabilities = {
  supportsStreaming: true,
  supportsFunctionCalling: true,
  supportsJsonMode: true,
  supportsPreciseTokenCount: false,
}

type OpenCodeModelSpec = {
  id: string
  label: string
  defaultModel: string
  contextWindow: number
  maxOutputTokens: number
  reasoning?: boolean
  vision?: boolean
  coding?: boolean
}

function openCodeModel(spec: OpenCodeModelSpec) {
  return defineModel({
    id: spec.id,
    label: spec.label,
    vendorId: 'openai',
    classification: [
      'chat',
      ...(spec.reasoning ? ['reasoning' as const] : []),
      ...(spec.vision ? ['vision' as const] : []),
      ...(spec.coding ? ['coding' as const] : []),
    ],
    defaultModel: spec.defaultModel,
    providerModelMap: {
      'opencode-free': spec.defaultModel,
    },
    capabilities: {
      ...baseCapabilities,
      supportsVision: spec.vision === true,
      supportsReasoning: spec.reasoning === true,
    },
    contextWindow: spec.contextWindow,
    maxOutputTokens: spec.maxOutputTokens,
  })
}

// Free models from OpenCode Zen (cost.input === 0)
const freeModels: OpenCodeModelSpec[] = [
  // Claude models (free tier)
  { id: 'opencode-free-claude-3-5-haiku', label: 'Claude 3.5 Haiku', defaultModel: 'claude-3-5-haiku', contextWindow: 200_000, maxOutputTokens: 8_192, vision: true },
  
  // GLM models (free)
  { id: 'opencode-free-glm-4.6', label: 'GLM 4.6', defaultModel: 'glm-4.6', contextWindow: 202_752, maxOutputTokens: 131_072, coding: true },
  { id: 'opencode-free-glm-4.7', label: 'GLM 4.7', defaultModel: 'glm-4.7', contextWindow: 202_752, maxOutputTokens: 131_072, coding: true },
  { id: 'opencode-free-glm-5', label: 'GLM 5', defaultModel: 'glm-5', contextWindow: 202_752, maxOutputTokens: 131_072, coding: true },
  
  // Kimi models (free)
  { id: 'opencode-free-kimi-k2', label: 'Kimi K2', defaultModel: 'kimi-k2', contextWindow: 128_000, maxOutputTokens: 128_000, coding: true },
  { id: 'opencode-free-kimi-k2-thinking', label: 'Kimi K2 Thinking', defaultModel: 'kimi-k2-thinking', contextWindow: 262_144, maxOutputTokens: 262_144, reasoning: true, coding: true },
  { id: 'opencode-free-kimi-k2.5', label: 'Kimi K2.5', defaultModel: 'kimi-k2.5', contextWindow: 262_144, maxOutputTokens: 262_144, reasoning: true, vision: true, coding: true },
  
  // MiniMax models (free)
  { id: 'opencode-free-minimax-m2.1', label: 'MiniMax M2.1', defaultModel: 'minimax-m2.1', contextWindow: 204_800, maxOutputTokens: 131_072, reasoning: true, coding: true },
  { id: 'opencode-free-minimax-m2.5', label: 'MiniMax M2.5', defaultModel: 'minimax-m2.5', contextWindow: 204_800, maxOutputTokens: 131_072, reasoning: true, vision: true, coding: true },
  { id: 'opencode-free-minimax-m2.5-free', label: 'MiniMax M2.5 Free', defaultModel: 'minimax-m2.5-free', contextWindow: 204_800, maxOutputTokens: 131_072, reasoning: true, coding: true },
  
  // Other free models
  { id: 'opencode-free-big-pickle', label: 'Big Pickle', defaultModel: 'big-pickle', contextWindow: 200_000, maxOutputTokens: 128_000, reasoning: true, coding: true },
  { id: 'opencode-free-nemotron-3-super-free', label: 'Nemotron 3 Super Free', defaultModel: 'nemotron-3-super-free', contextWindow: 204_800, maxOutputTokens: 128_000, reasoning: true, coding: true },
  { id: 'opencode-free-qwen3.6-plus-free', label: 'Qwen3.6 Plus Free', defaultModel: 'qwen3.6-plus-free', contextWindow: 1_048_576, maxOutputTokens: 64_000, reasoning: true, coding: true },
  { id: 'opencode-free-trinity-large-preview-free', label: 'Trinity Large Preview Free', defaultModel: 'trinity-large-preview-free', contextWindow: 131_072, maxOutputTokens: 131_072, coding: true },
]

export default freeModels.map(openCodeModel)

import { defineGateway } from '../define.js'
import type { ReasoningControlMode, ReasoningEffortLevel, ReasoningWireFormat, OpenAIShimTransportConfig } from '../descriptors.js'

type OpenCodeCatalogSpec = {
  id: string
  label: string
  endpointPath?: string
  zaiGlm?: boolean
}

function catalogEntry(spec: OpenCodeCatalogSpec) {
  const openaiShim: Partial<OpenAIShimTransportConfig> = {
    ...(spec.zaiGlm ? ZAI_GLM_OPENAI_SHIM : {}),
    ...(spec.endpointPath ? { endpointPath: spec.endpointPath } : {}),
  }
  return {
    id: spec.id,
    apiName: spec.id,
    label: spec.label,
    modelDescriptorId: `opencode-free-${spec.id}`,
    ...(spec.zaiGlm
      ? {
          capabilities: {
            supportsFunctionCalling: true,
            supportsJsonMode: true,
            supportsReasoning: true,
          },
          reasoning: {
            mode: 'levels' as ReasoningControlMode,
            levels: ['high', 'xhigh'] as ReasoningEffortLevel[],
            wireFormat: 'zai_compatible' as ReasoningWireFormat,
          },
        }
      : {}),
    ...(Object.keys(openaiShim).length > 0
      ? { transportOverrides: { openaiShim } }
      : {}),
  }
}

// Static fallback list of known free models from OpenCode Zen
// These are models where cost.input === 0 according to models.dev
const freeModelsFallback: OpenCodeCatalogSpec[] = [
  // Claude models (free tier)
  { id: 'claude-3-5-haiku', label: 'Claude 3.5 Haiku', endpointPath: '/messages' },
  
  // GLM models (free)
  { id: 'glm-4.6', label: 'GLM 4.6', zaiGlm: true },
  { id: 'glm-4.7', label: 'GLM 4.7', zaiGlm: true },
  { id: 'glm-5', label: 'GLM 5', zaiGlm: true },
  
  // Kimi models (free)
  { id: 'kimi-k2', label: 'Kimi K2' },
  { id: 'kimi-k2-thinking', label: 'Kimi K2 Thinking' },
  { id: 'kimi-k2.5', label: 'Kimi K2.5' },
  
  // MiniMax models (free)
  { id: 'minimax-m2.1', label: 'MiniMax M2.1' },
  { id: 'minimax-m2.5', label: 'MiniMax M2.5' },
  { id: 'minimax-m2.5-free', label: 'MiniMax M2.5 Free' },
  
  // Other free models
  { id: 'big-pickle', label: 'Big Pickle' },
  { id: 'nemotron-3-super-free', label: 'Nemotron 3 Super Free' },
  { id: 'qwen3.6-plus-free', label: 'Qwen3.6 Plus Free', endpointPath: '/messages' },
  { id: 'trinity-large-preview-free', label: 'Trinity Large Preview Free' },
]

// Dynamic model fetching from OpenCode Zen API
async function fetchAvailableModels(): Promise<OpenCodeCatalogSpec[]> {
  try {
    const response = await fetch('https://opencode.ai/zen/v1/models', {
      headers: {
        'User-Agent': 'openclaude/latest/cli',
      },
    })
    
    if (!response.ok) {
      console.warn('Failed to fetch OpenCode Zen models, using fallback list')
      return freeModelsFallback
    }
    
    const data = await response.json() as { data?: Array<{ id?: string }> }
    const modelIds = (data.data ?? [])
      .map(m => m.id)
      .filter((id): id is string => Boolean(id))
    
    // Filter to only include models from our free list
    return freeModelsFallback.filter(m => modelIds.includes(m.id))
  } catch (error) {
    console.warn('Error fetching OpenCode Zen models:', error)
    return freeModelsFallback
  }
}

// Cost information from models.dev
type ModelsDevModelInfo = {
  status?: string | null
  cost?: {
    input?: number | null
    output?: number | null
    cache_read?: number | null
    cache_write?: number | null
  } | null
}

async function fetchModelsDevInfo(): Promise<Record<string, ModelsDevModelInfo> | undefined> {
  try {
    const response = await fetch('https://models.dev/api.json')
    if (!response.ok) return undefined
    
    const json = await response.json() as {
      opencode?: { models?: Record<string, ModelsDevModelInfo> }
    }
    return json.opencode?.models
  } catch {
    return undefined
  }
}

// Filter models to only include free ones (cost.input === 0)
function filterFreeModels(
  models: OpenCodeCatalogSpec[],
  modelsDevInfo?: Record<string, ModelsDevModelInfo>,
): OpenCodeCatalogSpec[] {
  if (!modelsDevInfo) {
    // If we can't fetch cost info, return all models (they're from our curated free list)
    return models
  }
  
  return models.filter(model => {
    const info = modelsDevInfo[model.id]
    if (!info) return false
    
    const cost = info.cost
    if (!cost) return false
    
    // Only include models where input cost is 0
    return (cost.input ?? 0) === 0
  })
}

// ZAI GLM transport configuration
const ZAI_GLM_OPENAI_SHIM: Partial<OpenAIShimTransportConfig> = {
  // ZAI GLM specific configuration
}

// Build the catalog with dynamic fetching and free model filtering
async function buildCatalog() {
  const [availableModels, modelsDevInfo] = await Promise.all([
    fetchAvailableModels(),
    fetchModelsDevInfo(),
  ])
  
  const freeModels = filterFreeModels(availableModels, modelsDevInfo)
  
  return {
    source: 'static' as const,
    models: freeModels.map(catalogEntry),
  }
}

// Export the gateway definition
export default defineGateway({
  id: 'opencode-free',
  label: 'OpenCode Free',
  category: 'aggregating',
  defaultBaseUrl: 'https://opencode.ai/zen/v1',
  defaultModel: 'big-pickle',
  setup: {
    requiresAuth: true,
    authMode: 'api-key',
    credentialEnvVars: ['OPENCODE_API_KEY'],
  },
  transportConfig: {
    kind: 'openai-compatible',
    openaiShim: {
      supportsAuthHeaders: true,
    },
  },
  preset: {
    id: 'opencode-free',
    vendorId: 'openai',
    description: 'OpenCode Free - free models only (no cost)',
    apiKeyEnvVars: ['OPENCODE_API_KEY'],
    modelEnvVars: ['OPENAI_MODEL'],
  },
  validation: {
    kind: 'credential-env',
    routing: {
      matchDefaultBaseUrl: true,
    },
    credentialEnvVars: ['OPENCODE_API_KEY', 'OPENAI_API_KEYS', 'OPENAI_API_KEY'],
    missingCredentialMessage:
      'OPENCODE_API_KEY or OPENAI_API_KEYS / OPENAI_API_KEY is required. Get your API key from https://opencode.ai',
  },
  catalog: {
    source: 'static',
    models: freeModelsFallback.map(catalogEntry),
  },
  usage: { supported: false },
})

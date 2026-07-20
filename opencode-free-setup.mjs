import { createServer } from 'node:http'
import { request as httpsRequest, Agent } from 'node:https'
import crypto from 'node:crypto'
import process from 'node:process'

const MODELS_DEV_URL = 'https://models.dev/api.json'
const OPCODE_ZEN_HOST = 'opencode.ai'
const OPCODE_ZEN_PORT = 443
const UPSTREAM_PATH_PREFIX = '/zen'
const REQUEST_TIMEOUT_MS = 60_000
const PORT = parseInt(process.env.OPENCODE_FREE_PROXY_PORT ?? process.env.PORT ?? '8080', 10)
const HOST = '127.0.0.1'

const UPSTREAM_AGENT = new Agent({ keepAlive: true, keepAliveMsecs: 30_000, maxSockets: 64 })

const FALLBACK_FREE_MODELS = [
  'deepseek-v4-flash-free', 'qwen3.6-plus-free', 'glm-5',
  'nemotron-3-super-free', 'big-pickle', 'minimax-m2.5-free',
  'kimi-k2.5', 'kimi-k2', 'kimi-k2-thinking', 'glm-4.7',
  'glm-4.6', 'minimax-m2.1', 'trinity-large-preview-free',
]

const MODEL_PRIORITY = [
  'deepseek-v4-flash-free', 'qwen3.6-plus-free',
  'nemotron-3-super-free', 'big-pickle',
  'minimax-m2.5-free', 'kimi-k2.5', 'glm-5',
]

const HOP_BY_HOP = new Set([
  'connection', 'keep-alive', 'proxy-authenticate', 'proxy-authorization',
  'te', 'trailer', 'transfer-encoding', 'upgrade',
])

const freeModelIdsPromise = fetchFreeModels()

function generateId() {
  return crypto.randomUUID().replace(/-/g, '').slice(0, 26)
}

function buildOpenCodeHeaders() {
  return {
    'User-Agent': 'opencode/latest/1.3.15/cli',
    'x-opencode-client': 'cli',
    'x-opencode-session': generateId(),
    'x-opencode-project': generateId(),
    'x-opencode-request': generateId(),
  }
}

async function fetchFreeModels() {
  try {
    const response = await fetch(MODELS_DEV_URL)
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const data = await response.json()
    const free = []
    for (const [id, info] of Object.entries(data?.opencode?.models ?? {})) {
      if (info?.status === 'deprecated') continue
      if (info?.cost?.input === 0 && info?.cost?.output === 0) {
        free.push(id)
      }
    }
    if (free.length > 0) return free
    throw new Error('No free models found in API response')
  } catch (error) {
    console.error(`[opencode-free] Warning: failed to fetch free model list: ${error.message}`)
    return FALLBACK_FREE_MODELS
  }
}

function pickBestModel(ids) {
  for (const preferred of MODEL_PRIORITY) {
    if (ids.includes(preferred)) return preferred
  }
  return ids[0]
}

function formatModelList(ids) {
  return {
    object: 'list',
    data: ids.map(id => ({
      id,
      object: 'model',
      created: 1710000000,
      owned_by: 'opencode',
    })),
  }
}

function formatModelEntry(id) {
  return {
    id,
    object: 'model',
    created: 1710000000,
    owned_by: 'opencode',
  }
}

function formatErrorBody(message, type, code = null, param = null) {
  return { error: { message, type: type ?? 'not_found', code, param } }
}

function isHopByHopHeader(name) {
  return HOP_BY_HOP.has(name.toLowerCase())
}

function corsHeaders() {
  return {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': '*',
    'Access-Control-Max-Age': '86400',
  }
}

function sendJson(res, statusCode, data) {
  const headers = { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' }
  res.writeHead(statusCode, headers)
  res.end(JSON.stringify(data))
}

function normalizePath(url) {
  if (url.startsWith('/zen/v1/')) return '/v1/' + url.slice('/zen/v1/'.length)
  if (url.startsWith('/v1/')) return url
  return null
}

async function handleModelsList(req, res) {
  const ids = await freeModelIdsPromise
  sendJson(res, 200, formatModelList(ids))
}

async function handleModelById(req, res, url) {
  const modelId = url.slice('/v1/models/'.length)
  const ids = await freeModelIdsPromise
  if (ids.includes(modelId)) {
    sendJson(res, 200, formatModelEntry(modelId))
  } else {
    sendJson(res, 404, formatErrorBody(`Model '${modelId}' not found`))
  }
}

function handleChatCompletions(req, res) {
  const controller = new AbortController()
  let timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS)

  const opencodeHeaders = buildOpenCodeHeaders()
  if (req.headers['content-type']) {
    opencodeHeaders['content-type'] = req.headers['content-type']
  }

  const proxyReq = httpsRequest({
    hostname: OPCODE_ZEN_HOST,
    port: OPCODE_ZEN_PORT,
    path: UPSTREAM_PATH_PREFIX + '/v1/chat/completions',
    method: 'POST',
    headers: opencodeHeaders,
    rejectUnauthorized: true,
    agent: UPSTREAM_AGENT,
    signal: controller.signal,
  }, (proxyRes) => {
    clearTimeout(timeout)
    const responseHeaders = { 'access-control-allow-origin': '*' }
    for (const [key, value] of Object.entries(proxyRes.headers)) {
      if (!isHopByHopHeader(key)) responseHeaders[key] = value
    }

    if (proxyRes.headers['content-type']?.includes('text/event-stream')) {
      responseHeaders['cache-control'] = 'no-cache'
      responseHeaders['x-accel-buffering'] = 'no'
      proxyRes.on('data', () => {
        clearTimeout(timeout)
        timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS)
      })
      proxyRes.on('close', () => clearTimeout(timeout))
    }

    res.writeHead(proxyRes.statusCode, proxyRes.statusMessage ?? '', responseHeaders)
    proxyRes.pipe(res)
  })

  proxyReq.on('error', (error) => {
    clearTimeout(timeout)
    if (!res.headersSent) {
      const isTimeout = error.name === 'AbortError'
      sendJson(res, isTimeout ? 408 : 502, formatErrorBody(
        isTimeout ? 'Upstream request timed out' : `Upstream error: ${error.message}`,
        isTimeout ? 'timeout_error' : 'upstream_error'
      ))
    } else {
      res.destroy()
    }
  })

  req.on('close', () => {
    clearTimeout(timeout)
    if (req.socket?.destroyed && !proxyReq.destroyed) {
      proxyReq.destroy()
    }
  })

  req.pipe(proxyReq)
}

function handleHealth(res) {
  sendJson(res, 200, { status: 'ok', uptime: process.uptime() })
}

function start() {
  const server = createServer(async (req, res) => {
    const { method, url } = req

    if (method === 'OPTIONS') {
      res.writeHead(204, corsHeaders())
      return res.end()
    }

    if (method === 'GET' && url === '/health') {
      return handleHealth(res)
    }

    const normalized = normalizePath(url)
    if (!normalized) {
      return sendJson(res, 404, formatErrorBody('Not found', 'not_found'))
    }

    try {
      if (method === 'GET' && normalized === '/v1/models') {
        return await handleModelsList(req, res)
      }
      if (method === 'GET' && normalized.startsWith('/v1/models/')) {
        return await handleModelById(req, res, normalized)
      }
      if (method === 'POST' && normalized === '/v1/chat/completions') {
        return handleChatCompletions(req, res)
      }
      return sendJson(res, 404, formatErrorBody('Not found', 'not_found'))
    } catch (error) {
      console.error(`[opencode-free] Route error: ${error.message}`)
      if (!res.headersSent) {
        sendJson(res, 500, formatErrorBody('Internal server error', 'server_error'))
      } else {
        res.destroy()
      }
    }
  })

  server.listen(PORT, HOST, () => {
    const addr = server.address()
    console.error(`[opencode-free] Proxy running on http://${HOST}:${addr.port}`)
  })

  const shutdown = () => {
    console.error('[opencode-free] Shutting down...')
    UPSTREAM_AGENT.destroy()
    server.close(() => process.exit(0))
    setTimeout(() => process.exit(0), 2000).unref()
  }
  process.on('SIGINT', shutdown)
  process.on('SIGTERM', shutdown)

  return server
}

start()

# Stage 5 Refinement — Merged Plan

## 1. Architecture Overview

### Current (261 lines)
```
┌─ Port 1: HTTP Proxy Server (lines 1–205)
│   • Router: /zen/v1/models, /zen/v1/models/:id, /zen/v1/chat/completions
│   • Forwarding: req.pipe() → https.request → proxyRes.pipe(res)
│   • Fresh pi-opencode-zen headers per request
│   • CORS on every response
│   • Hop-by-hop stripping
│
└─ Port 2: openclaude Spawner (lines 207–261)
    • main() → listen(0) → spawn(openclaude, ...) → signal handlers → cleanup
```

### Target (~175 lines)
```
┌─ Standalone HTTP Proxy Server
│   • Router: /v1/models, /v1/models/:id, /v1/chat/completions
│             /zen/v1/models, /zen/v1/models/:id, /zen/v1/chat/completions
│   • Forwarding: https.Agent (keepAlive pool) → req.pipe() → proxyRes.pipe(res)
│   • Request timeout (AbortController, 60s, SSE per-data-event reset)
│   • SSE streaming passthrough with header augmentation
│   • Health check endpoint (GET /health)
│   • CORS preflight (OPTIONS → 204)
│   • Full OpenAI error format (message, type, code, param)
│   • Client disconnect cleanup (req.on('close') → proxyReq.destroy())
│   • Graceful shutdown (SIGINT/SIGTERM → Agent.destroy() → server.close() → exit)
```

**Key architectural change:** Removed `child_process` entirely. No `spawn()`, no child lifecycle management. The server IS the process — it starts, prints its address to stderr, and stays alive until killed. This is the same model as `http-server`, `serve`, `webpack-dev-server`.

**Port env chain:** `OPENCODE_FREE_PROXY_PORT` > `PORT` > `8080` default.

---

## 2. Exact Change List

### Delete

| Lines | Content | Why |
|-------|---------|-----|
| 3 | `import { spawn } from 'node:child_process'` | No child processes |
| 207–261 | `async function main() { ... }` / `main().catch(...)` | Entire spawner: env setup, spawn, child lifecycle, signal forwarding |

**Total: 56 lines removed** (line 3 + lines 207–261)

### Modify

| Lines | Change | Details |
|-------|--------|---------|
| 1 | `import { createServer } from 'node:http'` | **Unchanged** |
| 2 | `import { request as httpsRequest } from 'node:https'` | → `import { request as httpsRequest, Agent } from 'node:https'` — adds Agent for connection pool |
| 4 | `import crypto from 'node:crypto'` | **Unchanged** |
| 5 | `import process from 'node:process'` | **Unchanged** |
| 7–9 | `MODELS_DEV_URL`, `OPCODE_ZEN_HOST`, `OPCODE_ZEN_PORT` | **Unchanged** (keep as-is) |
| After line 9 | — | Add 4 new constants: `UPSTREAM_PATH_PREFIX = '/zen'`, `REQUEST_TIMEOUT_MS = 60_000`, `PORT` (with env chain), `HOST = '127.0.0.1'` |
| After new constants | — | Add `UPSTREAM_AGENT = new Agent({ keepAlive: true, keepAliveMsecs: 30_000, maxSockets: 64 })` |
| 11–16 | `FALLBACK_FREE_MODELS` | **Unchanged** |
| 18–22 | `MODEL_PRIORITY` | **Unchanged** |
| 24–27 | `HOP_BY_HOP` | **Unchanged** |
| 29 | `let cachedFreeModels = null` | **Delete** — no longer needed (no main() caching) |
| 30 | `const freeModelIdsPromise = fetchFreeModels()` | **Unchanged** — eager fetch (per decision) |
| 62 | `return cachedFreeModels ?? FALLBACK_FREE_MODELS` | → `return FALLBACK_FREE_MODELS` — no more cachedFreeModels |
| 94–96 | `formatErrorBody(message, type)` | → `formatErrorBody(message, type, code = null, param = null)` — adds `code` and `param` fields for full OpenAI error shape |
| 102–109 | `handleModelsList` | Simplify: use `sendJson()` helper instead of raw `writeHead/end` |
| 111–127 | `handleModelById` | Model ID extraction: use normalized URL → `.slice('/v1/models/'.length)` instead of hardcoded `/zen/v1/models/`. Use `sendJson()` helper. |
| 129–168 | `handleChatCompletions` | **Major rewrite** — see §2.1 below |
| 170–205 | `const server = createServer(...)` router | **Rewrite** — see §2.2 below |

### Add (new code at appropriate positions)

| Position | What | Lines (approx) |
|----------|------|----------------|
| After global constants | `UPSTREAM_AGENT` | 1 |
| After helper functions | `corsHeaders()` — returns CORS headers object | 4 |
| After corsHeaders | `sendJson(res, statusCode, data)` — JSON response helper | 3 |
| After sendJson | `normalizePath(url)` — maps `/zen/v1/...` and `/v1/...` to `/v1/...` | 5 |
| After route handlers | `handleHealth(res)` — returns `{ status: "ok" }` | 3 |
| After handleHealth | `start()` — server creation, listen, signal handling, returns server | 20 |
| At end of file | `start()` — top-level invocation | 1 |

### 2.1 `handleChatCompletions` Rewrite (replaces lines 129–168)

Changes from current:
1. **AbortController timeout** — 60s timeout per request, cleared on response
2. **SSE per-data-event reset** — if `content-type` includes `text/event-stream`, each `data` event on `proxyRes` resets the timeout; `close` event clears it
3. **Connection pool** — `agent: UPSTREAM_AGENT` on upstream request options
4. **SSE header augmentation** — if SSE detected, add `cache-control: no-cache` and `x-accel-buffering: no` to response headers
5. **Client disconnect cleanup** — `req.on('close')` → `clearTimeout(timeout)` + `proxyReq.destroy()`
6. **Timeout error type** — `AbortError` → 408 status, `timeout_error` type
7. **UPSTREAM_PATH_PREFIX** — upstream path built as `UPSTREAM_PATH_PREFIX + '/v1/chat/completions'`
8. **Backpressure** — `.pipe()` (per decision, Node's built-in backpressure is battle-tested)

### 2.2 Server Router Rewrite (replaces lines 170–205)

Changes from current:
1. **CORS preflight first** — `OPTIONS` → 204 with CORS headers
2. **Health check** — `GET /health` → `handleHealth(res)`
3. **normalizePath** — normalizes `/zen/v1/...` and `/v1/...` to `/v1/...`
4. **Pass normalized URL to handlers** — `handleModelById` receives normalized URL
5. **All handlers use `sendJson()`** — consistent JSON response creation

Constants kept:
- `freeModelIdsPromise` remains module-scoped (eager fetch)
- `FALLBACK_FREE_MODELS`, `MODEL_PRIORITY`, `HOP_BY_HOP` unchanged

Helpers kept:
- `generateId()`, `buildOpenCodeHeaders()`, `fetchFreeModels()`, `pickBestModel()`
- `formatModelList()`, `formatModelEntry()`
- `isHopByHopHeader()` — unchanged

---

## 3. Final Code (complete script, ~175 lines)

```js
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
    proxyReq.destroy()
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
        return handleModelsList(req, res)
      }
      if (method === 'GET' && normalized.startsWith('/v1/models/')) {
        return handleModelById(req, res, normalized)
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
```

---

## 4. Feature Checklist

- [x] No spawn() or child_process import — line 3 deleted, `node:child_process` never imported
- [x] Standalone proxy (no openclaude needed) — `main()` removed, process IS the server
- [x] Dynamic free model fetch — `fetchFreeModels()` via `models.dev/api.json`, same as before
- [x] Only free models in /v1/models — `formatModelList()` filters by status/cost, falls back to curated list
- [x] pi-opencode-zen headers (User-Agent, x-opencode-*) — `buildOpenCodeHeaders()` with `generateId()` per-request
- [x] Dual-path routing (/v1/* and /zen/v1/*) — `normalizePath()` maps both to same `/v1/...` handlers
- [x] CORS on all responses — `Access-Control-Allow-Origin: *` in `sendJson()`, `corsHeaders()`, and `handleChatCompletions`
- [x] CORS preflight (OPTIONS → 204) — first check in router, returns 204 with `corsHeaders()`
- [x] Health check endpoint — `GET /health` → `{ status: "ok", uptime: N }`
- [x] Streaming SSE support — SSE detected via `content-type`, forwarded with `cache-control: no-cache` + `x-accel-buffering: no`
- [x] Hop-by-hop header stripping — `isHopByHopHeader()` filters 8 headers from upstream response
- [x] Client disconnect cleanup — `req.on('close')` → `clearTimeout(timeout)` + `proxyReq.destroy()`
- [x] Connection pooling (https.Agent, keepAlive) — `UPSTREAM_AGENT = new Agent({ keepAlive: true, keepAliveMsecs: 30000, maxSockets: 64 })`
- [x] Request timeout (60s with SSE data-event reset) — `AbortController` + `setTimeout(60_000)`, SSE data events reset timer
- [x] Full OpenAI error format (message, type, code, param) — `formatErrorBody()` includes all 4 fields
- [x] Agent destroy on shutdown — `UPSTREAM_AGENT.destroy()` in shutdown handler
- [x] Forced exit fallback — `setTimeout(() => process.exit(0), 2000).unref()` after `server.close()`
- [x] Zero external dependencies — only `node:http`, `node:https`, `node:crypto`, `node:process`
- [x] Zero config file modification — no file I/O for config; no write to `~/.openclaude/`
- [x] Zero .env files — configuration via env vars only (`PORT`, `OPENCODE_FREE_PROXY_PORT`)
- [x] Default port 8080 (configurable via PORT env, OPENCODE_FREE_PROXY_PORT override) — env chain: `OPENCODE_FREE_PROXY_PORT` > `PORT` > `8080`
- [x] Signal handling (SIGINT, SIGTERM) — `shutdown()` registered for both, drains Agent + closes server

### Error types supported

| HTTP Status | Type | Trigger |
|-------------|------|---------|
| 404 | `not_found` | Unknown path, unknown model ID |
| 408 | `timeout_error` | Upstream request exceeds 60s without data |
| 500 | `server_error` | Route handler throws unexpected error |
| 502 | `upstream_error` | Upstream connection error (DNS, TLS, ECONNREFUSED) |

### Path mapping

| Incoming | Normalized | Upstream forwarded to |
|----------|-----------|----------------------|
| `GET /v1/models` | `/v1/models` | (served locally from fetched list) |
| `GET /zen/v1/models` | `/v1/models` | (same) |
| `GET /v1/models/:id` | `/v1/models/:id` | (served locally) |
| `GET /zen/v1/models/:id` | `/v1/models/:id` | (same) |
| `POST /v1/chat/completions` | `/v1/chat/completions` | `opencode.ai/zen/v1/chat/completions` |
| `POST /zen/v1/chat/completions` | `/v1/chat/completions` | (same upstream) |
| `GET /health` | (no normalization) | (served locally) |
| `OPTIONS *` | (no normalization) | (204 immediately) |
| Everything else | `null` | 404 |

---

## 5. Testing Strategy

### Unit tests (manual curl — zero dependencies)

**Test 1: Proxy starts and health check responds**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8080/health
# Expected: 200
kill $PID
```

**Test 2: Model list returns only free models**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
MODELS=$(curl -s http://127.0.0.1:8080/v1/models)
echo "$MODELS" | python3 -c "import json,sys; d=json.load(sys.stdin); assert d['object']=='list'; assert all(m['owned_by']=='opencode' for m in d['data'])"
# Verify: both paths return same list
[ "$(curl -s http://127.0.0.1:8080/v1/models)" = "$(curl -s http://127.0.0.1:8080/zen/v1/models)" ] && echo "PASS" || echo "FAIL"
kill $PID
```

**Test 3: Model by ID — found and not found**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -w " %{http_code}" http://127.0.0.1:8080/v1/models/deepseek-v4-flash-free | grep -q " 200$" && echo "PASS" || echo "FAIL"
curl -s -w " %{http_code}" http://127.0.0.1:8080/v1/models/nonexistent | grep -q " 404$" && echo "PASS" || echo "FAIL"
kill $PID
```

**Test 4: Chat completion (non-streaming)**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Say hello in French, one word"}],"max_tokens":5}' \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['choices'][0]['message']['content'])"
kill $PID
```

**Test 5: SSE streaming**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Count to 3"}],"stream":true,"max_tokens":20}' \
  | tee /tmp/sse-test.out | head -5
grep -q "data: \[DONE\]" /tmp/sse-test.out && echo "DONE marker found: PASS" || echo "FAIL"
kill $PID
```

**Test 6: CORS headers everywhere**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -D - http://127.0.0.1:8080/v1/models 2>&1 | grep -qi "access-control-allow-origin: \*" && echo "GET: PASS" || echo "GET: FAIL"
curl -s -D - -X OPTIONS http://127.0.0.1:8080/v1/chat/completions 2>&1 | grep -qi "access-control-allow-origin: \*" && echo "OPTIONS: PASS" || echo "OPTIONS: FAIL"
kill $PID
```

**Test 7: No child_process dependency**
```bash
grep -c "child_process" /tmp/opencode/opencode-free-setup.mjs
# Expected: 0
```

**Test 8: 404 on unknown paths**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -w " %{http_code}" http://127.0.0.1:8080/unknown | grep -q " 404$" && echo "unknown path: PASS" || echo "FAIL"
curl -s -w " %{http_code}" http://127.0.0.1:8080/v1/unknown | grep -q " 404$" && echo "v1/unknown: PASS" || echo "FAIL"
kill $PID
```

**Test 9: Concurrent requests**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 3
for i in {1..5}; do
  curl -s -X POST http://127.0.0.1:8080/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"hi"}],"max_tokens":3}' \
    -o /tmp/resp-$i.json -w "%{http_code}\n" &
done
wait
for i in {1..5}; do echo "Response $i: $(cat /tmp/resp-$i.json | python3 -c "import json,sys; print(json.load(sys.stdin).get('error',{}).get('message','ok')[:20])")"; done
kill $PID
```

**Test 10: Graceful shutdown**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
kill $PID; sleep 1
kill -0 $PID 2>/dev/null && echo "FAIL (still running)" || echo "PASS (exited)"
```

**Test 11: Port env var chain**
```bash
PORT=9090 node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:9090/health
# Expected: 200
kill $PID
OPENCODE_FREE_PROXY_PORT=9091 node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:9091/health
# Expected: 200
kill $PID
```

**Test 12: /zen/v1 path parity**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!; sleep 2
[ "$(curl -s http://127.0.0.1:8080/v1/models)" = "$(curl -s http://127.0.0.1:8080/zen/v1/models)" ] && echo "model list parity: PASS" || echo "FAIL"
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8080/zen/v1/chat/completions
# Verify: POST with body works the same
kill $PID
```

### Integration test (openclaude round-trip)

```bash
node /tmp/opencode/opencode-free-setup.mjs &
sleep 3
CLAUDE_CODE_USE_OPENAI=1 \
  OPENAI_BASE_URL=http://127.0.0.1:8080/v1 \
  OPENAI_API_KEY=public \
  OPENAI_MODEL=deepseek-v4-flash-free \
  timeout 30 openclaude "Say hello in French, one word only"
# Expected: outputs "Bonjour."
```

### File impact verification

```bash
# Verify no ~/.openclaude/ files modified
ls -la ~/.openclaude/ 2>&1 | head -3
# Should be unchanged or non-existent
```

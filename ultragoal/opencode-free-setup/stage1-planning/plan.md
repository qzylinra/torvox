# Final Plan: Reverse Proxy Script for openclaude → opencode.ai Free Models

**Author**: Plan A's author (integrated)
**Source documents**: `plan-a.md`, `plan-b.md`, `planning-decision.md`

---

## 1. Architecture

Single Node.js file (`opencode-free-setup.mjs`) with **zero external dependencies** (Node.js built-in modules only). Creates a local HTTP proxy that sits between openclaude and `opencode.ai/zen/v1/`.

```
openclaude → localhost:{RANDOM_PORT} (proxy) → opencode.ai/zen/v1/
               └── intercepts /zen/v1/models (returns free models only)
               └── intercepts /zen/v1/models/:id (returns single model or 404)
               └── forwards /zen/v1/chat/completions with pi-opencode-zen headers
               └── all other paths → 404
```

### Key design decisions (from planning-decision.md)

| Decision | Ruling |
|----------|--------|
| Architecture | Single file (Plan A) |
| Port | Dynamic port 0 (Plan A) |
| Auth handling | Strip Authorization before upstream (Plan A) |
| Model validation on chat | Skip it — `/v1/models` filtering is sufficient |
| SSE streaming | `pipe()` for response (both) |
| Model list endpoint | Include `/v1/models/{id}` (Plan A) |
| `--bg` mode | Support it with `child.unref()` (Plan A) |
| Exit propagation | Pass through child exit code with 2s timeout (Plan A) |
| Session/project IDs | Fresh per request (as in pi-opencode-zen) |

### Adopted from Plan B

1. `req.pipe(proxyReq)` for request body — zero-copy, no buffering
2. Full hop-by-hop header stripping (connection, keep-alive, proxy-authenticate, proxy-authorization, te, trailer, transfer-encoding, upgrade)
3. Fresh `x-opencode-*` IDs per request (matches pi-opencode-zen behavior exactly)
4. `headersSent` guard in upstream error handler — prevent crash on late errors
5. CORS headers on model list response (`Access-Control-Allow-Origin: *`)
6. `/zen/v1` prefix filtering — only handle known paths, return 404 for unknown
7. Cache fallback preserving stale data — if fetch fails, keep previously cached data (or use hardcoded fallback if no cache exists yet)
8. No request body buffering — use `req.pipe()` instead of collecting body

---

## 2. Module Imports (Zero Dependencies)

```js
import { createServer } from 'node:http'
import { request as httpsRequest } from 'node:https'
import { spawn } from 'node:child_process'
import crypto from 'node:crypto'
import process from 'node:process'
```

Global `fetch` (available since Node.js 18) for the models.dev fetch. No npm packages. No external dependencies.

---

## 3. Constants

```js
const MODELS_DEV_URL = 'https://models.dev/api.json'
const OPCODE_ZEN_HOST = 'opencode.ai'
const OPCODE_ZEN_PORT = 443

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
```

---

## 4. Utility Functions

### `generateId() → string`

```js
function generateId() {
  return crypto.randomUUID().replace(/-/g, '').slice(0, 26)
}
```

Produces a 26-character lowercase hex string, identical to pi-opencode-zen's format.

### `buildOpenCodeHeaders() → object`

```js
function buildOpenCodeHeaders() {
  return {
    'User-Agent': 'opencode/latest/1.3.15/cli',
    'x-opencode-client': 'cli',
    'x-opencode-session': generateId(),
    'x-opencode-project': generateId(),
    'x-opencode-request': generateId(),
  }
}
```

Called once per proxied request — every forwarded request gets fresh IDs.

### `fetchFreeModels() → Promise<string[]>`

- Fetches `https://models.dev/api.json`
- Filters for models where `cost.input === 0 && cost.output === 0`
- Excludes models with `status === 'deprecated'`
- On success: returns array of free model IDs
- On network failure or non-2xx response:
  - If a cached result exists (from a previous successful fetch), **return the stale cache** (preserving stale data on transient failure)
  - If no cache exists, return `FALLBACK_FREE_MODELS`
  - In both fallback cases: write warning to stderr
- If the fetch succeeds but returns an empty model list: same fallback behavior

```js
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
    return cachedFreeModels ?? FALLBACK_FREE_MODELS
  }
}
```

### `pickBestModel(ids: string[]) → string`

```js
function pickBestModel(ids) {
  for (const preferred of MODEL_PRIORITY) {
    if (ids.includes(preferred)) return preferred
  }
  return ids[0]
}
```

Selects the first matching model from `MODEL_PRIORITY`, falling back to `ids[0]`.

### `formatModelList(ids: string[]) → object`

```js
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
```

Returns OpenAI-compatible model list format.

### `formatModelEntry(id: string) → object`

```js
function formatModelEntry(id) {
  return {
    id,
    object: 'model',
    created: 1710000000,
    owned_by: 'opencode',
  }
}
```

Returns OpenAI-compatible single model entry format.

### `formatErrorBody(message: string, type?: string) → object`

```js
function formatErrorBody(message, type) {
  return { error: { message, type: type ?? 'not_found' } }
}
```

### `isHopByHopHeader(name: string) → boolean`

```js
const HOP_BY_HOP = new Set([
  'connection', 'keep-alive', 'proxy-authenticate', 'proxy-authorization',
  'te', 'trailer', 'transfer-encoding', 'upgrade',
])

function isHopByHopHeader(name) {
  return HOP_BY_HOP.has(name.toLowerCase())
}
```

---

## 5. State Variables

```js
let cachedFreeModels = null
let freeModelIdsPromise = fetchFreeModels()
```

`freeModelIdsPromise` captures the in-flight fetch so that concurrent requests arriving before the fetch completes all await the same promise — avoids multiple simultaneous fetches. `cachedFreeModels` is populated by the fetch on success and used as stale fallback on subsequent failures.

---

## 6. HTTP Server — Route Handling

The server listens on `127.0.0.1:0` (OS-assigned random port). All routes require the `/zen/v1` prefix — unknown paths return 404.

### Generic handler: `handleRequest(req, res)`

```js
const server = createServer(async (req, res) => {
  const { method, url } = req

  // ── Prefix guard: only handle /zen/v1/* ────────
  if (!url.startsWith('/zen/v1')) {
    res.writeHead(404, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    })
    return res.end(JSON.stringify(formatErrorBody('Not found', 'not_found')))
  }

  // ── Route dispatch ──────────────────────────────
  try {
    if (method === 'GET' && url === '/zen/v1/models') {
      return handleModelsList(req, res)
    }
    if (method === 'GET' && url.startsWith('/zen/v1/models/')) {
      return handleModelById(req, res, url)
    }
    if (method === 'POST' && url === '/zen/v1/chat/completions') {
      return handleChatCompletions(req, res)
    }
    // Catch-all: 404
    res.writeHead(404, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    })
    res.end(JSON.stringify(formatErrorBody('Not found', 'not_found')))
  } catch (error) {
    console.error(`[opencode-free] Route error: ${error.message}`)
    if (!res.headersSent) {
      res.writeHead(500, { 'Content-Type': 'application/json' })
      res.end(JSON.stringify(formatErrorBody('Internal server error', 'server_error')))
    } else {
      res.destroy()
    }
  }
})
```

### `handleModelsList(req, res)`

```js
async function handleModelsList(req, res) {
  const ids = await freeModelIdsPromise
  res.writeHead(200, {
    'Content-Type': 'application/json',
    'Access-Control-Allow-Origin': '*',
  })
  res.end(JSON.stringify(formatModelList(ids)))
}
```

### `handleModelById(req, res, url)`

```js
async function handleModelById(req, res, url) {
  const modelId = url.slice('/zen/v1/models/'.length)
  const ids = await freeModelIdsPromise
  if (ids.includes(modelId)) {
    res.writeHead(200, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    })
    res.end(JSON.stringify(formatModelEntry(modelId)))
  } else {
    res.writeHead(404, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    })
    res.end(JSON.stringify(formatErrorBody(`Model '${modelId}' not found`)))
  }
}
```

### `handleChatCompletions(req, res)` — Proxy Logic

**Flow:**

1. Generate fresh pi-opencode-zen headers via `buildOpenCodeHeaders()`
2. Copy `content-type` from incoming request (if present)
3. **Do NOT** forward `authorization` header — strip it
4. **Do NOT** buffer request body — pipe it directly via `req.pipe(proxyReq)`
5. Make `https.request` to `opencode.ai/zen/v1/chat/completions`:
   - Method: POST (from incoming request)
   - Path: `/zen/v1/chat/completions` (same as incoming path)
   - Headers: pi-opencode-zen headers + `content-type`
   - `rejectUnauthorized: true` (default)
6. On upstream response:
   - Forward the upstream status code
   - Strip hop-by-hop headers from upstream response headers
   - Forward remaining upstream response headers
   - Add CORS header: `Access-Control-Allow-Origin: *`
   - Pipe upstream response to client: `upstreamResponse.pipe(res)`
7. On upstream error:
   - If `res.headersSent` is false: write 502 with JSON error body
   - If `res.headersSent` is true: `res.destroy()` (avoid crash from late write)

```js
async function handleChatCompletions(req, res) {
  const opencodeHeaders = buildOpenCodeHeaders()

  // Copy content-type from incoming request
  if (req.headers['content-type']) {
    opencodeHeaders['content-type'] = req.headers['content-type']
  }

  // NOTE: Authorization is intentionally NOT forwarded
  // pi-opencode-zen does not send auth to streaming endpoints

  const proxyReq = httpsRequest({
    hostname: OPCODE_ZEN_HOST,
    port: OPCODE_ZEN_PORT,
    path: '/zen/v1/chat/completions',
    method: 'POST',
    headers: opencodeHeaders,
    rejectUnauthorized: true,
  }, (proxyRes) => {
    const responseHeaders = { 'access-control-allow-origin': '*' }
    for (const [key, value] of Object.entries(proxyRes.headers)) {
      if (!isHopByHopHeader(key)) {
        responseHeaders[key] = value
      }
    }
    res.writeHead(proxyRes.statusCode, proxyRes.statusMessage ?? '', responseHeaders)
    proxyRes.pipe(res)
  })

  proxyReq.on('error', (error) => {
    console.error(`[opencode-free] Upstream error: ${error.message}`)
    if (!res.headersSent) {
      res.writeHead(502, {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      })
      res.end(JSON.stringify(formatErrorBody(`Upstream error: ${error.message}`, 'upstream_error')))
    } else {
      res.destroy()
    }
  })

  // Stream incoming body to upstream — zero buffering
  req.pipe(proxyReq)
}
```

---

## 7. Spawn Logic

### `main()` entry point

```js
async function main() {
  const args = process.argv.slice(2)
  const isBg = args.includes('--bg')

  // Fetch free models (populates cache for /v1/models endpoint)
  const freeModelIds = await freeModelIdsPromise
  cachedFreeModels = freeModelIds

  // Start HTTP server
  const server = createServer(handleRequest)

  await new Promise((resolve, reject) => {
    server.listen(0, '127.0.0.1', () => {
      resolve()
    })
    server.on('error', reject)
  })

  const address = server.address()
  const port = address.port

  // Pick best model
  const bestModel = pickBestModel(freeModelIds)

  // Prepare environment
  const env = {
    ...process.env,
    CLAUDE_CODE_USE_OPENAI: '1',
    OPENAI_BASE_URL: `http://127.0.0.1:${port}/zen/v1`,
    OPENAI_API_KEY: 'public',
    OPENAI_MODEL: bestModel,
  }

  // Spawn openclaude
  const child = spawn('/usr/local/bin/openclaude', args.filter(a => a !== '--bg'), {
    env,
    stdio: 'inherit',
  })

  if (isBg) {
    child.unref()
    // Proxy process stays alive
    console.error(`[opencode-free] Proxy running on http://127.0.0.1:${port}`)
    console.error(`[opencode-free] Background mode: openclaude spawned with PID ${child.pid}`)
    return
  }

  // ── Signal handling ──────────────────────────────
  const cleanup = () => {
    child.kill()
    server.close(() => process.exit(0))
    setTimeout(() => process.exit(0), 2000).unref()
  }
  process.on('SIGINT', cleanup)
  process.on('SIGTERM', cleanup)

  // ── Child exit propagation ───────────────────────
  child.on('exit', (code, signal) => {
    server.close(() => process.exit(code ?? (signal ? 1 : 0)))
    setTimeout(() => process.exit(code ?? (signal ? 1 : 0)), 2000).unref()
  })
}
```

### `--bg` mode behavior

If `process.argv.includes('--bg')`:
- `child.unref()` — Node.js won't wait for the child to exit before the process exits
- The proxy process stays alive and continues serving requests
- No signal handlers registered (background mode, no terminal to receive signals)
- Print proxy port and child PID to stderr for diagnostic purposes

---

## 8. Signal Handling (Interactive Mode)

When NOT in `--bg` mode:

| Signal | Action |
|--------|--------|
| `SIGINT` | Kill child, close server, exit with 0 |
| `SIGTERM` | Same as SIGINT |

Both signals registered via `process.on()`. If server.close() hangs, force exit after 2 seconds.

**Child process exit** handler:
- On child exit (any code): close server, then `process.exit(childCode)`
- If child was killed by a signal: exit with code 1
- 2 second force-exit timeout as fallback

---

## 9. Error Handling Map

| Scenario | Behavior |
|----------|----------|
| models.dev fetch fails at startup (no cache) | Log warning to stderr, use `FALLBACK_FREE_MODELS`, continue |
| models.dev fetch fails (stale cache exists) | Log warning to stderr, **preserve stale cached data**, continue |
| models.dev returns empty list | Log warning, use `FALLBACK_FREE_MODELS` |
| Port binding fails (port 0 never fails, but handle anyway) | Log error, exit with code 1 |
| Upstream opencode.ai returns error response | Forward exact status code + headers + body to client (transparent) |
| Upstream connection failure (network down, DNS) | `proxyReq.on('error')`: if `!res.headersSent`, send 502; else `res.destroy()` |
| Incoming request path is not `/zen/v1/*` | Return 404 with JSON error body |
| Incoming request method/path combination unknown | Return 404 with JSON error body |
| Child process crashes or exits with non-zero | Close server, exit with same code |
| SIGINT/SIGTERM in interactive mode | Kill child, close server, exit with 0 within 2s |
| Upstream TLS error | Caught by `proxyReq.on('error')` — treated as upstream connection failure |
| Upstream rate-limiting (429) | Forwarded transparently to client |
| Upstream sends chunked response | `pipe()` handles it natively — no manual chunk management |
| `handleRequest` catch-all error | If `!res.headersSent`, send 500; else `res.destroy()` |

---

## 10. Complete Code Structure

```
opencode-free-setup.mjs
├── Imports
│   ├── createServer from 'node:http'
│   ├── request as httpsRequest from 'node:https'
│   ├── spawn from 'node:child_process'
│   ├── crypto from 'node:crypto'
│   └── process from 'node:process'
├── Constants
│   ├── MODELS_DEV_URL      → 'https://models.dev/api.json'
│   ├── OPCODE_ZEN_HOST     → 'opencode.ai'
│   ├── OPCODE_ZEN_PORT     → 443
│   ├── FALLBACK_FREE_MODELS → string[]
│   ├── MODEL_PRIORITY      → string[]
│   └── HOP_BY_HOP          → Set<string> (hop-by-hop header names)
├── State
│   ├── cachedFreeModels    → string[] | null
│   └── freeModelIdsPromise → Promise<string[]> (started at module load)
├── Utility functions
│   ├── generateId()                        → 26-char hex string
│   ├── buildOpenCodeHeaders()              → { User-Agent, x-opencode-*, ... }
│   ├── fetchFreeModels()                   → Promise<string[]>
│   ├── pickBestModel(ids)                  → string
│   ├── formatModelList(ids)                → { object, data: [...] }
│   ├── formatModelEntry(id)                → { id, object, created, owned_by }
│   ├── formatErrorBody(msg, type?)         → { error: { message, type } }
│   └── isHopByHopHeader(name)              → boolean
├── Route handlers
│   ├── handleModelsList(req, res)          → void (returns cached model list)
│   ├── handleModelById(req, res, url)      → void (single model or 404)
│   ├── handleChatCompletions(req, res)     → void (proxies to opencode.ai)
│   └── handleRequest(req, res)             → void (dispatcher)
└── main()
    ├── Parse args, detect --bg
    ├── Await freeModelIdsPromise → populate cachedFreeModels
    ├── Create server, listen on port 0
    ├── Get assigned port from server.address()
    ├── Pick best model
    ├── Build env vars (CLAUDE_CODE_USE_OPENAI, OPENAI_BASE_URL, etc.)
    ├── Spawn openclaude with stdio: 'inherit'
    ├── If --bg: child.unref(), print port/PID to stderr, return
    ├── Register SIGINT/SIGTERM → kill child + close server + 2s timeout
    └── child.on('exit') → close server + process.exit(child code) + 2s timeout
```

---

## 11. Testing Approach

### A. Automated test script (`test-opencode-free.mjs`)

Use a mirror server to verify request headers, model list filtering, and streaming:

```js
import { createServer } from 'node:http'
import { spawn } from 'node:child_process'
import process from 'node:process'

const MIRROR_PORT = 3199  // mock opencode.ai
const PROXY_SCRIPT = './opencode-free-setup.mjs'

let testsPassed = 0
let testsFailed = 0

async function assert(label, condition) {
  if (condition) { testsPassed++; console.error(`  ✓ ${label}`) }
  else { testsFailed++; console.error(`  ✗ ${label}`) }
}
```

**Test 1: Header injection fidelity**
- Start mirror server on port 3199
- Start proxy with `OPENAI_BASE_URL=http://127.0.0.1:3199` (override to test against mirror)
- Send POST /zen/v1/chat/completions
- Verify mirror received: correct User-Agent, x-opencode-client=cli, valid 26-char hex IDs in session/project/request, NO Authorization header
- Verify session, project, request IDs are all different from each other
- Make two requests and verify request IDs differ between them

**Test 2: Model list filtering**
- Start proxy
- GET /zen/v1/models → verify response is `{ object: "list", data: [...] }`
- Verify every entry has `object: "model"` and a non-empty `id`
- Verify paid model IDs (e.g., "claude-opus") do NOT appear
- GET /zen/v1/models/:id for an existing model → 200 + correct entry
- GET /zen/v1/models/:id for a non-existent model → 404 + error body

**Test 3: Request body streaming**
- Start mirror server that records the POST body
- Send large body through proxy
- Verify mirror received exact body byte-for-byte

**Test 4: Hop-by-hop header stripping**
- Start mirror, serve response with connection, keep-alive, transfer-encoding headers
- Verify proxy client receives stripped versions

**Test 5: Error handling**
- Point proxy at unreachable upstream
- Send POST → verify 502 response with JSON error body
- Verify headersSent guard: if headers already sent, proxy does NOT crash

**Test 6: Path filtering**
- GET /unknown → 404 with JSON error body
- GET /v1/models (without /zen/v1 prefix) → 404

### B. Integration test (manual)

```bash
node opencode-free-setup.mjs
# openclaude starts, connected through local proxy
# Type /model — should show free models
# Type a question — should stream response
```

### C. Background mode test

```bash
node opencode-free-setup.mjs --bg &
sleep 3
# Test proxy still works
curl -s http://127.0.0.1:$(ss -tlnp | grep opencode | grep -oP ':\K\d+')/zen/v1/models
# Kill it
pkill -f "opencode-free-setup"
```

### D. Profile file untouched verification

```bash
HASH_BEFORE=$(md5sum ~/.openclaude/.openclaude-profile.json 2>/dev/null || echo "NO_FILE")
node opencode-free-setup.mjs --bg &
sleep 3
pkill -f "opencode-free-setup"
HASH_AFTER=$(md5sum ~/.openclaude/.openclaude-profile.json 2>/dev/null || echo "NO_FILE")
# HASH_BEFORE === HASH_AFTER (both "NO_FILE" or identical hash)
```

### E. Golden request capture (ultimate verification)

```bash
tcpdump -i lo0 -X port 3199 > pi-zen-dump.txt
# Compare with proxy dump against same mirror
diff pi-zen-dump.txt proxy-dump.txt
```

---

## 12. Implementation Notes

1. **Port detection**: `server.listen(0)` with `await`, then `server.address().port` is synchronous after the callback fires. Port is stable for the proxy's lifetime.

2. **`req.pipe(proxyReq)`**: Node.js `IncomingMessage` (the `req` object) is a readable stream. `http.ClientRequest` is a writable stream. `req.pipe(proxyReq)` wires them together with backpressure. The request body is never buffered in memory.

3. **`proxyRes.pipe(res)`**: The upstream response is a readable stream. `res` (ServerResponse) is a writable stream. `pipe()` handles backpressure, chunking, and SSE streaming correctly.

4. **No `Transfer-Encoding` forwarding**: Transfer-Encoding is a hop-by-hop header and is stripped. Node.js handles chunked encoding automatically for the response to the client.

5. **`headersSent` guard**: Without this guard, if the upstream connection fails after we've already started sending response headers (or body), calling `res.writeHead()` would throw an uncaught exception. The guard checks `res.headersSent` before writing.

6. **CORS headers**: `Access-Control-Allow-Origin: *` is added to every response (model list, single model, error responses, proxied responses). This ensures the proxy works when openclaude runs in a browser-based context.

7. **`freeModelIdsPromise` at module scope**: Starting the fetch at module load means the fetch begins before `main()` runs, minimizing startup latency. The `await` in `main()` resolves almost immediately if the fetch completes quickly.

8. **Auth stripping**: The `Authorization` header from openclaude is never forwarded. This matches pi-opencode-zen's behavior where free model streaming endpoints receive no auth. The `OPENAI_API_KEY=public` env var satisfies openclaude's credential check without sending real credentials upstream.

9. **No temp files**: Unlike the previous approach (temp dirs + profile file manipulation), this approach uses zero disk I/O beyond the script itself. The dynamic port is communicated via `server.address().port` returned synchronously — no sentinel file needed because `main()` is sequential.

10. **Fallback list freshness**: The hardcoded `FALLBACK_FREE_MODELS` should be updated when new free models are released. This is a manual maintenance task — no automatic re-fetching during proxy lifetime.

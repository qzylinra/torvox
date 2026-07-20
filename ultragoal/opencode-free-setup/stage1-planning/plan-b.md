# Plan B: Hybrid Node.js Proxy + `--require` Hook Approach

## Core Idea

Instead of a standalone proxy server, use two cooperating mechanisms:

1. **`--require` preload hook** that patches `http.request` / `https.request` to inject pi-opencode-zen headers into any outbound request matching `opencode.ai`
2. **A tiny model-filtering proxy** that only handles `/v1/models` — intercepting the model list response and filtering to free models

This avoids running a full-duplex proxy for every streaming response while still satisfying the "同样的http请求" requirement at the HTTP level.

---

## Three Approaches — Tradeoff Analysis

| Criterion | Proxy (standalone) | `--require` Hook | Env-only (current) |
|---|---|---|---|
| Headers match exactly | ✅ Full control | ✅ Full control | ❌ Missing all custom headers |
| Model filtering | ✅ Intercept `/v1/models` | ❌ Cannot intercept response | ✅ Client-side filter (no server check) |
| Streaming SSE | ✅ Must pipe raw bytes | ✅ Zero-copy (requests pass through normally) | ✅ Direct, no indirection |
| Works outside Node.js | ✅ Any HTTP client | ❌ Node.js only | ✅ Any client |
| Setup complexity | Medium (server lifecycle) | Low (one file) | Minimal |
| `openclaude` integration | Must set `OPENAI_BASE_URL` to proxy | Must add `--require` flag to launch | Must set env vars only |
| Reliability risk | Connection drops, port conflicts | Zero (no network move) | Zero |
| Anti-detection | Headers correct, extra `Host` reflects proxy | Headers correct, `Host` is real `opencode.ai` | Headers wrong — easy to fingerprint as non-CLI |
| Maintainability | Full HTTP server | Single 50-line monkey-patch | No moving parts |

**Recommendation for "同样的http请求":**
- The proxy approach wins for exact fidelity at the TCP level (same destination IP, same TLS handshake after CONNECT tunnel)
- The hook approach wins for simplicity — headers are injected, but the TCP/ TLS is still direct to opencode.ai
- The env-only approach fails the core requirement: no custom headers are sent at all, making requests trivially distinguishable from pi-opencode-zen

**This plan pursues a hybrid that gets the best of both: hook for headers, tiny proxy endpoint only for model filtering.**

---

## Approach: `--require` Hook for Headers + Thin Model Proxy

### Architecture

```
openclaude (Node.js process)
  │
  ├── [--require] pi-opencode-hook.mjs
  │     └── patches http/https to inject headers on *.opencode.ai
  │
  ├── API calls to opencode.ai/zen/v1/... (HEADERS INJECTED, pass-through)
  │
  └── GET /zen/v1/models
        └── → localhost:${PORT}/v1/models → proxy fetches from opencode.ai, filters models

Startup script sets:
  OPENAI_BASE_URL=http://localhost:${PORT}/zen/v1
  (only /v1/models goes through proxy; other paths just merge headers via hook)
```

Wait — this is more complex than necessary. Let me simplify.

### Revised Architecture

**Single entry point: `proxy.mjs`** — starts a local HTTP proxy that:
- Forwards ALL requests to `opencode.ai/zen/v1/...`
- Injects pi-opencode-zen headers identically
- Intercepts `/v1/models` to filter free models
- Streams everything else (chat completions, SSE) byte-for-byte

The `--require` hook idea is elegant but doesn't solve model filtering, and for the specific case where we need both header injection AND model filtering, a proxy is the cleanest single mechanism. The hook would be better in a scenario where no response modification is needed.

---

## Proxy Code Structure

### File: `opencode-free-proxy.mjs`

```js
import { createServer } from 'node:http'
import { request as httpRequest } from 'node:http'
import { request as httpsRequest } from 'node:https'
import { randomUUID } from 'node:crypto'

const OPCODE_ZEN_HOST = 'opencode.ai'
const OPCODE_ZEN_PORT = 443
const LOCAL_PORT = parseInt(process.env.PROXY_PORT || '3180', 10)
const MODELS_DEV_URL = 'https://models.dev/api.json'

// ── pi-opencode-zen header generation ──────────────────────────

function opencodeHeaders() {
  const id = () => randomUUID().replace(/-/g, '').slice(0, 26)
  return {
    'User-Agent': 'opencode/latest/1.3.15/cli',
    'x-opencode-client': 'cli',
    'x-opencode-session': id(),
    'x-opencode-project': id(),
    'x-opencode-request': id(),
  }
}

// ── Free model list from models.dev ────────────────────────────

let cachedFreeModels = null
let lastFetchTime = 0
const CACHE_TTL_MS = 300_000  // 5 minutes

async function getFreeModelIds() {
  const now = Date.now()
  if (cachedFreeModels && (now - lastFetchTime) < CACHE_TTL_MS) return cachedFreeModels
  try {
    const res = await fetch(MODELS_DEV_URL)
    if (!res.ok) throw new Error(`models.dev returned ${res.status}`)
    const data = await res.json()
    const freeIds = []
    for (const [name, info] of Object.entries(data?.opencode?.models ?? {})) {
      if (info?.cost?.input === 0 && info?.status === 'active') freeIds.push(name)
    }
    if (freeIds.length > 0) {
      cachedFreeModels = freeIds
      lastFetchTime = now
    }
    return freeIds
  } catch (err) {
    console.error(`[proxy] models.dev fetch failed: ${err.message}`)
    return cachedFreeModels ?? FALLBACK_FREE_MODELS
  }
}

const FALLBACK_FREE_MODELS = [
  'deepseek-v4-flash-free', 'qwen3.6-plus-free', 'glm-5',
  'nemotron-3-super-free', 'big-pickle', 'minimax-m2.5-free',
  'kimi-k2.5', 'kimi-k2', 'kimi-k2-thinking',
  'glm-4.7', 'glm-4.6', 'minimax-m2.1', 'trinity-large-preview-free',
]

// Fetched once at startup for `/v1/models` interception
let freeModelIdsPromise = getFreeModelIds()

// ── Proxy server ───────────────────────────────────────────────

const server = createServer(async (req, res) => {
  // Only handle /zen/v1/* paths
  if (!req.url.startsWith('/zen/v1')) {
    res.writeHead(404)
    return res.end('Not found')
  }

  const targetPath = req.url  // keep /zen/v1 prefix
  const headers = {
    ...opencodeHeaders(),
    ...(req.headers.host ? {} : {}),
  }

  // Copy over auth header if present (for /zen/v1/models)
  if (req.headers.authorization) {
    headers.authorization = req.headers.authorization
  }

  // Handle content-type, content-length for POST bodies
  if (req.headers['content-type']) {
    headers['content-type'] = req.headers['content-type']
  }
  if (req.headers['content-length']) {
    headers['content-length'] = req.headers['content-length']
  }

  // ── Intercept /zen/v1/models to filter free models ─────

  if (targetPath === '/zen/v1/models' && req.method === 'GET') {
    try {
      const freeIds = await freeModelIdsPromise
      const modelData = {
        object: 'list',
        data: freeIds.map(id => ({ id, object: 'model' })),
      }
      res.writeHead(200, {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      })
      return res.end(JSON.stringify(modelData))
    } catch (err) {
      res.writeHead(502, { 'Content-Type': 'text/plain' })
      return res.end(`Proxy error: ${err.message}`)
    }
  }

  // ── All other requests: stream through to opencode.ai ───

  const options = {
    hostname: OPCODE_ZEN_HOST,
    port: OPCODE_ZEN_PORT,
    path: targetPath,
    method: req.method,
    headers,
    rejectUnauthorized: true,
  }

  const proxyReq = httpsRequest(options, (proxyRes) => {
    // Forward status + headers
    const responseHeaders = { ...proxyRes.headers }
    // Strip hop-by-hop headers
    delete responseHeaders['transfer-encoding']
    delete responseHeaders['connection']
    delete responseHeaders['keep-alive']
    delete responseHeaders['proxy-authenticate']
    delete responseHeaders['proxy-authorization']
    delete responseHeaders['te']
    delete responseHeaders['trailer']
    delete responseHeaders['upgrade']

    res.writeHead(proxyRes.statusCode, responseHeaders)

    // CRITICAL: stream raw bytes — no buffering
    proxyRes.pipe(res)
  })

  proxyReq.on('error', (err) => {
    console.error(`[proxy] upstream error: ${err.message}`)
    if (!res.headersSent) {
      res.writeHead(502, { 'Content-Type': 'text/plain' })
      res.end(`Upstream error: ${err.message}`)
    } else {
      res.destroy()
    }
  })

  // Pipe request body to upstream
  req.pipe(proxyReq)
})

server.listen(LOCAL_PORT, '127.0.0.1', () => {
  console.error(`[proxy] listening on http://127.0.0.1:${LOCAL_PORT}`)
  console.error(`[proxy] models.dev free models: ${cachedFreeModels?.length ?? 'fetching...'}`)
})

// ── Graceful shutdown ──────────────────────────────────────────

function shutdown() {
  server.close()
  process.exit(0)
}
process.on('SIGINT', shutdown)
process.on('SIGTERM', shutdown)
```

---

## Streaming SSE Handling — Why This Works

The critical insight: **SSE works through a proxy automatically when you pipe raw bytes.**

```
openclaude → proxy (localhost:3180/zen/v1/chat/completions)
          → proxy injects headers
          → HTTPS to opencode.ai/zen/v1/chat/completions
          → opencode.ai streams SSE chunks back
          → proxy pipes chunks verbatim to openclaude
```

Key guarantees:
- `req.pipe(proxyReq)` — forwards POST body without buffering
- `proxyRes.pipe(res)` — forwards response chunks without buffering
- No `transfer-encoding` manipulation — let Node.js handle chunked encoding naturally
- The `connection` header is stripped to prevent half-close issues

What breaks streaming:
- Buffering the entire response before forwarding (kills real-time SSE)
- Rewriting `Content-Type` (must preserve `text/event-stream`)
- Adding `Content-Length` to a chunked response (breaks HTTP)
- Not stripping `Connection: keep-alive` when proxy doesn't support persistent connections

All of these are avoided by the raw `pipe()` approach above.

---

## Model Filtering — Exact Path

### The `/v1/models` intercept works because:

1. openclaude calls `GET /zen/v1/models` on startup to discover available models
2. The proxy intercepts this BEFORE it reaches opencode.ai
3. Returns a synthetic response with only free model IDs
4. openclaude then auto-selects the first model from the list (or user picks)

### How openclaude uses the model list:

openclaude calls the OpenAI-compatible `/models` endpoint at startup. The response `{ data: [{ id: "..." }, ...] }` is the same shape as OpenAI's API. The proxy returns only free models, which means openclaude will:
- Show only free models in its model picker UI
- Auto-select the first free model as default
- Never attempt a paid model (even if user manually types the name, the proxy can reject non-free model requests at the `/v1/models` level)

### Stale model ID detection:

If a user hardcodes `OPENAI_MODEL=claude-sonnet-4` (a paid model) via env var, the proxy should reject it at the chat completion endpoint. Enhancement:

```js
// In the streaming handler, before forwarding:
if (req.method === 'POST' && targetPath.endsWith('/chat/completions')) {
  const body = await collectBody(req)  // buffer to check model
  const parsed = JSON.parse(body)
  const freeIds = await freeModelIdsPromise
  if (!freeIds.includes(parsed.model)) {
    res.writeHead(403, { 'Content-Type': 'application/json' })
    return res.end(JSON.stringify({
      error: { message: `Model '${parsed.model}' is not a free model. Available: ${freeIds.join(', ')}` }
    }))
  }
  // Re-create proxyReq with the body
}
```

However, this requires buffering the POST body, which adds latency. **Better approach**: trust the `/v1/models` filtering + the user's `OPENAI_MODEL` env var. If they manually override to a paid model, that's their choice. The proxy's job is to provide the right model list by default.

---

## Setup Script

### File: `openclaude-free.mjs`

```js
#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const PROXY_PORT = process.env.PROXY_PORT || '3180'

// Start proxy as sidecar
const proxyPath = resolve(__dirname, 'opencode-free-proxy.mjs')
const proxy = spawn(process.execPath, [proxyPath], {
  stdio: ['ignore', 'inherit', 'inherit'],
  env: { ...process.env, PROXY_PORT },
})

// Wait for proxy to be ready
await new Promise((resolve, reject) => {
  const check = () => {
    const http = require('http')
    http.get(`http://127.0.0.1:${PROXY_PORT}/zen/v1/health`, (res) => {
      resolve()
    }).on('error', () => setTimeout(check, 100))
  }
  setTimeout(() => reject(new Error('Proxy failed to start')), 10000)
  proxy.on('exit', (code) => reject(new Error(`Proxy exited with code ${code}`)))
  check()
})

// Launch openclaude pointing at proxy
const child = spawn('openclaude', process.argv.slice(2), {
  stdio: 'inherit',
  env: {
    ...process.env,
    OPENAI_BASE_URL: `http://127.0.0.1:${PROXY_PORT}/zen/v1`,
    OPENCODE_API_KEY: 'public',
    CLAUDE_CODE_USE_OPENAI: '1',
  },
})

const cleanup = () => { child.kill(); proxy.kill() }
process.on('SIGINT', cleanup)
process.on('SIGTERM', cleanup)

child.on('exit', (code) => {
  proxy.kill()
  process.exit(code ?? 0)
})
```

---

## Anti-Detection — What Matters Most

| Factor | Impact | What We Do |
|--------|--------|-----------|
| Header names + values | **Critical** — server fingerprints by User-Agent | Match pi-opencode-zen exactly: `User-Agent: opencode/latest/1.3.15/cli` |
| `x-opencode-*` header values format | **Critical** — server validates UUID format | Use `randomUUID().replace(/-/g,'').slice(0,26)` — identical to pi-opencode-zen |
| `x-opencode-*` header uniqueness per request | **Critical** — detects replay | Generate fresh IDs for every request |
| `Host` header | Low — proxy changes it from `localhost:3180` to `opencode.ai` | Node.js `http.request` auto-sets `Host` to `hostname` option value |
| TLS fingerprint | Low — Node.js https uses same OpenSSL as the CLI, similar JA3 | No special handling needed |
| Header ordering | Low — HTTP headers are unordered by spec | No special handling needed |
| Timing / request pattern | Very Low — server doesn't profile timing | No special handling needed |
| Source IP / geolocation | Not applicable — local proxy to same machine | N/A |

**Summary**: The two things that would get us detected are (a) missing or wrong User-Agent, and (b) stale/invalid session IDs. Everything else is secondary.

---

## Edge Cases

### 1. models.dev is unreachable

- The proxy falls back to `FALLBACK_FREE_MODELS` hardcoded list
- Prints a warning to stderr
- The fallback list should be updated periodically (manually, when models change)
- The `/v1/models` endpoint still returns a valid subset

### 2. opencode.ai changes API

- If the `/zen/v1` path changes, the `OPENAI_BASE_URL` env var becomes the single point of change
- If the `/v1/models` response shape changes, the proxy's intercept needs updating
- If the custom header requirements change, update `opencodeHeaders()`
- **Mitigation**: The proxy is a small, focused file — easy to update

### 3. User has existing openclaude config

- The script sets `CLAUDE_CODE_USE_OPENAI=1` as an env var, which takes highest priority in openclaude's provider selection (see `exploration-openclaude.md` sections 8-12)
- The profile file at `~/.openclaude/.openclaude-profile.json` is NEVER touched
- When the user runs `openclaude` normally (without this script), their existing config is used
- **No interference**: env-var-based selection short-circuits profile file reading

### 4. OpenClaude spawns child processes

- If openclaude spawns subprocesses, they inherit modified env vars
- The proxy is always running on the same port, so child requests work
- However, if a child process spawns its OWN Node.js (e.g., openclaude's `--bg` mode), the `OPENAI_BASE_URL` still points to the proxy
- **Risk**: If the proxy dies before the child, requests fail — solution: proxy health check in the child process or use `TcpProxy` from the spawner

### 5. Port conflicts

- Default port 3180 may be in use
- **Solution**: Let the OS assign a port (`port: 0`), then print the assigned port to a temp file for the parent to read

```js
// Proxy side — write port to a sentinel file
server.listen(0, '127.0.0.1', () => {
  const port = server.address().port
  fs.writeFileSync(PORT_FILE, String(port))
  console.error(`[proxy] listening on port ${port}`)
})

// Launcher side — read port from sentinel file
const port = fs.readFileSync(PORT_FILE, 'utf8').trim()
```

---

## Concrete File Layout

```
opencode-free-setup/           ← in /tmp/opencode or project root
├── openclaude-free.mjs        ← Entry point: starts proxy + spawns openclaude
├── opencode-free-proxy.mjs    ← Proxy server: header injection + model filtering
├── pi-opencode-hook.mjs       ← ALTERNATIVE: --require hook (not used in default path)
└── test/
    └── test-proxy.mjs         ← Verification tests
```

---

## Testing Approach — How to Verify "同样的http请求"

### 1. Capture-and-compare test

Use a mirror server that records the exact HTTP request and compares it against what pi-opencode-zen would send:

```js
// test/test-proxy.mjs
import { createServer } from 'node:http'
import { request } from 'node:http'

// Start a mirror that records the request
const mirrorPort = 3199
const recorded = {}
const mirror = createServer((req, res) => {
  let body = ''
  req.on('data', c => body += c)
  req.on('end', () => {
    recorded[req.url] = {
      method: req.method,
      headers: { ...req.headers },
      body,
    }
    res.end('ok')
  })
})
await new Promise(r => mirror.listen(mirrorPort, r))

// Send request through proxy
const proxyPort = 3180
const result = await fetch(`http://127.0.0.1:${proxyPort}/zen/v1/chat/completions`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ model: 'deepseek-v4-flash-free', messages: [{ role: 'user', content: 'hi' }] }),
})

// Verify headers match expected pattern
const sent = recorded['/zen/v1/chat/completions']
assert(sent.headers['user-agent'] === 'opencode/latest/1.3.15/cli')
assert(sent.headers['x-opencode-client'] === 'cli')
assert(/^[0-9a-f]{26}$/.test(sent.headers['x-opencode-session']))
assert(/^[0-9a-f]{26}$/.test(sent.headers['x-opencode-project']))
assert(/^[0-9a-f]{26}$/.test(sent.headers['x-opencode-request']))
// Verify IDs are different per request
assert(sent.headers['x-opencode-session'] !== sent.headers['x-opencode-request'])

mirror.close()
```

### 2. Model list filtering test

```js
// Hit /v1/models through proxy
const res = await fetch(`http://127.0.0.1:${proxyPort}/zen/v1/models`)
const data = await res.json()
// Verify only free models returned
assert(data.object === 'list')
assert(data.data.length > 0)
assert(data.data.every(m => m.object === 'model'))
// Verify no paid models (e.g., claude-opus-4-1) appear
const paidModels = data.data.filter(m => m.id.includes('opus') || m.id.includes('sonnet-4'))
assert(paidModels.length === 0, `Found paid models: ${paidModels.map(m => m.id)}`)
```

### 3. Streaming SSE end-to-end test

```js
const res = await fetch(`http://127.0.0.1:${proxyPort}/zen/v1/chat/completions`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    model: 'deepseek-v4-flash-free',
    messages: [{ role: 'user', content: 'Say hello' }],
    stream: true,
  }),
})
assert(res.ok)
const reader = res.body.getReader()
let chunkCount = 0
while (true) {
  const { done, value } = await reader.read()
  if (done) break
  chunkCount++
}
assert(chunkCount > 5, `Expected many SSE chunks, got ${chunkCount}`)
```

### 4. Full integration smoke test

```bash
PROXY_PORT=3180 node opencode-free-proxy.mjs &
PROXY_PID=$!
sleep 1

# Test model list
curl -s http://127.0.0.1:3180/zen/v1/models | jq '.data | length'
# Should print about 10-14 (free models count)

# Test chat completion
curl -s http://127.0.0.1:3180/zen/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"hi"}]}' \
  | jq '.choices[0].message.content'

# Test streaming
curl -sN http://127.0.0.1:3180/zen/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"count to 5"}],"stream":true}'

kill $PROXY_PID
```

### 5. Golden request capture

For ultimate verification, run pi-opencode-zen in a controlled environment with a TCP mirror, capture the exact bytes it sends, then compare byte-for-byte against what the proxy sends for the same request body:

```bash
# Record what pi-opencode-zen sends
tcpdump -i lo0 -X port 3199 > pi-zen-dump.txt
# Record what our proxy sends (same mirror)
tcpdump -i lo0 -X port 3199 > proxy-dump.txt
diff pi-zen-dump.txt proxy-dump.txt
```

This is the gold standard for "同样的http请求" verification.

---

## Error Handling Strategy

| Failure Mode | Detection | Recovery |
|---|---|---|
| Proxy startup fails | Timeout in launcher | Exit with error message |
| Upstream (opencode.ai) unreachable | `proxyReq.on('error')` | Return 502 with JSON error body |
| `models.dev` fetch fails | `getFreeModelIds()` throws | Use fallback hardcoded list |
| Upstream returns 4xx/5xx | Passed through transparently | No special handling — openclaude handles it |
| Proxy killed mid-request | TCP connection drops | openclaude retries or shows error |
| Port already in use | `server.listen` throws | Launcher detects exit, suggests `PROXY_PORT=N` |
| Request body too large | Node.js streams handle it | No buffering, so no issue |
| Upstream TLS error | `rejectUnauthorized: true` by default | Return 502 |
| opencode.ai rate-limits proxy IP | 429 passed through to openclaude | openclaude shows rate-limit message |

---

## Comparison With Plan A (when written)

| Dimension | Plan B (Hybrid) | Plan A |
|---|---|---|
| Header injection mechanism | Proxy rewrites all upstream request headers | (TBD) |
| Model filtering | Intercept `/v1/models` response | (TBD) |
| Streaming | Raw pipe — zero-copy | (TBD) |
| `openclaude` integration | Env vars `OPENAI_BASE_URL` → proxy | (TBD) |
| Statefulness | Stateless proxy, cache of free models | (TBD) |
| Port management | Dynamic port with sentinel file | (TBD) |
| Cleanup | SIGINT/SIGTERM kills both proxy + child | (TBD) |

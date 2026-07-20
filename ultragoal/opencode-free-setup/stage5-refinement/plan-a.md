# Plan A: Stage 5 Refinement — Standalone High-Performance Proxy

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

### Target (standalone)
```
┌─ HTTP Proxy Server (full file)
│   • Router: /v1/models, /v1/models/:id, /v1/chat/completions
│             /zen/v1/models, /zen/v1/models/:id, /zen/v1/chat/completions
│   • Forwarding: http.Agent (keepAlive pool) → pipe with backpressure
│   • Request timeout (AbortController, 120s)
│   • SSE streaming passthrough (no buffering)
│   • Health check endpoint (GET /health)
│   • Same: header injection, CORS, hop-by-hop, zero config file impact
│   • Graceful shutdown (SIGINT/SIGTERM → close → exit(0))
```

**Key architectural change:** No `main()`. No `spawn()`. No child process lifecycle. The server IS the process — it starts, prints its address to stderr, and stays alive until killed. This is the same model as `http-server`, `serve`, `webpack-dev-server`, etc.

---

## 2. Exact Line Changes

### Lines to REMOVE

| Lines | Content | Reason |
|-------|---------|--------|
| 3 | `import { spawn } from 'node:child_process'` | No child processes |
| 207–261 | `async function main() { ... }\nmain()...` | Entire spawner (env setup, spawn, signal handlers, child lifecycle) |

**Total: 56 lines removed** (line 3 + lines 207–261)

### Lines to MODIFY

| Lines | Change | Details |
|-------|--------|---------|
| 1 | `import { createServer } from 'node:http'` → add `Agent` import | `import { createServer, request as httpRequest } from 'node:http'` |
| 2 | `import { request as httpsRequest } from 'node:https'` | Keep but add `Agent` import: `import { request as httpsRequest, Agent } from 'node:https'` |
| 7 | Add `const REQUEST_TIMEOUT_MS = 120_000` | 2 minutes max per request |
| 8 | Add `const UPSTREAM_AGENT = new Agent({ keepAlive: true, keepAliveMsecs: 30_000, maxSockets: 32 })` | Connection pooling config |
| 9 | Add `const DEFAULT_PORT = parseInt(process.env.PORT, 10) || 8080` | User-configurable port |
| 10 | Add `const DEFAULT_HOST = process.env.HOST || '127.0.0.1'` | User-configurable host |
| 30 | Change `freeModelIdsPromise` to lazy init | Only fetch when first `/models` request arrives, not at startup |
| 102–109 | `handleModelsList` | No change (already correct) |
| 111–127 | `handleModelById` | No change |
| 129–168 | `handleChatCompletions` | **Major rewrite** — see section 2.1 |
| 170–205 | Router + server creation | **Rewrite** — see section 2.2 |

### Lines to ADD (new file position)

After line 168 (after enhanced `handleChatCompletions`):

| New Lines | Content |
|-----------|---------|
| ~5 | `handleHealth` — returns `{ status: "ok" }` on GET /health |
| ~2 | `isOpenAIPath(url)` — returns true if url starts with `/v1/` or `/zen/v1/` |
| ~15 | Normalized router: map both `/v1/...` and `/zen/v1/...` to same handlers |
| ~15 | Server startup: `server.listen(DEFAULT_PORT, DEFAULT_HOST, callback)` |
| ~8 | Graceful shutdown: `process.on('SIGINT')` / `process.on('SIGTERM')` → `server.close()` |

### 2.1 Enhanced `handleChatCompletions` (lines 129–168 rewrite)

Current behavior:
- Creates `https.request(...)` with pi-opencode-zen headers
- `proxyRes.pipe(res)` — simple pipe, no backpressure awareness
- `req.pipe(proxyReq)` — pipes body upstream
- Error handler: 502 or res.destroy()
- No timeout mechanism

New behavior:
- **AbortController** for timeout: `AbortSignal.timeout(120_000)` or manual setTimeout
- **Connection pooling**: `agent: UPSTREAM_AGENT` on upstream request
- **Backpressure**: replace `.pipe()` with manual `pipeWithBackpressure()` using `drain` events
- **SSE detection**: if upstream `content-type` includes `text/event-stream`, forward as SSE (no extra buffering)
- **Standard error shapes**: match OpenAI error format spec (`{ error: { message, type, code } }`)
- **Anti-detection**: same fresh IDs per request (unchanged)

### 2.2 Router + Server Rewrite (lines 170–205)

Current:
```js
// Only /zen/v1/... paths
// Single router function with try/catch
```

New:
```js
const server = createServer(async (req, res) => {
  const { method, url } = req

  // CORS preflight
  if (method === 'OPTIONS') {
    res.writeHead(204, corsHeaders())
    return res.end()
  }

  // Health check
  if (method === 'GET' && url === '/health') {
    return handleHealth(res)
  }

  // Normalize path: accept /v1/... or /zen/v1/...
  const normalized = normalizePath(url)
  if (!normalized) {
    return sendJson(res, 404, formatErrorBody('Not found', 'not_found'))
  }

  try {
    if (method === 'GET' && normalized === '/v1/models') return handleModelsList(req, res)
    if (method === 'GET' && normalized.match(/^\/v1\/models\//)) return handleModelById(req, res, url)
    if (method === 'POST' && normalized === '/v1/chat/completions') return handleChatCompletions(req, res)
    return sendJson(res, 404, formatErrorBody('Not found', 'not_found'))
  } catch (error) {
    // ... error handling (same as current)
  }
})
```

Where `normalizePath`:
```js
function normalizePath(url) {
  if (url.startsWith('/zen/v1/')) return '/v1' + url.slice('/zen/v1'.length)
  if (url.startsWith('/v1/')) return url
  return null
}
```

And the upstream path is set to `/zen/v1/chat/completions` always (opencode.ai expects `/zen/v1/...`).

---

## 3. Performance Improvements

### 3.1 Connection Pooling (`https.Agent` with keepAlive)

| Detail | Value |
|--------|-------|
| Implementation | `new Agent({ keepAlive: true, keepAliveMsecs: 30_000, maxSockets: 32 })` |
| Benefit | Reuses TCP connections to opencode.ai → no TLS handshake per request, no TCP slow start |
| Before | Each request creates new socket + TLS handshake (+shutdown after) |
| After | First request → socket; subsequent requests → reused for 30s idle |
| Impact | ~40-60ms latency saved per request (TLS handshake); ~3x throughput at concurrency=10 |

### 3.2 Request Timeout

| Detail | Value |
|--------|-------|
| Implementation | `AbortController` + `setTimeout(120_000)` |
| Benefit | No hung connections; frees pooled socket for next request |
| Impact | Prevents resource exhaustion under load |

### 3.3 Backpressure-Aware Piping

| Detail | Value |
|--------|-------|
| Before | `proxyRes.pipe(res)` — if `res` is slow, data buffers in memory indefinitely |
| After | Manual pipe with `drain` handling — when `res.write()` returns false, pause `proxyRes` until `drain` fires |
| Benefit | Memory-bounded streaming; O(1) buffer per connection regardless of response size |

### 3.4 Concurrent Request Handling

| Detail | Value |
|--------|-------|
| Before | Single global `freeModelIdsPromise` but concurrent requests share it fine (read-only after resolve) |
| After | Same pattern, but `handleChatCompletions` creates no shared mutable state per request |
| Benefit | Unbounded concurrent requests (limited only by Node.js event loop + `maxSockets: 32`) |

### 3.5 Lazy Model Fetch

| Detail | Value |
|--------|-------|
| Before | `freeModelIdsPromise = fetchFreeModels()` at module scope — starts fetching on `import` |
| After | `freeModelIdsPromise` is created on first `/models` request (lazy) |
| Benefit | Proxy starts up faster if `/models` is never fetched (unlikely but correct) |

### 3.6 Expected Performance

| Metric | Before (estimate) | After (target) |
|--------|------------------|----------------|
| Cold start to listen | ~300ms (model fetch on import) | ~20ms |
| P90 latency (single request) | ~800ms (incl. TLS handshake) | ~400ms |
| Concurrent throughput (10 req) | ~3 req/s | ~15 req/s |
| Memory per connection | ~50 KB (no pool) | ~8 KB (reused socket) |
| Max concurrent connections | Limited by ephemeral ports | 32 (pool-limited, graceful degradation) |

---

## 4. Protocol Enhancements

### 4.1 Dual Path Support

```
Client sends    →   Internal normalized   →   Upstream path
/v1/models          /v1/models               /zen/v1/models
/zen/v1/models      /v1/models               /zen/v1/models
/v1/chat/completions /v1/chat/completions     /zen/v1/chat/completions
/zen/v1/chat/...    /v1/chat/completions     /zen/v1/chat/completions
```

Rationale: OpenAI spec says `/v1/...`. openclaude uses `/zen/v1/...`. Supporting both means any OpenAI-compatible client (curl, aider, continue.dev, etc.) can use the proxy without configuration workarounds.

### 4.2 SSE Streaming Proper Handling

```
Upstream SSE stream (text/event-stream)
  │
  ├─ Detect content-type: text/event-stream
  ├─ Set response headers: content-type: text/event-stream
  ├─ Pipe with backpressure (drain-aware)
  ├─ Flush after each data event (res.write() without buffering)
  └─ On error → send error event: data: {"error":{...}}\n\n
```

Key: No `Transform` stream, no accumulation. Raw `data` events forwarded as-is. SSE is line-delimited so this is safe.

### 4.3 Standard Error Format

| Current | New | OpenAI Spec |
|---------|-----|-------------|
| `{ error: { message, type } }` | `{ error: { message, type, code, param } }` | Matches |
| 502: `upstream_error` | 502: `upstream_error` | Same |
| 500: `server_error` | 500: `server_error` | Same |
| 404: `not_found` | 404: `not_found` | Same |
| N/A | 408: `timeout` | New — request timeout |
| N/A | 429: `rate_limit_exceeded` | If upstream returns 429 |

### 4.4 CORS Preflight

| Detail | Value |
|--------|-------|
| Before | No OPTIONS handling |
| After | `OPTIONS *` → `204` with `Access-Control-Allow-Headers: *`, `Access-Control-Allow-Methods: *` |

### 4.5 Health Check

```
GET /health → 200 { status: "ok", uptime: 123, models_fetched: 42 }
```

Useful for Docker health checks, Kubernetes probes, systemd readiness.

---

## 5. File Structure

Single file: `/tmp/opencode/opencode-free-setup.mjs` (same location, updated in-place)

Logical sections within the file (in order):

| Section | Lines (approx) | Description |
|---------|---------------|-------------|
| Imports | 1–5 | `node:http`, `node:https`, `node:crypto`, `node:process` |
| Constants | 6–20 | URLs, model lists, timeout, agent, port/host config |
| Helpers | 21–100 | `generateId`, `buildOpenCodeHeaders`, `fetchFreeModels`, `pickBestModel`, `formatModelList`, `formatModelEntry`, `formatErrorBody`, `isHopByHopHeader`, `corsHeaders`, `sendJson`, `normalizePath` |
| Route Handlers | 101–180 | `handleModelsList`, `handleModelById`, `handleChatCompletions`, `handleHealth` |
| Server | 181–220 | Router (createServer), error handling wrapper |
| Bootstrap | 221–235 | server.listen(), graceful shutdown, process.on(SIGINT/SIGTERM) |

No other files. No config file. No `.env`. No `node_modules`.

---

## 6. Startup / Usage Model

### Running the proxy

```bash
# Default: port 8080, bind 127.0.0.1
node /tmp/opencode/opencode-free-setup.mjs

# Custom port
PORT=3000 node /tmp/opencode/opencode-free-setup.mjs

# Listen on all interfaces (for Docker/network access)
HOST=0.0.0.0 PORT=8888 node /tmp/opencode/opencode-free-setup.mjs

# Background with log file
nohup node /tmp/opencode/opencode-free-setup.mjs > /tmp/proxy.log 2>&1 &
```

### Using the proxy with any client

```bash
# curl
curl http://127.0.0.1:8080/v1/models
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"hi"}]}'

# openclaude
OPENAI_BASE_URL=http://127.0.0.1:8080/v1 \
  OPENAI_API_KEY=public \
  OPENAI_MODEL=deepseek-v4-flash-free \
  CLAUDE_CODE_USE_OPENAI=1 \
  openclaude

# aider
OPENAI_BASE_URL=http://127.0.0.1:8080/v1 \
  OPENAI_API_KEY=public \
  OPENAI_MODEL=deepseek-v4-flash-free \
  aider

# Any OpenAI SDK
client = OpenAI(base_url="http://127.0.0.1:8080/v1", api_key="public")
```

### Stopping the proxy

```bash
kill <PID>          # SIGTERM → graceful shutdown
kill -INT <PID>     # SIGINT  → graceful shutdown
```

### Lifecycle

```
1. Start:    node proxy.mjs
2. Init:     server.listen() → print "[opencode-free] Proxy running on http://127.0.0.1:8080"
3. Running:  Accepts requests, forwards to opencode.ai, returns responses
4. Shutdown: SIGINT/SIGTERM → server.close() → process.exit(0)
```

---

## 7. Cooperation with Existing openclaude Config

### Design Principle: Zero Modification, Mutual Compatibility

The proxy must **not touch** `~/.openclaude/` in any way. The config directory must remain usable by openclaude (or any other client) independently.

### How it works

1. **No profile file created or modified.** Unlike the current script (which also doesn't), the new script never reads `~/.openclaude/` either.

2. **Environment variable-based configuration.** Clients set their own `OPENAI_BASE_URL`, `OPENAI_API_KEY`, `OPENAI_MODEL` when they want to use the proxy. These env vars don't affect `~/.openclaude/`.

3. **Config coexistence example**:

```
# Session A: using proxy (env vars override openclaude config)
OPENAI_BASE_URL=http://127.0.0.1:8080/v1 \
  CLAUDE_CODE_USE_OPENAI=1 \
  openclaude

# Session B: using openclaude normally (no env vars → uses ~/.openclaude/ config)
openclaude
```

Both sessions can run simultaneously. Session A routes through the proxy (free models via opencode.ai). Session B uses openclaude's own config and infrastructure.

4. **Why this works:** The proxy is a network-level intercept. It doesn't modify files, install anything, or leave artifacts. It's purely a process on a port — when it's not running, nothing changes. When it is running, only clients that explicitly point at it are affected.

5. **`~/.openclaude/.openclaude-profile.json`** is never created, read, or written. Zero disk I/O for config.

---

## 8. Testing Approach

### 8.1 Unit Tests (manual, command-line)

All tests use `curl` and standard Unix tools. No test framework required.

**Test 1: Proxy starts and listens**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!
sleep 2
# Check port open
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8080/health
# Should print 200
kill $PID
```

**Test 2: Model list returns only free models**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!
sleep 2
MODELS=$(curl -s http://127.0.0.1:8080/v1/models)
# Verify: object=list, data array, each has id/object/created/owned_by
# Verify: no paid models (gpt-4, claude-3, gemini-ultra not in list)
# Verify: /zen/v1/models returns same as /v1/models
kill $PID
```

**Test 3: Model by ID — found / not found**
```bash
curl -s http://127.0.0.1:8080/v1/models/deepseek-v4-flash-free  # 200
curl -s -w "%{http_code}" http://127.0.0.1:8080/v1/models/nonexistent-model  # 404
```

**Test 4: Chat completion works**
```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Say hello in French, one word"}]}'
# Verify: response has choices[0].message.content
# Verify: response time < 30s
```

**Test 5: SSE streaming**
```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Count to 5"}],"stream":true}'
# Verify: each line starts with "data: "
# Verify: final line is "data: [DONE]"
# Verify: content-type is text/event-stream
```

**Test 6: CORS headers on all responses**
```bash
curl -s -D - http://127.0.0.1:8080/v1/models 2>&1 | grep -i access-control
curl -s -D - -X OPTIONS http://127.0.0.1:8080/v1/chat/completions 2>&1 | grep -i access-control
# Verify: access-control-allow-origin: * on both
```

**Test 7: No child_process dependency**
```bash
grep -c "child_process" /tmp/opencode/opencode-free-setup.mjs
# Should print 0
```

**Test 8: 404 on unknown paths**
```bash
curl -s -w "%{http_code}" http://127.0.0.1:8080/unknown  # 404
curl -s -w "%{http_code}" http://127.0.0.1:8080/v1/unknown  # 404
```

**Test 9: Concurrent requests**
```bash
for i in {1..5}; do
  curl -X POST http://127.0.0.1:8080/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Say hello"}],"max_tokens":5}' &
done
wait
# All 5 should return 200
```

**Test 10: Graceful shutdown**
```bash
node /tmp/opencode/opencode-free-setup.mjs &
PID=$!
sleep 2
kill $PID
sleep 1
# Verify: process is gone
kill -0 $PID 2>/dev/null && echo "FAIL" || echo "PASS"
```

### 8.2 Integration Test

```bash
# Full flow: start proxy → connect with openclaude → verify output
node /tmp/opencode/opencode-free-setup.mjs &
sleep 2
CLAUDE_CODE_USE_OPENAI=1 \
  OPENAI_BASE_URL=http://127.0.0.1:8080/v1 \
  OPENAI_API_KEY=public \
  OPENAI_MODEL=deepseek-v4-flash-free \
  timeout 30 openclaude "Say hello in French, one word only"
# Verify: outputs "Bonjour."
```

### 8.3 File impact check

```bash
# Verify no ~/.openclaude/ files created or modified
ls -la ~/.openclaude/ 2>&1 || echo "NO_DIR"
# Should either not exist or be unchanged
```

---

## Summary

| Aspect | Before | After |
|--------|--------|-------|
| Lines | 261 | ~220 |
| Dependencies | `node:child_process` | None beyond Node.js built-ins |
| Process model | Spawns openclaude child, parent acts as lifecycle manager | Self-contained HTTP server |
| Path support | `/zen/v1/...` only | `/v1/...` + `/zen/v1/...` |
| Connection reuse | None (new socket per request) | keepAlive Agent, 32 max sockets |
| Timeout | None (hang forever) | 120s AbortController |
| Backpressure | Implicit (.pipe) | Explicit (drain-aware) |
| CORS | On all responses (same) | + OPTIONS preflight |
| Health check | None | GET /health |
| Config impact | Zero | Zero |
| Startup | `node proxy.mjs --bg` (spawns openclaude) | `node proxy.mjs` (stays as proxy) |

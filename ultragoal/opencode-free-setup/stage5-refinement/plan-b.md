# Plan B: Standalone High-Performance HTTP Proxy

**Stage 5 Refinement — Refactor `opencode-free-setup.mjs` from launcher-with-proxy to pure proxy**

---

## 1. Full Diff (Current → Target)

### Deletions

| Lines | What | Why |
|-------|------|-----|
| 3 | `import { spawn } from 'node:child_process'` | No child process management |
| 207–261 | Entire `main()` function | Launcher logic removed |
| 231–255 | Child spawn, env setup, cleanup with `child.kill()` | Not needed |
| 258–261 | `main().catch(...)` | No entry point wrapping needed |

### Modifications

1. **Import additions** (after line 2):

   ```js
   import { Agent } from 'node:https'
   import cluster from 'node:cluster'        // optional, only if multi-core
   ```

2. **Named constants** — extract magic strings:

   ```js
   const PORT = parseInt(process.env.OPENCODE_FREE_PROXY_PORT ?? '0', 10)
   const HOST = '127.0.0.1'
   const REQUEST_TIMEOUT_MS = 30_000
   const UPSTREAM_HOST = OPCODE_ZEN_HOST   // already defined
   const UPSTREAM_PORT = OPCODE_ZEN_PORT   // already defined
   ```

3. **Global upstream Agent** (connection reuse):

   ```js
   const upstreamAgent = new Agent({
     keepAlive: true,
     keepAliveMsecs: 1000,
     maxSockets: 256,
     maxFreeSockets: 64,
     scheduling: 'lifo',
   })
   ```

4. **Route table** — replace the `if/else` chain with a dispatch table:

   ```js
   const ROUTES = [
     { method: 'GET',  pattern: /^\/v1\/models\/?$/,              handler: handleModelsList },
     { method: 'GET',  pattern: /^\/zen\/v1\/models\/?$/,         handler: handleModelsList },
     { method: 'GET',  pattern: /^\/v1\/models\/(.+)$/,           handler: handleModelById },
     { method: 'GET',  pattern: /^\/zen\/v1\/models\/(.+)$/,      handler: handleModelById },
     { method: 'POST', pattern: /^\/v1\/chat\/completions\/?$/,   handler: handleChatCompletions },
     { method: 'POST', pattern: /^\/zen\/v1\/chat\/completions\/?$/, handler: handleChatCompletions },
   ]
   ```

5. **`handleChatCompletions`** — add timeout + `agent`:

   ```js
   function handleChatCompletions(req, res) {
     const controller = new AbortController()
     const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS)

     const opencodeHeaders = buildOpenCodeHeaders()
     if (req.headers['content-type']) {
       opencodeHeaders['content-type'] = req.headers['content-type']
     }

     const proxyReq = httpsRequest({
       hostname: UPSTREAM_HOST,
       port: UPSTREAM_PORT,
       path: '/zen/v1/chat/completions',
       method: 'POST',
       headers: opencodeHeaders,
       rejectUnauthorized: true,
       agent: upstreamAgent,
       signal: controller.signal,
     }, (proxyRes) => {
       clearTimeout(timeout)
       const responseHeaders = { 'access-control-allow-origin': '*' }
       for (const [key, value] of Object.entries(proxyRes.headers)) {
         if (!isHopByHopHeader(key)) responseHeaders[key] = value
       }
       // Ensure SSE headers for streaming
       if (proxyRes.headers['content-type']?.includes('text/event-stream')) {
         responseHeaders['cache-control'] = 'no-cache'
         responseHeaders['x-accel-buffering'] = 'no'
       }
       res.writeHead(proxyRes.statusCode, proxyRes.statusMessage ?? '', responseHeaders)
       proxyRes.pipe(res)
     })

     proxyReq.on('error', (error) => {
       clearTimeout(timeout)
       if (error.name === 'AbortError') {
         console.error('[opencode-free] Upstream request timed out')
       } else {
         console.error(`[opencode-free] Upstream error: ${error.message}`)
       }
       if (!res.headersSent) {
         res.writeHead(502, {
           'Content-Type': 'application/json',
           'Access-Control-Allow-Origin': '*',
         })
         res.end(JSON.stringify(formatErrorBody(
           error.name === 'AbortError' ? 'Upstream request timed out' : `Upstream error: ${error.message}`,
           error.name === 'AbortError' ? 'timeout_error' : 'upstream_error'
         )))
       } else {
         res.destroy()
       }
     })

     req.pipe(proxyReq)
   }
   ```

6. **Server create** — wrap in `start()`:

   ```js
   function start() {
     const server = createServer(async (req, res) => {
       const { method, url } = req
       let matched = false
       for (const route of ROUTES) {
         const match = url.match(route.pattern)
         if (route.method === method && match) {
           matched = true
           try {
             await route.handler(req, res, url, match)
           } catch (error) {
             console.error(`[opencode-free] Route error: ${error.message}`)
             if (!res.headersSent) {
               res.writeHead(500, { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' })
               res.end(JSON.stringify(formatErrorBody('Internal server error', 'server_error')))
             } else {
               res.destroy()
             }
           }
           break
         }
       }
       if (!matched) {
         res.writeHead(404, {
           'Content-Type': 'application/json',
           'Access-Control-Allow-Origin': '*',
         })
         res.end(JSON.stringify(formatErrorBody('Not found', 'not_found')))
       }
     })

     server.listen(PORT, HOST, () => {
       const addr = server.address()
       console.error(`[opencode-free] Proxy running on http://${HOST}:${addr.port}`)
     })

     // Clean signal handling — no child processes
     const shutdown = () => {
       console.error('[opencode-free] Shutting down...')
       upstreamAgent.destroy()
       server.close(() => process.exit(0))
       setTimeout(() => process.exit(0), 2000).unref()
     }
     process.on('SIGINT', shutdown)
     process.on('SIGTERM', shutdown)
     process.on('SIGQUIT', shutdown)

     return server
   }

   // No main() wrapper — direct at top level
   start()
   ```

7. **`handleModelById`** — accept regex match from route dispatch:

   ```js
   async function handleModelById(req, res, url, match) {
     const modelId = match[1]   // capture group from /v1/models/(.+) or /zen/v1/models/(.+)
     // ... rest unchanged
   }
   ```

### Files unchanged

- Everything from line 1–166 except `import { spawn }` removal and `handleChatCompletions` modifications.
- `fetchFreeModels()`, `buildOpenCodeHeaders()`, `formatModelList()`, `formatModelEntry()`, `formatErrorBody()`, `isHopByHopHeader()`, `pickBestModel()` — all unchanged.

---

## 2. Performance Strategy

| Technique | Implementation | Expected gain |
|-----------|---------------|---------------|
| **Connection reuse** | Single `https.Agent` with `keepAlive: true`, `maxSockets: 256`, `scheduling: 'lifo'` | Eliminates TCP+TLS handshake per request (RTT ~50-300ms saved per request) |
| **Request timeout** | `AbortController` + `setTimeout` per upstream request | Prevents connection pile-up from stalled upstream; free sockets remain available |
| **Efficient streaming** | `req.pipe(proxyReq)` / `proxyRes.pipe(res)` — already optimal | Zero buffering, backpressure-aware, minimal memory per stream |
| **Route dispatch via regex** | Linear scan of 6 route entries | O(6) per request — negligible. `RegExp.prototype.test` is fast-path compiled |
| **No per-request allocation of unused objects** | Remove `new` wrappers, inline headers objects, use `Object.assign` patterns where needed | Reduces GC pressure |
| **LIFO socket scheduling** | `scheduling: 'lifo'` in Agent | Warmer sockets used first (reduces idle-timeout churn) |

**Anti-patterns explicitly avoided:**
- Array spread/copy per request
- Per-request `new URL()` parsing (use regex on raw `url` string)
- `JSON.parse` of request body unless needed (currently not needed — streamed directly)

---

## 3. Protocol Compatibility Strategy

### Path mapping

| Incoming path | Map to | Behavior |
|---------------|--------|----------|
| `GET /v1/models` | `GET /zen/v1/models` | Same response (model list) |
| `GET /v1/models/:id` | `GET /zen/v1/models/:id` | Same response (single model) |
| `GET /zen/v1/models` | Direct | Same |
| `POST /v1/chat/completions` | `POST /zen/v1/chat/completions` → upstream `https://opencode.ai/zen/v1/chat/completions` | Proxied with opencode headers |
| `POST /zen/v1/chat/completions` | Direct → upstream | Same |

### SSE streaming

- Upstream sends `content-type: text/event-stream` when `stream: true` in request body.
- Proxy forwards raw SSE bytes via `pipe()` — no parsing/modification.
- Response headers augmented with `cache-control: no-cache` and `x-accel-buffering: no` for SSE to prevent intermediary buffering.

### OpenAI error shape

All error responses use:

```json
{
  "error": {
    "message": "...",
    "type": "upstream_error|timeout_error|not_found|server_error"
  }
}
```

This matches the OpenAI API error schema (`{ error: { message, type } }`).

### Model list format

Returns standard OpenAI `/v1/models` shape:

```json
{
  "object": "list",
  "data": [
    { "id": "deepseek-v4-flash-free", "object": "model", "created": 1710000000, "owned_by": "opencode" }
  ]
}
```

---

## 4. Usage Examples

### Direct curl

```bash
# List models
curl http://127.0.0.1:3456/v1/models

# Chat completion (streaming)
curl http://127.0.0.1:3456/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer public" \
  -d '{
    "model": "deepseek-v4-flash-free",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
  }'

# Same via /zen/v1 namespace
curl http://127.0.0.1:3456/zen/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer public" \
  -d '{"model": "deepseek-v4-flash-free", "messages": [{"role": "user", "content": "Hi"}], "stream": false}'
```

### openclaude config

Profile `~/.config/openclaude/profile.yml`:

```yaml
provider: openai
api_key: "public"
base_url: "http://127.0.0.1:3456/v1"    # <-- only change needed
models:
  - deepseek-v4-flash-free
  - qwen3.6-plus-free
```

The proxy exposes `/v1/models`, `/v1/chat/completions` — openclaude (or any OpenAI client) uses these as standard endpoints. No `/zen/v1` path needed in the profile.

### Custom port

```bash
# Via env var
OPENCODE_FREE_PROXY_PORT=9999 node server.mjs

# Via CLI arg (add arg parsing)
node server.mjs --port 9999
```

---

## 5. Testing Strategy

### Unit tests

| Test | What it verifies | How |
|------|-----------------|-----|
| Route matching | All 6 routes match correct URLs, reject wrong methods | `RegExp.prototype.test` against known URL strings |
| Model list format | Output shape matches OpenAI spec | `JSON.parse` response body, assert shape |
| Single model lookup | Valid ID returns 200, invalid returns 404 | Hit both paths |
| Error body shape | `formatErrorBody` produces `{ error: { message, type } }` | Assert structure |
| Hop-by-hop filtering | `connection`, `transfer-encoding` etc are stripped | Assert filtered set |
| `buildOpenCodeHeaders` | Contains 4 required x-opencode-* headers | Assert keys exist |
| Model priority | `pickBestModel` returns highest priority model from list | Assert known order |

### Integration tests

| Test | What it verifies | How |
|------|-----------------|-----|
| Proxy chat completion (non-streaming) | Full HTTP round-trip, status 200, valid JSON body | `fetch()` → assert status/body |
| Proxy chat completion (streaming) | SSE bytes arrive, no buffering corruption | `fetch()` with reader, assert `text/event-stream` content-type |
| Timeout behavior | Stalled upstream triggers 502 after timeout | Set very short timeout, connect to black-hole, assert 502 |
| Connection reuse | Multiple sequential requests reuse sockets | `netstat` or Agent `totalSocketCount` before/after |
| Server shutdown | SIGINT/SIGTERM → graceful `close()` | Send signal, check `server.close` callback fires |
| Port selection | Env var `OPENCODE_FREE_PROXY_PORT` is respected | Start with env, assert `server.address().port` matches |

### Test tooling

- `node:test` (built-in, available in Node 20+) for unit tests in a separate test file
- Manual `curl` smoke tests (documented above)
- No test framework dependency needed (zero-dep constraint applies to test file too, or use `node --test`)

---

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `AbortSignal` not supported in older Node | Low (Node 15+) | High — proxy silently hangs | Document minimum Node version (18+). Use `'abort'` event listener as fallback. |
| `keepAlive: true` creates zombie sockets | Low | Medium — FD leak | `maxFreeSockets: 64` bounds idle sockets; Agent auto-destroys on `server.close` |
| Upstream `opencode.ai` changes path schema | Medium | High — proxy breaks | Single constant `UPSTREAM_PATH_PREFIX`. Add runtime upstream schema check (optional). |
| SSE stream corruption from header injection | Low | Medium — garbled output | Proxy does not modify SSE body, only passes through. Tested via integration test. |
| `req.pipe(proxyReq)` with `AbortController` | Low | Medium — partial writes | `AbortSignal` immediately stops `https.request`; proxyReq response never fires, caught by error handler. |
| Memory leak from long-lived connections | Low | Medium | Bound `maxSockets` and `maxFreeSockets`; default Node HTTP client timeouts clean up idle sockets. |
| Client disconnects mid-stream | Medium | Low — unhandled rejection | Already handled by `res.destroy()` in error cases. Add `req.on('close') → proxyReq.destroy()` for proactive cleanup. |

### Risk: client disconnect mid-stream (add to `handleChatCompletions`)

```js
  // Proactive cleanup on client disconnect
  req.on('close', () => {
    proxyReq.destroy()
  })
```

This prevents orphaned upstream requests when the client drops mid-SSE-stream.

### Minimum Node version

- Node 18+ (LTS). `AbortSignal` available natively. `node:test` available.
- Node 20+ recommended for `--experimental-test-coverage` and `permission` model.

---

## Summary

| Dimension | Plan A (current) | Plan B (target) |
|-----------|-----------------|-----------------|
| Dependencies | Node builtins | Node builtins |
| External binaries | openclaude (spawn) | None |
| Paths supported | `/zen/v1/*` only | `/v1/*` + `/zen/v1/*` |
| Connection reuse | None (new socket per request) | `keepAlive: true` Agent |
| Request timeout | None | `AbortController` 30s |
| Error types | 502, 404, 500 | + `timeout_error` type |
| Server lifecycle | Managed by `main()` + child | Direct `start()` + signal handlers |
| Lines of code | 261 | ~190 (net -70) |

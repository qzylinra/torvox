# Plan A: Reverse Proxy Script for openclaude → opencode.ai Free Models

## Architecture

Single Node.js file (`opencode-free-setup.mjs`) with no dependencies beyond built-in modules. Creates a local HTTP proxy that sits between openclaude and `opencode.ai/zen/v1/`:

```
openclaude → localhost:{RANDOM_PORT} (proxy) → opencode.ai/zen/v1/
               └── intercepts /v1/models (returns free models only)
```

The proxy:
1. Starts HTTP server on `127.0.0.1:0` (OS-assigned random port)
2. Reads the assigned port from `server.address()`
3. Fetches free model list from `models.dev/api.json`
4. Spawns openclaude with `CLAUDE_CODE_USE_OPENAI=1` pointing to local proxy
5. Proxies chat completion requests, injecting pi-opencode-zen headers
6. Intercepts `/v1/models` to return only free models
7. Cleans up both child process and server on exit

---

## Module Imports (Zero Dependencies)

```js
import { createServer } from 'node:http'
import { request as httpsRequest } from 'node:https'
import { spawn } from 'node:child_process'
import crypto from 'node:crypto'
import process from 'node:process'
```

No npm packages. No external dependencies. Global `fetch` (available since Node.js 18) for the models.dev fetch.

---

## Header Generation and Management

Pi-opencode-zen sends these headers on every request. Our proxy generates them identically:

| Header | Generation | Stability |
|--------|-----------|-----------|
| `User-Agent` | Literal `"opencode/latest/1.3.15/cli"` | Constant |
| `x-opencode-client` | Literal `"cli"` | Constant |
| `x-opencode-session` | `crypto.randomUUID().replace(/-/g, '').slice(0, 26)` | Once per proxy lifetime |
| `x-opencode-project` | Same pattern as session | Once per proxy lifetime |
| `x-opencode-request` | Same pattern | Fresh per proxied request |

**Session vs request lifetime:**
- `x-opencode-session` and `x-opencode-project` are generated once at proxy startup and reused across all forwarded requests. This matches pi-opencode-zen's per-extension-instance behavior.
- `x-opencode-request` is generated fresh for each proxied `POST /v1/chat/completions` call.

**Auth policy for forwarded requests:**
- The proxy does NOT send `Authorization` header when forwarding to `opencode.ai/zen/v1/chat/completions`. This matches pi-opencode-zen's public mode behavior, where no auth is sent to streaming endpoints. Free models are served without authentication.
- Any `authorization` header from the incoming openclaude request is stripped before forwarding.

---

## HTTP Server Routes

### `POST /v1/chat/completions` — Proxy to opencode.ai

Flow:
1. Buffer incoming request body (JSON payload from openclaude)
2. Generate fresh `x-opencode-request` header
3. Build header set: all pi-opencode-zen headers + `Content-Type: application/json` from incoming request
4. Make `https.request` to `opencode.ai/zen/v1/chat/completions`:
   - Method: POST
   - Headers: the built header set (NO Authorization)
   - Body: the buffered request body
5. On upstream response:
   - Strip `content-length` from upstream headers (streaming changes length)
   - Forward all other upstream headers
   - Set response status code matching upstream
   - Pipe upstream response stream to client (`upstreamRes.pipe(res)`)
6. On upstream error: write 502 response with error message

**Streaming (SSE) handling:**
- `pipe()` handles SSE natively — the upstream response is `text/event-stream` with `Transfer-Encoding: chunked`. Each SSE chunk from opencode.ai flows through immediately to the client.

### `GET /v1/models` — Intercepted (model list)

Flow:
1. Return cached free model list (fetched from models.dev at startup)
2. Format: OpenAI-compatible response
   ```json
   {
     "object": "list",
     "data": [
       {
         "id": "deepseek-v4-flash-free",
         "object": "model",
         "created": 1710000000,
         "owned_by": "opencode"
       }
     ]
   }
   ```
3. Set `Content-Type: application/json`
4. Status: 200

### `GET /v1/models/:id` — Intercepted (single model)

Flow:
1. Look up model ID in the cached free model list
2. If found: return single model object (same format as array entry)
3. If not found: return 404 with OpenAI-compatible error format:
   ```json
   { "error": { "message": "Model 'xxx' not found", "type": "not_found" } }
   ```

### `/*` — Catch-all

Return 404 with OpenAI-compatible error format.

---

## Model Discovery and Caching

### Startup fetch

Immediately upon proxy start, before spawning openclaude:

```js
async function fetchFreeModels() {
  const res = await fetch('https://models.dev/api.json')
  if (!res.ok) return null
  const data = await res.json()
  const free = []
  for (const [id, info] of Object.entries(data?.opencode?.models ?? {})) {
    if (info?.status === 'deprecated') continue
    if (info?.cost?.input === 0 && info?.cost?.output === 0) {
      free.push(id)
    }
  }
  return free
}
```

### Fallback list

If `fetchFreeModels()` returns null or empty array, use a hardcoded fallback:

```js
const FALLBACK_FREE_MODELS = [
  'deepseek-v4-flash-free', 'qwen3.6-plus-free', 'glm-5',
  'nemotron-3-super-free', 'big-pickle', 'minimax-m2.5-free',
  'kimi-k2.5', 'kimi-k2', 'kimi-k2-thinking', 'glm-4.7',
  'glm-4.6', 'minimax-m2.1', 'trinity-large-preview-free',
]
```

### Model selection (best for default)

Priority list for the default model passed to openclaude:

```js
const MODEL_PRIORITY = [
  'deepseek-v4-flash-free', 'qwen3.6-plus-free',
  'nemotron-3-super-free', 'big-pickle',
  'minimax-m2.5-free', 'kimi-k2.5', 'glm-5'
]
```

Pick first match, or fall back to `models[0]`.

### Cache strategy

Free model list is fetched once at startup and cached for the lifetime of the proxy. No re-fetching. If the fetch fails entirely (network error, DNS failure), use fallback list with a warning on stderr.

---

## Spawn Logic and Env Vars

After the proxy is listening and models are resolved:

### Env vars passed to openclaude child

```js
const env = {
  ...process.env,
  CLAUDE_CODE_USE_OPENAI: '1',
  OPENAI_BASE_URL: `http://127.0.0.1:${port}`,
  OPENAI_API_KEY: 'public',
  OPENAI_MODEL: bestModelId,
}
```

**Why this works:**
- `CLAUDE_CODE_USE_OPENAI=1` triggers `hasExplicitProviderSelection()` → returns `true` → profile file is NOT read → `~/.openclaude/.openclaude-profile.json` is untouched.
- `OPENAI_BASE_URL` points to our local proxy.
- `OPENAI_API_KEY` is set to `"public"` so openclaude doesn't error about missing credentials. The proxy strips this before forwarding to opencode.ai.
- `OPENAI_MODEL` sets the default model.
- `CLAUDE_CODE_PROVIDER_PROFILE_ENV_APPLIED` is intentionally NOT set — we want openclaude to process the env var selection fresh.

### Spawn call

```js
const child = spawn('/usr/local/bin/openclaude', process.argv.slice(2), {
  env,
  stdio: 'inherit',
})
```

Forward all remaining CLI args from the wrapper script. Use `stdio: 'inherit'` for direct terminal interaction.

### Background mode (`--bg`)

If `process.argv.includes('--bg')`:
- Call `child.unref()`
- Do NOT exit the proxy process (proxy stays alive serving requests)
- The proxy's own process continues running in background

---

## Signal Handling and Cleanup

### Current process signals

| Signal | Action |
|--------|--------|
| `SIGINT` | Kill child process, close server, exit |
| `SIGTERM` | Same as SIGINT |
| `SIGHUP` | Same as SIGINT |

### Child process exit

```js
child.on('exit', (code, signal) => {
  server.close(() => process.exit(code ?? 1))
  // Force exit after 2s if server doesn't close cleanly
  setTimeout(() => process.exit(code ?? 1), 2000).unref()
})
```

### Proxy error during operation

If the upstream `opencode.ai` returns a non-2xx status, forward the error response as-is to the client. The proxy continues running.

If the upstream connection fails (network down, DNS failure), return `502 Bad Gateway` with a JSON error body.

---

## Error Handling Map

| Scenario | Behavior |
|----------|----------|
| models.dev fetch fails at startup | Log warning to stderr, use fallback model list, continue |
| models.dev returns empty model list | Use fallback list, log warning |
| Proxy port binding fails | Exit with error code 1 |
| Upstream opencode.ai returns error | Forward exact status + headers + body to client |
| Upstream connection timeout | Return 502 after reasonable timeout (no explicit timeout — let the TCP stack handle it) |
| Malformed request from openclaude | Return 400 with JSON error body |
| Child process crashes | Log exit code, clean up, exit with same code |
| SIGINT/SIGTERM | Kill child, close server, graceful exit within 2s timeout |

---

## Code Structure Outline

```
opencode-free-setup.mjs
├── Imports (node:http, node:https, node:child_process, node:crypto, node:process)
├── Constants (MODELS_DEV_URL, OPENCODE_ZEN_URL, FALLBACK_FREE_MODELS, MODEL_PRIORITY, HEADERS)
├── Utility functions
│   ├── generateId()         → 26-char hex string
│   ├── buildHeaders()       → pi-opencode-zen header set (session + project stable, request fresh)
│   ├── fetchFreeModels()    → fetch and filter models.dev/api.json
│   ├── pickBestModel(ids)   → priority-based model selection
│   ├── buildModelList()     → format free models as OpenAI-compatible JSON
│   └── buildErrorBody(msg)  → OpenAI-compatible error JSON
├── main()
│   ├── Parse CLI args (check for --bg)
│   ├── Fetch free models (with fallback)
│   ├── Create HTTP server
│   │   ├── Route: POST /v1/chat/completions  → proxy logic
│   │   ├── Route: GET /v1/models             → return cached list
│   │   ├── Route: GET /v1/models/:id         → return single or 404
│   │   └── Route: *                          → 404
│   ├── Start listening (port 0)
│   ├── Get assigned port
│   ├── Prepare env vars
│   ├── Spawn openclaude
│   ├── If --bg: unref child, do NOT exit
│   ├── Register signal handlers (SIGINT, SIGTERM)
│   └── Child exit handler (cleanup + exit)
└── main().catch(err => { console.error(err); process.exit(1) })
```

---

## Testing Approach

### Unit tests (manual, using curl)

**Test 1: Model list filtering**
```bash
# Start proxy in background
node opencode-free-setup.mjs --bg &
PROXY_PID=$!
sleep 2

# Find proxy port from listening connections
PORT=$(ss -tlnp | grep "node.*opencl" | awk '{print $4}' | grep -oP ':\K\d+')

# Test /v1/models returns only free models
curl -s http://127.0.0.1:$PORT/v1/models | jq '.data[].id'

# Verify no model has an id with a paid model name pattern
# (e.g., "claude-opus" should not appear)
curl -s http://127.0.0.1:$PORT/v1/models | jq '[.data[].id] | length'

kill $PROXY_PID
```

**Test 2: Chat completion streaming**
```bash
PORT=$(ss -tlnp | grep "node.*opencl" | awk '{print $4}' | grep -oP ':\K\d+')
curl -s -N http://127.0.0.1:$PORT/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Say hello"}],"stream":true}'
# Should see SSE data: [DONE] chunks
```

**Test 3: Error propagation**
```bash
PORT=$(ss -tlnp | grep "node.*opencl" | awk '{print $4}' | grep -oP ':\K\d+')
# Send an invalid model
curl -s http://127.0.0.1:$PORT/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"nonexistent-model","messages":[{"role":"user","content":"Hi"}],"stream":true}'
# Should see 4xx error from upstream forwarded correctly
```

**Test 4: Unknown endpoint returns 404**
```bash
PORT=$(ss -tlnp | grep "node.*opencl" | awk '{print $4}' | grep -oP ':\K\d+')
curl -s -w "\n%{http_code}" http://127.0.0.1:$PORT/v1/embeddings
# Should see 404
```

**Test 5: Profile file untouched**
```bash
# Before running
HASH_BEFORE=$(md5sum ~/.openclaude/.openclaude-profile.json 2>/dev/null || echo "NO_FILE")

node opencode-free-setup.mjs --bg &
sleep 5
kill $!

# After running
HASH_AFTER=$(md5sum ~/.openclaude/.openclaude-profile.json 2>/dev/null || echo "NO_FILE")
echo "Before: $HASH_BEFORE   After: $HASH_AFTER"
# Should be identical (both "NO_FILE" or same hash)
```

### Integration test (interactive)

```bash
node opencode-free-setup.mjs
# openclaude starts, connected through local proxy
# Type /model — should show the free model
# Type a question — should stream response
```

---

## Implementation Notes

1. **Port detection**: After `server.listen(0)`, call `server.address().port` synchronously before spawning. Port is stable for the proxy's lifetime.

2. **Request body buffering**: `POST /v1/chat/completions` must buffer the full incoming body (using `data` + `end` events on the incoming message) before forwarding. Request bodies are small JSON payloads, so this is not a memory concern.

3. **Upstream request with `https.request`**: The forwarded request uses `node:https.request` which returns a `ClientRequest` (writable stream). Write the buffered body and call `.end()`.

4. **Response streaming with `.pipe()`**: `upstreamRes.pipe(res)` handles backpressure correctly. No manual chunk management needed.

5. **Header forwarding**: Forward `content-type`, `transfer-encoding`, and `cache-control` from upstream. Strip `content-length` (streaming changes it). Strip `connection` (Node.js manages it).

6. **Error boundary per request**: Each proxied request has its own error handler. A failure in one request does not affect the proxy's ability to handle subsequent requests.

7. **`--bg` mode**: Both proxy and child run as background processes. The proxy process does NOT exit after spawning. To kill: `pkill -f "opencode-free-setup"` or `kill <pid>`.

8. **No temp files**: Unlike the previous approach (which used temp dirs and profile files), this approach uses zero disk I/O beyond the script itself. All state is in memory.

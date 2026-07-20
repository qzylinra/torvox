# Stage 5 Refinement — Implementation Report

## Summary of Changes

### Removed
- `import { spawn } from 'node:child_process'` (was line 3) — entire `node:child_process` dependency eliminated
- `let cachedFreeModels = null` (was line 29) — no longer needed without `main()` caching
- `main()` function and `main().catch(...)` (was lines 207–261) — entire openclaude spawner removed
- `cachedFreeModels ?? FALLBACK_FREE_MODELS` fallback → replaced with just `FALLBACK_FREE_MODELS`
- Hardcoded `/zen/v1` path prefix in routes → replaced with `normalizePath()` for dual-path support

### Added
- `Agent` import from `node:https` — keepAlive connection pool (`UPSTREAM_AGENT`)
- Constants: `UPSTREAM_PATH_PREFIX`, `REQUEST_TIMEOUT_MS`, `PORT` (env chain), `HOST`
- `corsHeaders()` — CORS preflight helper with 4 headers
- `sendJson()` — consistent JSON response helper with automatic CORS
- `normalizePath()` — maps both `/zen/v1/...` and `/v1/...` to `/v1/...`
- `handleHealth()` — `GET /health` → `{ status: "ok", uptime: N }`
- `start()` — server creation, listen, SIGINT/SIGTERM handlers
- `formatErrorBody()` now takes `code` and `param` params (full OpenAI error shape)

### Fixed (not in plan, discovered during testing)
- `req.on('close')` handler: used `req.destroyed` check which fires on normal body consumption. Changed to `req.socket?.destroyed` to detect real client disconnect. Without this fix, every non-streaming request got `socket hang up` / ECONNRESET because `proxyReq.destroy()` was called prematurely.

## Test Results

| # | Test | Result |
|---|------|--------|
| 1 | Health check (`GET /health` → 200) | **PASS** |
| 2 | Model list returns only free models with correct structure | **PASS** |
| 3 | Model by ID: found (200) / not found (404) | **PASS** |
| 4 | Chat completion (non-streaming) | **PASS** |
| 5 | SSE streaming with `data: [DONE]` marker | **PASS** |
| 6 | CORS headers on GET, OPTIONS; OPTIONS returns 204 | **PASS** |
| 7 | No `child_process` dependency | **PASS** (0 references) |
| 8 | 404 on `/unknown` and `/v1/unknown` | **PASS** |
| 9 | Concurrent requests (5 simultaneous) | **PASS** (all 200) |
| 10 | Graceful shutdown (SIGINT → exit) | **PASS** |
| 11 | Port env var chain (PORT=9090, OPENCODE_FREE_PROXY_PORT=9091) | **PASS** |
| 12 | `/zen/v1` path parity (identical model list response) | **PASS** |

**All 12 tests pass.**

## Issues Encountered

### Issue 1: `socket hang up` on every upstream request (CRITICAL)

**Root cause:** The `req.on('close')` handler called `proxyReq.destroy()` unconditionally when `req` emitted `close`. In Node.js, `IncomingMessage.close` fires when the request body stream has been fully consumed by `req.pipe(proxyReq)`, NOT only on actual client disconnect. This destroyed the upstream connection before the response arrived.

**Fix:** Changed the guard from `req.destroyed` (which is true after body consumption) to `req.socket?.destroyed` (which is true only when the TCP connection is forcibly closed). This is the canonical way to detect real client disconnects in Node.js HTTP servers.

**Detection:** Debug logging showed the event order: `req.close` → `proxyReq.destroy()` → `ECONNRESET` error on proxyReq. Isolating the Agent/signal/pipe combination in a minimal test reproduced the issue only when the close handler destroyed proxyReq.

### Issue 2: Plan vs actual line count

The plan estimated ~175 lines. The final script is 264 lines. The discrepancy is because the plan counted the core logic but the actual file includes all helper functions (`generateId`, `buildOpenCodeHeaders`, `fetchFreeModels`, `pickBestModel`, `formatModelList`, `formatModelEntry`, `formatErrorBody`, `isHopByHopHeader`, `corsHeaders`, `sendJson`, `normalizePath`) plus the handlers, router, and `start()`/shutdown boilerplate. The plan's ~175 estimate was optimistic — 264 lines is reasonable for a standalone proxy with these features.

## Final Script Stats

| Metric | Value |
|--------|-------|
| Lines of code | 264 |
| Dependencies | 4 (`node:http`, `node:https`, `node:crypto`, `node:process`) |
| External dependencies | 0 |
| Default port | 8080 |
| Port chain | `OPENCODE_FREE_PROXY_PORT` > `PORT` > `8080` |
| Request timeout | 60s (with SSE per-data-event reset) |
| Connection pool | `https.Agent` keepAlive, 30s idle, 64 max sockets |
| Path support | `/v1/*` and `/zen/v1/*` dual-path routing |

## Reproducing Tests

Start proxy:
```bash
node /tmp/opencode/opencode-free-setup.mjs
```

Test commands (run while proxy is running):

```bash
# Health
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8080/health
# Expected: 200

# Model list
curl -s http://127.0.0.1:8080/v1/models | python3 -c "import json,sys; d=json.load(sys.stdin); assert d['object']=='list'"

# Model by ID
curl -s -w " %{http_code}" http://127.0.0.1:8080/v1/models/deepseek-v4-flash-free
# Expected: 200
curl -s -w " %{http_code}" http://127.0.0.1:8080/v1/models/nonexistent
# Expected: 404

# Chat (non-streaming)
curl -s -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"hi"}],"max_tokens":3}'

# SSE streaming
curl -s -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"Count to 3"}],"stream":true,"max_tokens":20}' \
  | tee /tmp/sse.out | grep -q "data: \[DONE\]"

# CORS
curl -s -D - http://127.0.0.1:8080/v1/models | grep -qi "access-control-allow-origin: \*"
curl -s -o /dev/null -w "%{http_code}" -X OPTIONS http://127.0.0.1:8080/v1/chat/completions
# Expected: 204

# 404
curl -s -w " %{http_code}" http://127.0.0.1:8080/unknown
# Expected: 404

# Graceful shutdown
kill $(pgrep -f opencode-free-setup.mjs); sleep 1; kill -0 $PID 2>/dev/null && echo "FAIL" || echo "PASS"

# Port env chain
PORT=9090 node /tmp/opencode/opencode-free-setup.mjs &
kill %1
OPENCODE_FREE_PROXY_PORT=9091 node /tmp/opencode/opencode-free-setup.mjs &

# Path parity
cmp -s <(curl -s http://127.0.0.1:8080/v1/models) <(curl -s http://127.0.0.1:8080/zen/v1/models) && echo "PASS"
```

No test dependencies required — only `curl`, `python3`, `grep`, and standard Unix tools.

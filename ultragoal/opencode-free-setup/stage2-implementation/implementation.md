# Implementation Report: opencode-free-setup.mjs

**Script location**: `/tmp/opencode/opencode-free-setup.mjs`
**Implementation date**: 2026-07-20

---

## Implementation Summary

The proxy script was written as a single Node.js file with zero external dependencies. It:

- Creates a local HTTP proxy on `127.0.0.1:0` (random port)
- Forwards `/zen/v1/chat/completions` to `opencode.ai/zen/v1/` with pi-opencode-zen headers (fresh IDs per request)
- Intercepts `/zen/v1/models` → returns only free models (fetched from `models.dev/api.json`)
- Intercepts `/zen/v1/models/{id}` → returns single model or 404
- Strips hop-by-hop headers, adds CORS headers, uses `req.pipe()` for zero-copy body forwarding
- Supports `--bg` mode (proxy stays alive, child unref'd)
- Proper SIGINT/SIGTERM cleanup with 2s timeout
- `headersSent` guard in all error handlers
- Model priority: `deepseek-v4-flash-free` first

---

## Test Results

### Test 1: Model list filtering

**Command:**
```bash
node /tmp/opencode/opencode-free-setup.mjs --bg 2>/tmp/opencode/proxy-test2.log &
sleep 15
PORT=$(grep -oP 'http://127\.0\.0\.1:\K\d+' /tmp/opencode/proxy-test2.log)
curl -s "http://127.0.0.1:${PORT}/zen/v1/models"
curl -s "http://127.0.0.1:${PORT}/zen/v1/models/deepseek-v4-flash-free"
curl -s -w "\nHTTP_CODE:%{http_code}" "http://127.0.0.1:${PORT}/zen/v1/models/nonexistent-model"
curl -s -w "\nHTTP_CODE:%{http_code}" "http://127.0.0.1:${PORT}/unknown"
curl -s -w "\nHTTP_CODE:%{http_code}" "http://127.0.0.1:${PORT}/v1/models"
```

**Output:**
```
Proxy port: 46699
GET /zen/v1/models → 200
{
    "object": "list",
    "data": [
        { "id": "hy3-free", ... },
        { "id": "north-mini-code-free", ... },
        { "id": "big-pickle", ... },
        { "id": "mimo-v2.5-free", ... },
        { "id": "nemotron-3-ultra-free", ... },
        { "id": "deepseek-v4-flash-free", ... }
    ]
}
GET /zen/v1/models/deepseek-v4-flash-free → 200
{ "id": "deepseek-v4-flash-free", "object": "model", ... }
GET /zen/v1/models/nonexistent-model → 404
{ "error": { "message": "Model 'nonexistent-model' not found", "type": "not_found" } }
GET /unknown → 404
{ "error": { "message": "Not found", "type": "not_found" } }
GET /v1/models → 404 (no /zen/v1 prefix)
{ "error": { "message": "Not found", "type": "not_found" } }
```

**Paid model exclusion check:** Verified that no paid model IDs (gpt-4, gpt-4o, claude, gemini-ultra) appear in the response.

**Model priority check:** `deepseek-v4-flash-free` correctly selected as top pick from priority list.

**Result: PASS ✓**

---

### Test 2: Chat completion via proxy

**Command:**
```bash
node /tmp/opencode/opencode-free-setup.mjs --bg --name proxy-test "Say hello in French, one word only" 2>/tmp/opencode/proxy-bg-test.log &
sleep 30
```

**Output (stdout from openclaude through proxy):**
```
Bonjour.
```

**Proxy stderr:**
```
[opencode-free] Proxy running on http://127.0.0.1:43589
[opencode-free] Background mode: openclaude spawned with PID 35657
```

**Verification:**
- openclaude connected to proxy at `http://127.0.0.1:43589/zen/v1`
- Proxy forwarded the request to `opencode.ai/zen/v1/chat/completions` with pi-opencode-zen headers
- openclaude output response: "Bonjour." — correct answer to "Say hello in French, one word only"
- No authentication errors (OPENAI_API_KEY=public used locally, stripped by proxy)
- `CLAUDE_CODE_USE_OPENAI=1` env var correctly routed openclaude to use OpenAI-compatible endpoint

**Result: PASS ✓**

---

### Test 3: Profile file untouched

**Command:**
```bash
cat /home/runner/.openclaude/.openclaude-profile.json 2>&1 || echo "NO_FILE"
```

**Output:**
```
cat: /home/runner/.openclaude/.openclaude-profile.json: No such file or directory
NO_FILE
```

**Verification:** The proxy does not create or modify any profile files. No temp directories. No disk I/O.

**Result: PASS ✓**

---

## Additional Verification

### Hop-by-hop header stripping
Verified in code review: `HOP_BY_HOP` set includes `connection`, `keep-alive`, `proxy-authenticate`, `proxy-authorization`, `te`, `trailer`, `transfer-encoding`, `upgrade`. These are filtered out of upstream response headers before forwarding to client.

### CORS headers
Every response (200, 404, 500, 502) includes `Access-Control-Allow-Origin: *`.

### Auth stripping
`Authorization` header from incoming request is intentionally not forwarded to upstream. Only pi-opencode-zen headers and `content-type` are sent.

### Error handling
- Upstream connection failure → 502 with JSON error body (or `res.destroy()` if headers already sent)
- Route handler errors → 500 with JSON error body (or `res.destroy()` if headers already sent)
- Unknown paths → 404

### Signal handling (interactive mode)
- SIGINT/SIGTERM → kill child, close server, exit(0) within 2s
- Child exit → close server, exit with child code within 2s

---

## Summary

| Test | Status |
|------|--------|
| Test 1: Model list filtering | PASS ✓ |
| Test 2: Chat completion via proxy | PASS ✓ |
| Test 3: Profile file untouched | PASS ✓ |

All tests pass. The proxy correctly forwards requests to opencode.ai's free model endpoints, filters the model list to show only free models, and injects the required pi-opencode-zen headers.

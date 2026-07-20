# Cross-Review: Plan A (author) reviewing Plan B

## 1. What Plan B Does Better — Ideas to Adopt

**Raw pipe for request body** (Plan B line 226: `req.pipe(proxyReq)`): Plan A buffers the POST body before forwarding. Plan B avoids buffering entirely, reducing per-request latency and memory. **Adopt this.**

**Hop-by-hop header stripping** (Plan B lines 200-207): Proper proxy hygiene — strips `transfer-encoding`, `connection`, `keep-alive`, `proxy-authenticate`, `proxy-authorization`, `te`, `trailer`, `upgrade`. Plan A only strips `content-length` and `connection`. **Adopt the full list.**

**Fresh ID per request** (Plan B lines 89-96): Generates new session/project/request IDs for every request. Plan A generates session/project IDs once per proxy lifetime. Plan B's approach is more faithful to pi-opencode-zen behavior. **Adopt — generate all three IDs per request.**

**Error handler with headers-sent guard** (Plan B lines 215-223): If upstream dies mid-stream, tries `res.destroy()` instead of a 502 that would corrupt already-streamed data. Plan A doesn't handle this case. **Adopt.**

**Anti-detection analysis** (Plan B lines 372-383): Good table ranking each fingerprinting factor. Confirms our shared focus on headers is correct. Not code, but worth referencing in Plan A's docs.

---

## 2. Problems in Plan B That Plan A Avoided

### Authorization header leak (Plan B lines 153-155)
Plan B passes `authorization` through to opencode.ai. The value is `"public"`, but sending it at all is unnecessary — opencode.ai's free models don't require auth. Plan A correctly strips it. **Plan A is right; Plan B creates a fingerprinting risk (extra header).**

### Static port 3180 (Plan B line 83)
Port conflicts are inevitable. Plan B's mitigation (lines 420-433) uses a sentinel file + polling loop — fragile and complex. Plan A uses `server.listen(0)` + synchronous `server.address().port`, zero contention. **Plan A is strictly better.**

### Launcher bug: `require` in ESM (Plan B line 337)
```js
const http = require('http')  // TypeError in ESM module
```
Top-level `require` is invalid in `.mjs` files without a bundler. Use `import` or `createRequire`. This makes the launcher non-functional as written.

### Missing `GET /v1/models/:id` endpoint
Plan B only intercepts `/v1/models`. Some OpenAI clients (including openclaude in certain configs) query `/v1/models/{id}` to check availability. Plan A handles this (lines 102-111). **Plan A is more compatible.**

### No `--bg` mode
Plan B doesn't address background operation. If the user runs `openclaude --bg`, the launcher exits and the proxy dies. Plan A explicitly handles this with `child.unref()`.

### Health check endpoint mismatch (Plan B line 338)
Launcher polls `/zen/v1/health`, but the proxy doesn't route that path — it would get a 404. The check would still "succeed" (resolves on any response), but this is accidental, not intentional.

### Child exit signal handling (Plan B line 364)
```js
process.exit(code ?? 0)
```
Ignores the `signal` parameter from the `exit` event. When a child is killed by SIGTERM, `code` is null and `signal` is `'SIGTERM'`. Should exit with signal-based code (128 + signal number) or at minimum use `signal ?? code`.

---

## 3. Single File vs Two Files

**Single file is better for this use case.** The proxy is ~150 lines and the launcher is ~50 lines — not enough to justify separation. The two-file approach introduces:

1. **IPC complexity**: Launcher must poll for proxy readiness + read sentinel file for port
2. **Startup race**: Launcher sleeps/polls; Plan A's `server.address().port` is instant
3. **Double lifecycle**: Must manage two processes; Plan A has one process with a child
4. **Installation friction**: Two files to copy, two paths to maintain

Plan A's single-file approach keeps the server create/listen/spawn/cleanup in one place with zero IPC.

---

## 4. Buffering POST Body for Model Validation

Plan B considers buffering to validate model IDs (lines 294-311) then correctly rejects it:

> "Better approach: trust the `/v1/models` filtering + the user's `OPENAI_MODEL` env var."

Agreed fully. Buffering the body just to validate `parsed.model` adds latency for marginal security benefit. Users who hardcode a paid model via env var are making an intentional choice. The `/v1/models` intercept already provides the right default behavior.

Plan A buffers but doesn't validate either — it just forwards. **Both plans converge on the right answer, but Plan B gets there with zero buffering.**

---

## 5. Static Port (3180) vs Dynamic Port (0)

| Criterion | Dynamic (Plan A) | Static (Plan B) |
|---|---|---|
| Port conflicts | Impossible | Inevitable over time |
| Port discovery | `server.address().port` — instant | Sentinel file + polling — ~100ms to seconds |
| Multiple instances | Yes, any number | No, second fails |
| Reproducibility | Need to print port | Always known (with caveats) |

The reproducibility argument for static ports doesn't apply here — the proxy is a sidecar, not a public service. The user never connects to it directly (openclaude is told the URL via env var). **Dynamic port is unambiguously better.**

Plan B's own dynamic port solution (lines 420-433) is the worst of both worlds: requires a temp file *and* adds polling. Plan A's `server.address().port` is synchronous after `listen()` callback.

---

## 6. Overall Recommendation

**Plan A's architecture, with Plan B's specific improvements.**

Plan A wins on structural decisions:
- Single file → simpler lifecycle, zero IPC
- Dynamic port (0) → no conflicts
- Strips Authorization header → more faithful to pi-opencode-zen
- `GET /v1/models/:id` → more OpenAI-compatible
- `--bg` mode → usable as daemon

Plan B wins on per-request details:
- Raw pipe for request body (no buffering)
- Hop-by-hop header stripping (full list)
- Fresh IDs per request (more faithful to pi-opencode-zen)
- Error handler with headers-sent guard

**Merged approach**: use Plan A's single-file, dynamic-port structure, but adopt Plan B's raw pipe, header stripping, fresh ID generation, and guarded error handler. Discard Plan B's sentinel file, 2-file split, static port, and `--require` hook exploration.

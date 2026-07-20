# Cross-Review: Plan B reviewing Plan A

**Reviewer:** Plan B
**Subject:** Plan A

---

## 1. What Plan A Does Differently

| Dimension | Plan A | Plan B |
|-----------|--------|--------|
| **Route dispatch** | `normalizePath()` strips `/zen` prefix, single code path per route | Regex route table, separate entries for `/v1/...` and `/zen/v1/...` |
| **Backpressure** | Manual `pipeWithBackpressure()` with explicit `drain` handling | `proxyRes.pipe(res)` — Node's built-in pipe, which already handles backpressure |
| **Health check** | `GET /health` → `{ status, uptime, models_fetched }` | None |
| **CORS preflight** | `OPTIONS *` → `204` with explicit headers | Relies on response CORS headers only |
| **Model fetch** | Lazy — fetches on first `/models` request | Eager — fetches at module scope (unchanged) |
| **Timeout** | 120s | 30s |
| **Pool sizing** | `maxSockets: 32`, no LIFO | `maxSockets: 256`, `scheduling: 'lifo'`, `maxFreeSockets: 64` |
| **Default port** | `PORT` env var → 8080 | `OPENCODE_FREE_PROXY_PORT` env var → 0 (OS-assigned random) |
| **Startup pattern** | Module-level `server.listen()`, no wrapper | `start()` function wrapping `createServer` + `listen` |
| **Error shape** | `{ error: { message, type, code, param } }` (full OpenAI spec) | `{ error: { message, type } }` |
| **SSE handling** | Explicit content-type detection with "raw data events forwarded as-is" | Header augmentation (`cache-control`, `x-accel-buffering`) then `.pipe()` |
| **Client disconnect** | Not addressed | `req.on('close') → proxyReq.destroy()` for proactive cleanup |
| **Shutdown** | `server.close() → exit(0)` | `upstreamAgent.destroy()` + `server.close()` + forced `exit(0)` after 2s |
| **Config interaction** | Explicitly zero-touch — "never reads `~/.openclaude/`" | Suggests writing `profile.yml` in `~/.config/openclaude/` |
| **Signal handling** | SIGINT, SIGTERM | SIGINT, SIGTERM, SIGQUIT |

---

## 2. Ideas to Incorporate into Merged Plan

### 2.1 Health check endpoint

Low cost (~5 lines), high value for Docker health checks, Kubernetes liveness probes, and testing. Plan B should adopt this. Uptime and `models_fetched` are nice-to-haves; just `{ status: "ok" }` is sufficient.

### 2.2 CORS preflight (OPTIONS handling)

Plan A correctly handles `OPTIONS *` → `204`. Modern browsers send preflight for non-simple requests (POST with JSON). Without it, browser-based clients (e.g., continue.dev web view, aider web) get CORS errors on preflight. Plan B must adopt this.

### 2.3 Extended error format

Plan A's `{ error: { message, type, code, param } }` is closer to the OpenAI spec. Clients like continue.dev and aider may parse `code` for retry logic (e.g., `code: "rate_limit_exceeded"` → exponential backoff). Plan B should adopt the full shape — it's a trivial change.

### 2.4 Lazy model fetch (partial)

The principle is sound — don't do work at import time that can be deferred. However, Plan A overstates the benefit (20ms cold start is unrealistic for Node). The practical value: if the proxy only handles chat completions and never hits `/models`, we skip one upstream request. Merge this, but acknowledge ~20ms vs ~300ms is ~280ms saved once, not per-request.

### 2.5 Timeout as configurable

Plan A uses `PORT`/`HOST` env vars. Both plans should offer timeout via env var too. Not in Plan B, worth adding.

### 2.6 SSE `cache-control` headers

Plan B got this from Plan A's analysis of proper SSE handling. Already in Plan B. The `x-accel-buffering: no` is a good addition for nginx reverse-proxy scenarios.

---

## 3. Ideas in Plan A That Are Wrong or Over-Engineered

### 3.1 Manual backpressure pipe (over-engineered, wrong)

Plan A's "backpressure-aware pipe" replaces Node's built-in `.pipe()` with a manual implementation using `drain` events. **This is fixing something that isn't broken.** Node's `Readable.prototype.pipe()` already:
- Pauses the readable when `writable.write()` returns `false`
- Resumes on `drain`
- Manages `highWaterMark` automatically
- Handles `end` events

Adding a manual implementation introduces bug surface (did it handle `end` correctly? What about `error` propagation? `unpipe`?) for zero performance benefit. Plan B is correct to keep `.pipe()`.

**Verdict: DO NOT merge. Keep Plan B's `.pipe()`.**

### 3.2 120s timeout (wrong)

120s is too long for a proxy timeout. If the upstream is genuinely dead, a 30s timeout frees the connection 4x faster. If the upstream is just slow (generating a long response), streaming means data is still flowing — timeouts should only fire when *no data* arrives, not based on wall clock. A better approach (not in either plan): reset the timeout on each data event for SSE streams, so long generations don't timeout as long as they're making progress.

120s may cause a user to wait 2 minutes before getting a 502 instead of 30 seconds. For an interactive proxy, faster failure is better.

**Verdict: Keep Plan B's 30s, but add per-data-event reset for SSE streams.**

### 3.3 Connection pool sizing (over-engineered)

Plan A: `maxSockets: 32` with no other tuning.
Plan B: `maxSockets: 256`, `scheduling: 'lifo'`, `maxFreeSockets: 64`.

Both are extreme in opposite directions. 32 is too conservative — any concurrent LLM client with >32 concurrent requests will queue. 256 is too aggressive — opencode.ai likely has rate limits far below this.

Furthermore, `scheduling: 'lifo'` is wrong for a proxy. LIFO reuses the *most recently used* socket, which is great for minimizing idle socket churn but bad for load distribution. For a single upstream with keepalive, LIFO is actually fine since all sockets are equivalent. But the real issue is: both plans should have a moderate `maxSockets` (64-128) and drop the LIFO scheduling (it's an optimization with no measurable benefit at this scale).

**Verdict: Merge intermediate value (64-128). Drop LIFO.**

### 3.4 Cold start latency claim (wrong)

Plan A claims "Before: ~300ms (model fetch on import), After: ~20ms". A Node.js HTTP server takes ~50-100ms minimum to start even without any work. Claiming 20ms is false. The actual improvement is:
- Before: ~300ms to import + fetch models + start listening
- After: ~200ms to start listening (still need to initialize imports, crypto, etc.)

The `freeModelIdsPromise` is fetched at module scope currently. In the current script, this overlaps with server startup. In both plans, there's no reason it can't — they're independent operations. This "improvement" is marginal at best.

**Verdict: Acknowledge the marginal benefit but correct the numbers.**

### 3.5 `normalizePath` vs regex route table (style preference, not wrong)

Plan A uses `normalizePath()` to rewrite `/zen/v1/...` → `/v1/...` before routing. Plan B uses regex route table with separate entries. Both work. However:

- Plan A's approach means all error messages reference `/v1/...` paths even if the client sent `/zen/v1/...`. This is slightly misleading.
- Plan B's approach needs 2× entries per route.
- Plan A's approach is more DRY (one code path per route).

**Verdict: Preference. Either works. Plan B's regex table is more explicit about what paths are accepted.**

### 3.6 SSE as "special handling" (over-engineered)

Plan A dedicates a full section to SSE as if it requires special pipeline stages. It doesn't. SSE is just HTTP streaming with `text/event-stream` content-type. `.pipe()` forwards bytes unchanged. The only necessary additions are:
1. Set `content-type: text/event-stream` in response (already happens via proxy headers forwarding)
2. Disable buffering intermediaries (`cache-control: no-cache`, `x-accel-buffering: no`) — already in Plan B

Plan A's section 4.2 describes "flush after each data event" — this is what `.pipe()` does by default with `res.write()` (no Nagle buffering in Node). Not a special concern.

**Verdict: Plan B's approach is sufficient. Drop the SSE-as-special-treatment framing.**

---

## 4. Where the Two Plans Disagree

### 4.1 Zero config impact vs documented config change

**Plan A:** "Never touch `~/.openclaude/`. Zero disk I/O. No profile files."
**Plan B:** Suggests writing `~/.config/openclaude/profile.yml` with `base_url: "http://127.0.0.1:3456/v1"`.

This is a fundamental philosophical split:

- Plan A's position is correct for a *proxy tool* that shouldn't modify user configs. The user chooses which clients point at the proxy via env vars.
- Plan B's position is pragmatic — show the user exactly what to change. But writing the file is invasive.
- **Resolution:** The proxy should NOT write config files. But it should document (in comments at the top of the file or in a README) how to configure each supported client. Merge Plan A's zero-touch principle with Plan B's documentation of client configuration.

### 4.2 Lazy vs eager model fetch

- Plan A: Lazy (first request triggers fetch). Prevents unnecessary fetch. Adds latency to first request.
- Plan B: Eager (at module scope). Models ready at first request. No extra latency.

**Plan B is better for UX:** The first chat request shouldn't pay a 2-3s model fetch penalty. The proxy is always going to need models. The only case where lazy wins is if `/models` is never called — but the proxy serves both `/models` and `/chat/completions`, so models are needed regardless.

### 4.3 Default port choice

- Plan A: `PORT` env → 8080. Conventional HTTP proxy port.
- Plan B: `OPENCODE_FREE_PROXY_PORT` env → 0 (random). Namespaced env var, avoids conflicts.

Plan B's approach is safer (no port conflicts) but harder to use (must discover port from stderr output). Plan A's approach is more user-friendly for interactive use but risks conflict with other services on port 8080.

**Resolution:** Default to 8080 (Plan A), but respect `PORT` env var (Plan A). Add documentation that the user can set to 0 for random port if they need it. Use `OPENCODE_FREE_PROXY_PORT` as the namespaced override (Plan B), falling back to `PORT` (Plan A).

### 4.4 `start()` function vs module-level bootstrap

- Plan A: `server.listen()` at module level. Idiomatic for a single-purpose script.
- Plan B: `start()` function wrapping server creation. Enables testing and reusability.

Plan B's `start()` is better engineering — it allows the server to be imported, tested, and composed. For a pure proxy script, Plan A's approach is simpler. For a file that might be embedded in a larger tool, Plan B wins.

**Resolution: Keep Plan B's `start()` function, but call it at module level.** Best of both worlds — testable and executable.

### 4.5 Cluster mode

- Plan A: Not mentioned. Correctly ignores it.
- Plan B: Imports `cluster` (commented as "optional, only if multi-core"). Mentions it but doesn't use it.

Cluster mode is unnecessary for a proxy that forwards to a single upstream with connection pooling. Node.js can handle hundreds of concurrent connections in a single thread. The bottleneck is the upstream's rate limit, not CPU. Importing `cluster` (even unused) adds mental overhead.

**Resolution: Drop the cluster import. Plan A is correct to ignore it.**

### 4.6 SIGQUIT handling

- Plan A: SIGINT, SIGTERM only.
- Plan B: SIGINT, SIGTERM, SIGQUIT.

SIGQUIT on Unix generates a core dump by default. Adding a handler for it is fine, but it's not standard for graceful shutdown. If the user intends to kill the process with `kill -3`, they probably want a core dump, not a graceful shutdown.

**Resolution: Drop SIGQUIT. Plan A is correct — SIGINT + SIGTERM is standard.**

### 4.7 Agent destroy on shutdown

- Plan A: `server.close() → exit(0)`. Agent sockets are cleaned up by process exit.
- Plan B: `upstreamAgent.destroy()` before `server.close()`.

Plan B's approach is more correct for a library. For a CLI script that exits immediately after `close()`, the process exit handles cleanup. However, `upstreamAgent.destroy()` is a good practice and doesn't hurt.

**Resolution: Keep Plan B's `upstreamAgent.destroy()` — it's correct, even if the process exit would handle it.**

### 4.8 Forced exit timeout

- Plan A: `exit(0)` directly after `server.close()`.
- Plan B: `exit(0)` with 2s fallback timeout (`setTimeout(...).unref()`).

Plan B's approach is more robust — if `server.close()` hangs (e.g., a connection won't close), the timeout forces exit. Plan A assumes `close()` always completes. Plan B is safer.

**Resolution: Keep Plan B's forced-exit timeout.**

---

## 5. Summary of Merge Recommendations

| Component | Take from | Why |
|-----------|-----------|-----|
| Health check endpoint | Plan A | High value, low cost |
| CORS preflight handling | Plan A | Required for browser clients |
| Extended error format (`code`, `param`) | Plan A | Closer to OpenAI spec |
| Lazy model fetch | Plan B (eager) | Better UX, models always needed |
| `.pipe()` streaming | Plan B | Node handles backpressure natively |
| Timeout (60s with SSE reset) | Neither (merge) | 30s too short for long gen, 120s too long |
| Connection pool sizing (64-128) | Neither (merge) | Moderate between 32 and 256 |
| Default port (8080, `PORT` env) | Plan A | Easier for interactive use |
| `start()` function | Plan B | Enables testing |
| No cluster, no SIGQUIT | Plan A | Unnecessary complexity |
| Client disconnect cleanup | Plan B | Prevents orphaned upstream requests |
| Agent destroy on shutdown | Plan B | Good practice |
| Forced exit fallback | Plan B | Robustness |
| Zero config file modification | Plan A | Don't touch user's openclaude config |
| SSE header augmentation | Plan B | Sufficient without over-engineering |
| Namespaced port env var | Plan B (as override) | More discoverable than plain `PORT` |

**Net assessment:** Plan A is more feature-complete (health, CORS, error format) but over-engineers backpressure, timeout, and SSE. Plan B makes stronger engineering choices (`.pipe()`, `start()`, client cleanup) but misses health check and CORS preflight. The merged plan should take Plan B's architecture with Plan A's completeness improvements where they don't add unnecessary complexity.

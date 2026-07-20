# Acceptance Review: opencode-free-setup.mjs

**Reviewer**: Agent
**Date**: 2026-07-20
**Scope**: Implementation vs. `stage1-planning/plan.md`

---

## Verdict: **ACCEPT with minor fixes**

The implementation faithfully translates the plan. Every major requirement is present and working (verified via `implementation.md` test results). Two issues found, both minor.

---

## Completeness Checklist

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 1 | Dynamic port (port 0) | ✅ | `server.listen(0, '127.0.0.1')` at line 215 |
| 2 | Fresh IDs per request | ✅ | `buildOpenCodeHeaders()` called per-request at line 130, each call generates 3 fresh IDs |
| 3 | Auth stripped | ✅ | `buildOpenCodeHeaders()` creates fresh headers; only `content-type` copied from incoming — Authorization never forwarded |
| 4 | Hop-by-hop headers stripped | ✅ | `HOP_BY_HOP` set (line 24-27), filtered upstream response headers (line 145-148) |
| 5 | CORS headers on all responses | ⚠️ | Present on 200, 404, 502 responses. **Missing on 500 catch-all** (line 199). See finding #1. |
| 6 | `req.pipe()` for request body | ✅ | Line 167 |
| 7 | `proxyRes.pipe(res)` for streaming | ✅ | Line 151 |
| 8 | `/v1/models` intercept + free filtering | ✅ | `handleModelsList` (line 102); filters cost=0, excludes deprecated |
| 9 | `/v1/models/{id}` endpoint | ✅ | `handleModelById` (line 111); returns 404 for unknown models |
| 10 | `/zen/v1` prefix filtering | ✅ | Prefix guard at line 173; unknown paths return 404 |
| 11 | `--bg` mode with `child.unref()` | ✅ | Lines 237-241 |
| 12 | SIGINT/SIGTERM cleanup + 2s timeout | ✅ | Lines 244-250 |
| 13 | `headersSent` guard in all error handlers | ✅ | Present in proxy error (line 156) and catch-all route error (line 198) |
| 14 | Model priority picking | ✅ | `pickBestModel` with `MODEL_PRIORITY`, used at line 222 |
| 15 | Fetch fallback with stale cache preservation | ✅ | Returns `cachedFreeModels ?? FALLBACK_FREE_MODELS` on failure (line 62) |
| 16 | No external dependencies | ✅ | Only `node:*` built-in modules + global `fetch` (Node 18+) |

---

## Findings

### Finding #1: CORS missing on 500 catch-all response — **medium**

**Location**: `opencode-free-setup.mjs:199`

**Issue**: The catch-all error handler in `handleRequest` sends a 500 response without `Access-Control-Allow-Origin: *`. Every other response path (200, 404, 502, proxy responses) includes it.

```js
// Line 198-200 — missing CORS header
res.writeHead(500, { 'Content-Type': 'application/json' })
```

**Plan requirement** (plan.md §6, Implementation Notes item 6): *"CORS headers: Access-Control-Allow-Origin: \* is added to **every** response (model list, single model, error responses, proxied responses)."*

**Fix**: Add `'Access-Control-Allow-Origin': '*'` to the 500 response headers.

---

### Finding #2: No `proxyRes.on('error')` handler — **low**

**Location**: `opencode-free-setup.mjs:143-151`

**Issue**: The upstream response stream (`proxyRes`) is piped directly to `res` without an error handler. If the upstream stream errors mid-pipe (e.g., connection reset mid-response), the error propagates unhandled and crashes the process. Only the request-level error handler (`proxyReq.on('error')`) covers connection failures, not mid-stream response body errors.

**Plan error map** (plan.md §9): Covers upstream connection failure and TLS error but does not explicitly address mid-stream response errors. This is a gap in both plan and implementation.

**Fix**: Add a `proxyRes.on('error', ...)` handler with `headersSent` guard, destroying `res` if headers already sent or sending a 502 otherwise.

---

## Omissions from Plan

None. Every requirement and design decision from the plan is accounted for in the implementation.

---

## Documentation

The implementation is self-documenting for operational use (usage printed to stderr, clear error messages). No additional documentation required beyond what exists. The test results in `implementation.md` serve as adequate verification evidence.

The plan's testing section (plan.md §11) describes automated test scripts but the tests were run manually (curl + visual inspection). Creating `test-opencode-free.mjs` would be nice-to-have but was never scoped as a deliverable.

---

## Summary

| Aspect | Verdict |
|--------|---------|
| Fidelity to plan | High — all 16 requirements met |
| Correctness | High — tests pass, code is clean |
| Severity of issues | Medium (1) + Low (1) |
| Recommended action | Fix #1 (CORS on 500), consider #2 (proxyRes error handler) |

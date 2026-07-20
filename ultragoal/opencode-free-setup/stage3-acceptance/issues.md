# Issues: opencode-free-setup.mjs Acceptance Review

**Lead reviewer**: Orchestrator Agent
**Date**: 2026-07-20
**Status**: ACCEPTED (1 medium fix applied, remaining risks documented)

---

## Issues Must Fix

### #1: CORS header missing on 500 catch-all response — **Medium — FIXED**

**Source**: Review 1, Finding #1 (confirmed by all cross-reviews)
**Location**: `opencode-free-setup.mjs:199`
**Evidence**: 500 response `res.writeHead(500, { 'Content-Type': 'application/json' })` lacks `Access-Control-Allow-Origin: *`. Every other response path includes it.
**Recommended action**: Add CORS header to 500 response.
**Disposition**: FIXED. Simple one-line addition.

---

## Issues Not Fixed (Accepted Risks)

### #2: No `proxyRes` error handler (theoretical race window) — **Low — ACCEPTED**

**Source**: Review 1 Finding #2, Review 2 §1.3
**Evidence**: Between `httpsRequest` callback firing and `.pipe()` attachment, a `proxyRes` error could theoretically go unhandled.
**Analysis**: Node.js `pipe()` internally handles errors on the source stream. The race window is between callback execution and pipe attachment — on the order of microseconds. The upstream has already sent headers (thus the callback fired), so the connection is healthy.
**Remaining risk**: Negligible. No real-world crash scenario.
**Recommended action**: None for v1.

---

### #3: Hardcoded upstream path drops query parameters — **Medium — ACCEPTED**

**Source**: Review 2 §1.4 (confirmed by all)
**Location**: `opencode-free-setup.mjs:139`
**Evidence**: `path: '/zen/v1/chat/completions'` is hardcoded. The router uses exact-match (`===`), so query params would 404 before reaching this handler.
**Analysis**: Works correctly today. If openclaude ever adds query params to chat completion requests, compatibility breaks. Mitigation: the router would need updating anyway for new URL patterns.
**Remaining risk**: Low. openclaude doesn't send query params today. If needed, the fix is trivial.
**Recommended action**: None for v1.

---

### #4: No upstream connection timeout — **Medium — ACCEPTED**

**Source**: Review 2 §4.2 (confirmed by all)
**Evidence**: No `proxyReq.setTimeout()` call. If upstream connection stalls, the request hangs until TCP timeout (~2 min default).
**Analysis**: In practice, `opencode.ai` is a production API with reliable uptime. Network partitions are the primary risk, handled by TCP keep-alive and OS socket timeouts.
**Remaining risk**: Low. A stalled request could delay proxy shutdown by up to 2 minutes. The 2s force-exit timeout handles this in practice (unref'd timeout fires regardless).
**Recommended action**: None for v1.

---

### #5: `cachedFreeModels` effectively dead code — **Low — ACCEPTED**

**Source**: Review 2 §1.6 (confirmed by all)
**Evidence**: `cachedFreeModels` is set in `main()` but only read inside `fetchFreeModels()`, which is called once at module scope. The stale-cache path never triggers.
**Analysis**: No functional impact. The variable only matters if retry/re-fetch logic is added later.
**Remaining risk**: None. Harmless.
**Recommended action**: None for v1.

---

### #6: Missing `Accept` / `Accept-Encoding` headers — **Low — ACCEPTED**

**Source**: Review 2 §2.5 (confirmed by all)
**Evidence**: Proxy doesn't send `Accept` or `Accept-Encoding` headers that pi-opencode-zen may send.
**Analysis**: Minor fingerprinting difference. The critical anti-detection headers (User-Agent, x-opencode-*) are correct and match pi-opencode-zen exactly.
**Remaining risk**: Very low. opencode.ai likely doesn't fingerprint by `Accept` headers.
**Recommended action**: None for v1.

---

### #7: Proxy stays alive in `--bg` mode after child exits — **Low — ACCEPTED**

**Source**: Review 2 §4.7 (confirmed by all)
**Evidence**: In `--bg` mode, `child.unref()` detaches the child lifecycle from the proxy. If the child exits, the proxy stays running.
**Analysis**: Documented behavior. The user manages the proxy lifecycle (pkill or kill). The proxy continues serving model list requests even without an active child.
**Remaining risk**: User must remember to kill the proxy. Standard background-process management.
**Recommended action**: None for v1.

---

### #8: Hardcoded openclaude path — **Low — ACCEPTED**

**Source**: Review 3, Concern 1 (confirmed by all)
**Location**: `opencode-free-setup.mjs:232`
**Evidence**: `/usr/local/bin/openclaude` hardcoded. May fail if openclaude is in a non-standard location.
**Analysis**: The plan explicitly specified this path. `which openclaude` or `command -v` lookup would be more portable but adds complexity.
**Remaining risk**: Low in standard npm global install environments. Medium if deployed on systems with non-standard Node.js installations.
**Recommended action**: None for v1.

---

### #9: One-time model list fetch — **Low — ACCEPTED**

**Source**: Review 3, Concern 2 (confirmed by all)
**Evidence**: `freeModelIdsPromise` resolves once at startup. Long-running proxy never sees new free models without restart.
**Analysis**: For a minimal proxy, a startup-only fetch is acceptable. New models are rare (weeks/months). Restarting the proxy picks up changes.
**Remaining risk**: Low. If a new free model is added while the proxy is running, it won't appear until restart.
**Recommended action**: None for v1.

---

### #10: No body model validation on chat completions — **Low — ACCEPTED**

**Source**: Review 3, Concern 3 (confirmed by all)
**Evidence**: Chat completions handler doesn't check that the requested model is in the free list.
**Analysis**: Intentional design per the plan (§6). The `/v1/models` intercept provides correct defaults. Users overriding `OPENAI_MODEL` to a paid model are making an intentional choice.
**Remaining risk**: Low. The proxy doesn't enforce free-model-only on the chat endpoint. Mitigated by `/v1/models` filtering and correct default `OPENAI_MODEL`.
**Recommended action**: None for v1.

---

### #11: Fallback model freshness — **Low — ACCEPTED**

**Source**: Review 3, Concern 4 (confirmed by Review 1, corrected by Review 2)
**Evidence**: `FALLBACK_FREE_MODELS` hardcoded list may become stale.
**Analysis**: The fallback only activates when `models.dev` is unreachable (network failure). It's a last-resort, not a primary data source. A committed JSON file adds complexity without proportional benefit.
**Remaining risk**: Very low. The fallback list is a safety net, not the primary path.
**Recommended action**: None for v1.

---

## Summary

| # | Issue | Severity | Disposition |
|---|-------|----------|-------------|
| 1 | CORS missing on 500 | Medium | **FIXED** |
| 2 | proxyRes error race window | Low | Accepted |
| 3 | Hardcoded upstream path | Medium | Accepted |
| 4 | No upstream timeout | Medium | Accepted |
| 5 | cachedFreeModels dead code | Low | Accepted |
| 6 | Missing Accept headers | Low | Accepted |
| 7 | --bg proxy lifecycle | Low | Accepted |
| 8 | Hardcoded openclaude path | Low | Accepted |
| 9 | One-time model fetch | Low | Accepted |
| 10 | No body model validation | Low | Accepted |
| 11 | Fallback model freshness | Low | Accepted |

**No blocker issues. Acceptance passes.**

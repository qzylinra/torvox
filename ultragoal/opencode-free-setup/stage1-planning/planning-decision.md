# Planning Decision

## Consensus Points (both cross-reviews agree)

| Issue | Verdict | Source |
|-------|---------|--------|
| Architecture | **Single file** (Plan A) over two-file split (Plan B) | Both |
| Port management | **Dynamic port 0** (Plan A) over static 3180 (Plan B) | Both |
| Auth handling | **Strip Authorization** before upstream (Plan A) | Both |
| Model validation | **Skip it** — `/v1/models` filtering is sufficient | Both |
| SSE streaming | **`pipe()` for response** — both do this correctly | Both |
| Model list endpoint | **Include `/v1/models/{id}`** (Plan A) for compatibility | Both |
| `--bg` mode | **Support it** (Plan A) with `child.unref()` | Both |
| Exit propagation | **Pass through child exit code** (Plan A) with 2s timeout | Both |

## Disagreements Resolved

**Session/project ID generation:**
- Cross-Review A: Generate fresh IDs per request (matches pi-opencode-zen's `opencodeHeaders()` pattern)
- Cross-Review B: Generate once per proxy lifetime (session continuity)
- **Ruling: Fresh per request.** pi-opencode-zen calls `opencodeHeaders()` inside the per-request stream function, generating new IDs each time. "同样的http请求" means identical behavior — use fresh IDs per request.

## Greenlit Adoptions from Plan B

1. **Raw pipe for request body** (`req.pipe(proxyReq)`) — zero-copy forwarding
2. **Hop-by-hop header stripping** — full list from Plan B
3. **Fresh x-opencode-* IDs per request** — faithful to pi-opencode-zen
4. **Upstream error `headersSent` guard** — prevent late-error crashes
5. **CORS headers** on model list response (`Access-Control-Allow-Origin: *`)
6. **`/zen/v1` prefix filtering** — only handle known paths
7. **Cache fallback preserving stale data** — more robust on transient fetch failures
8. **Remove request body buffering** — use `req.pipe()` instead

## Designated Integrator

**Plan A's author** is selected to integrate the final plan, since Plan A's architecture (single-file, dynamic port, correct header lifecycle) forms the stronger foundation. Plan B's per-request improvements will be merged in.

## Final Plan

Write the integrated plan to `stage1-planning/plan.md`.

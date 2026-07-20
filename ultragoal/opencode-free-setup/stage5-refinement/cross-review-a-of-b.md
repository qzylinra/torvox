# Cross-Review: Plan A Reviews Plan B

## 1. What Plan B Does Differently

**Route dispatch**: Plan B uses a 6-entry regex route table (`ROUTES` array with `{ method, pattern, handler }`) and linear scan. Plan A uses a `normalizePath` function stripping `/zen/v1/` → `/v1/` then simple `if/else` chain. Plan B duplicates each path for `/v1/` and `/zen/v1/` variants in the patterns; Plan A normalizes once and dispatches once.

**Agent config**: `maxSockets: 256` vs Plan A's `32`. `keepAliveMsecs: 1000` vs `30_000`. Adds `scheduling: 'lifo'` and `maxFreeSockets: 64`.

**Timeout**: 30s vs Plan A's 120s.

**Backpressure**: No explicit handling — uses plain `.pipe()` (same as current code). Plan A adds drain-aware manual piping.

**Health check**: None. Plan A has `GET /health`.

**CORS**: No OPTIONS preflight handling. Plan A handles `OPTIONS *` → 204.

**Client disconnect**: Adds `req.on('close') → proxyReq.destroy()`. Plan A doesn't mention this.

**Testing**: `node:test` unit tests + integration tests. Plan A uses manual curl tests only.

**Port env var**: `OPENCODE_FREE_PROXY_PORT` vs Plan A's `PORT`.

**Node version**: Explicit minimum (18+). Plan A doesn't state one.

**Cluster**: Mentions optional `cluster` import. Plan A doesn't.

---

## 2. Ideas to Incorporate into Final Plan

| Idea | Why |
|------|-----|
| `scheduling: 'lifo'` | Uses warmer sockets first; harmless micro-optimization |
| `x-accel-buffering: no` for SSE | Prevents intermediary buffering on nginx/proxies |
| `req.on('close') → proxyReq.destroy()` | Prevents orphaned upstream requests on client disconnect |
| `UPSTREAM_PATH_PREFIX` constant | Maintainability; single point to update upstream path |
| Explicit Node 18+ minimum | Useful documentation for users |
| Timeout integration test (black-hole) | Clever way to test timeout behavior deterministically |
| Client disconnect risk assessment | Real gap Plan A missed — worth documenting |

---

## 3. What Plan B Gets Wrong or Over-Engineers

**Wrong:**

- **30s timeout**: Too tight. LLM API responses for long generations regularly exceed 30s. 120s (Plan A) is the right default. Plan B's value will cause false-positive timeouts in normal use.
- **`keepAliveMsecs: 1000`**: 1 second idle before dropping the socket defeats the purpose of keep-alive. A 1s gap between prompts (e.g., reading the response) kills the socket. Plan A's 30s is correct — keeps the socket warm across typical think-time between requests.
- **No health check**: This is a practical gap. Kubernetes, Docker, systemd, and simple monitoring all rely on health endpoints. Omitting it limits deployability for zero gain.

**Over-engineered:**

- **`maxSockets: 256`**: Ridiculous over-provisioning for a personal proxy handling 1 concurrent user. `maxSockets: 32` (Plan A) is already generous. 256 just wastes connection bookkeeping overhead.
- **`cluster` module mention**: A single-threaded Node process handles this load trivially. Cluster adds complexity, shared-state confusion, and port conflict risk for zero benefit.
- **Route table with 6 entries**: The `/v1/` + `/zen/v1/` duplication doubles the pattern count. Plan A's `normalizePath` approach is simpler and equally correct — one path to match, the other is derived.
- **`maxFreeSockets: 64`**: Fine as a guard but unnecessary at this scale. Harmless but extra config surface.

---

## 4. Points of Disagreement

| Topic | Plan A | Plan B | Verdict |
|-------|--------|--------|---------|
| Timeout | 120s | 30s | Plan A — 30s will break on long LLM responses |
| Keep-alive idle | 30s | 1s | Plan A — 1s is too short to be useful |
| Max sockets | 32 | 256 | Plan A — 256 is wasteful for single-user |
| Port env name | `PORT` | `OPENCODE_FREE_PROXY_PORT` | Plan A — follows standard convention used by Express, serve, webpack-dev-server, etc. |
| Backpressure | Explicit drain-aware | `.pipe()` only | Plan A — more robust under memory pressure |
| Health check | `/health` | None | Plan A — practical for deployment |
| Routing | normalizePath + if/else | Regex table, 6 entries | Both work; Plan A is simpler and avoids path duplication |
| Testing | `curl` smoke tests | `node:test` units + integration | Plan B is more thorough but Plan A's approach matches the zero-dep constraint better |
| Error schema | Full OpenAI (`code`, `param`) | Minimal (`message`, `type` only) | Plan A — stricter SDK clients expect full shape |

---

## Summary

Plan B has three genuinely good ideas Plan A missed: client disconnect cleanup, LIFO socket scheduling, and SSE anti-buffering headers. Everything else Plan B does differently is either wrong (30s timeout, 1s keepalive, no health check) or over-engineered for a personal utility proxy (256 sockets, cluster, route table with duplicated entries). The final plan should adopt Plan B's good ideas and keep Plan A's architecture as the foundation.

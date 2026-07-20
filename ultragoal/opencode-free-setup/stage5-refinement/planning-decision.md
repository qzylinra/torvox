# Planning Decision: Stage 5 Refinement

**Date**: 2026-07-20
**Ruling by**: Lead Reviewer (Orchestrator)

## Disposition

**Merged approach: Plan B's architecture with Plan A's feature completeness.**

---

## Remaining Disagreements & Rulings

| Topic | Plan A | Plan B | Ruling | Rationale |
|-------|--------|--------|--------|-----------|
| **Timeout** | 120s | 30s | **60s** with per-data-event reset for SSE streams | 30s too short (long generations cut off), 120s too long (bad UX on dead upstream). Per-event reset ensures long streams work. |
| **Pool sizing** | maxSockets: 32 | maxSockets: 256 | **maxSockets: 64**, no LIFO, no maxFreeSockets | Both extreme; 64 is moderate. LIFO offers no benefit for single-upstream proxy. |
| **Port env var** | `PORT` → 8080 | `OPENCODE_FREE_PROXY_PORT` → 0 | **Default 8080, `PORT` env, `OPENCODE_FREE_PROXY_PORT` override** | Namespaced override for discovery; standard `PORT` for familiarity. |
| **Model fetch** | Lazy (first /models request) | Eager (module scope) | **Eager** | Models needed for ALL requests (model selection for chat). No benefit to deferring. |
| **Backpressure** | Manual drain-aware pipe | `.pipe()` only | **`.pipe()`** | Node's pipe is battle-tested and correctly handles backpressure. Manual pipe adds bug surface. |
| **Route dispatch** | normalizePath + if/else | Regex table, 6 entries | **normalizePath** | DRY, simpler, fewer bugs. |
| **Error format** | Full OpenAI (code, param) | Minimal (message, type) | **Full OpenAI** | SDK clients may expect code/param fields for retry logic. |
| **LIFO scheduling** | Not mentioned | LIFO | **Drop LIFO** | No measurable benefit vs FIFO default. |
| **Cluster** | Not mentioned | Optional import | **Drop** | Unnecessary for single-user proxy. |
| **SIGQUIT** | SIGINT+SIGTERM only | +SIGQUIT | **SIGINT+SIGTERM only** | SIGQUIT normally produces core dump; overriding breaks user expectation. |
| **Config modification** | Zero touch | Suggests writing profile | **Zero touch** | Proxy must NOT write config files. Document client setup in comments. |

---

## Points of Agreement (adopted from both)

Adopted from **Plan A**:
- Health check endpoint `GET /health`
- CORS preflight `OPTIONS` → 204
- Full OpenAI error shape (`code`, `param`)
- normalizePath approach

Adopted from **Plan B**:
- `start()` function wrapping server creation (testable)
- Client disconnect cleanup (`req.on('close') → proxyReq.destroy()`)
- SSE header augmentation (`x-accel-buffering: no`, `cache-control: no-cache`)
- Agent destroy + forced exit timeout on shutdown
- Connection pool with `https.Agent({ keepAlive: true })`
- Dual-path support with `UPSTREAM_PATH_PREFIX` constant

---

## Integration Assignment

**Designate**: Plan B's author to integrate the final plan.

**Rationale**: Plan B's architecture (start() function, .pipe(), client disconnect, clean shutdown) provides a stronger foundation. Plan B only needs to incorporate Plan A's feature additions (health check, CORS preflight, full error format, normalizePath). The merge is straightforward.

**Instructions for the integrator**:
1. Read both plans (`plan-a.md` and `plan-b.md`) thoroughly
2. Read this decision document
3. Write the final merged plan to `plan.md`
4. The final plan must be a complete, self-contained specification for the implementation sub-agent to follow
5. Include the final code diff as a reference implementation

# Cross-Review: Plan B author evaluates Plan A

## 1. What Plan A does better — ideas to adopt

**Single-file architecture.** Plan A's one-file design (`opencode-free-setup.mjs`) is strictly better than Plan B's two-file split (`opencode-free-proxy.mjs` + `openclaude-free.mjs`). The proxy and launcher live in one process, so port communication is free — `server.address().port` is available immediately after `listen()` (Plan A line 371). Plan B requires a sentinel file hack (lines 422-432) that Plan A's author correctly avoided. Adopt: **merge launcher and proxy into one file**.

**Dynamic port (port 0).** Plan B acknowledges this is better in edge case 5 (line 421: "Solution: Let the OS assign a port") but keeps static 3180 as the implementation. Plan A does it right — `server.listen(0)` + read port synchronously (line 12-13). Port conflicts eliminated entirely. Adopt: **use `port: 0`**.

**Stable session/project IDs.** Plan A generates `x-opencode-session` and `x-opencode-project` once per proxy lifetime (line 50), matching pi-opencode-zen's per-extension-instance behavior. Plan B generates fresh IDs for every request in `opencodeHeaders()` (line 88-97), which would make every request appear to come from a different extension instance — detectable by a server that tracks session continuity. Adopt: **generate session/project IDs once at startup**.

**Auth stripping.** Plan A explicitly strips `Authorization` from forwarded requests (line 55). Plan B copies it through (line 153-155), which would send `public` or whatever openclaude supplies to the upstream — potentially causing rejection. Adopt: **strip Authorization before forwarding to opencode.ai**.

**Correct hop-by-hop handling.** Plan A strips `content-length` from upstream responses (line 73) — correct for streaming. Plan B strips `transfer-encoding` (line 200) which would break chunked transfer — this is a bug in Plan B. The `transfer-encoding` header must be preserved for the client to decode the stream. Adopt: **Plan A's header forwarding logic**.

**Better cleanup architecture.** Plan A handles child exit → server close → process exit (lines 228-233) with a 2s force-kill timeout. Plan B's `shutdown()` (line 236-239) just calls `server.close()` and `process.exit(0)` without killing the child. Plan A also passes the child's exit code through (`process.exit(code ?? 1)`). Adopt: **Plan A's exit propagation and force-kill timeout**.

**CLI args forwarding.** Plan A passes `process.argv.slice(2)` to the child (line 199), so users can pass openclaude flags through the wrapper. Plan B only passes them through the launcher (line 348) but the launcher doesn't forward its own args — `process.argv.slice(2)` would include the launcher's own name. This is a bug in Plan B's launcher. Adopt: **forward remaining CLI args**.

---

## 2. What Plan A gets wrong that Plan B avoids

**POST body buffering.** Plan A buffers the full incoming body before forwarding (line 64: "Buffer incoming request body" + line 374). Plan B's `req.pipe(proxyReq)` (line 226) forwards bytes as they arrive — zero-copy, zero-latency. For small JSON payloads this is a minor issue (a few KB), but Plan A's approach prevents model validation in the request path (you'd need to buffer anyway for that, but Plan A doesn't do it either — see §6).

**No `Access-Control-Allow-Origin` on model list.** Plan B sets `Access-Control-Allow-Origin: *` on intercepted model responses (line 176). Plan A doesn't mention CORS headers. If openclaude runs in a browser context or a webview, this could cause CORS failures. Minor since openclaude is a CLI tool.

**Missing `/zen/v1` prefix in routing.** Plan B's proxy only handles paths starting with `/zen/v1` (line 141), so it doesn't accidentally match non-OpenAI paths. Plan A doesn't specify path filtering — any route (including `/v1/embeddings`) gets forwarded if not explicitly intercepted. Plan B's approach is safer for filtering.

**No upstream error resilience.** When the upstream connection fails mid-stream, Plan B checks `!res.headersSent` before writing the error (line 217). Plan A's upstream error handler (line 76) doesn't check `headersSent`, so a late error could throw after the response headers are already sent.

**Model field completeness.** Plan B's model list includes only `id` and `object` (line 172). Plan A includes `created` and `owned_by` (lines 93-94). Neither includes `created` timestamps, which OpenAI clients may expect. Plan A is slightly more complete.

**Fallback logic.** Plan A's `fetchFreeModels()` returns `null` on failure (line 127), then uses fallback. Plan B's implementation preserves any previously cached data and only uses fallback if cache is empty (line 123: `cachedFreeModels ?? FALLBACK_FREE_MODELS`). Plan B's approach is more robust for transient network failures during proxy lifetime.

---

## 3. Single-file vs two-file

**Single-file is better for this use case.** Reasons:

- **No inter-process communication.** Port is read directly from `server.address()` in the same process (Plan A line 371). Plan B's two-file approach needs a sentinel file or a health-check polling loop (lines 335-345) — more moving parts.
- **Atomic lifecycle.** In Plan A, the same process owns both proxy and child — signal handling, cleanup, and exit code propagation are trivial. Plan B has two processes that must coordinate shutdown.
- **One file to distribute.** User copies one file, runs one command. No `PROXY_PORT` env var needed, no separate launcher script.
- **Testability tradeoff.** Plan B claims independent proxy testing as an advantage, but Plan A's single file can still be tested by importing it as a module or starting it with `--bg`. The benefit is marginal.

**Verdict:** Adopt Plan A's single-file approach.

---

## 4. Dynamic port (port 0) vs static port

**Dynamic port is strictly better** and Plan B's own edge case analysis (line 421) acknowledges this. There is no downside to `port: 0`:
- No port conflicts — the OS guarantees uniqueness
- No `PORT` env var to configure — one less thing to document
- Port is known immediately after `listen()` completes — used to set `OPENAI_BASE_URL` in env

The only reason Plan B uses static port is that the two-file approach makes port communication harder (parent can't read child's `server.address().port`). This is another reason single-file wins.

**Verdict:** Adopt Plan A's `port: 0` approach.

---

## 5. POST body buffering and SSE

**Plan A buffers the POST body before forwarding (line 64, 374). Plan B does not (line 226: `req.pipe(proxyReq)`).** Is this a problem for SSE?

**No — the RESPONSE is never buffered in either plan.** Both use `upstreamRes.pipe(res)` (Plan A line 75, Plan B line 212) for the response stream. SSE chunks flow through immediately once the upstream connection is established. The request body buffering only affects:
1. **Time-to-first-byte (TTFB):** Plan A adds the time to read the full body before sending the upstream request. For a typical chat completion request (~1 KB body) this is a single TCP packet — latency is negligible (<1ms).
2. **Memory:** Only the request body is buffered (~1 KB), not the response stream. Not a concern.
3. **Model validation:** Plan A is already positioned to validate the model in the buffered body (it has it). Plan B would need to buffer first anyway to implement validation (lines 297-298).

**Plan A's buffering is harmless for this use case.** Plan B's zero-copy approach is theoretically cleaner but provides no practical benefit for small JSON payloads. If buffering large request bodies (megabytes) became relevant later, switch to piping — but that's not this use case.

**However**, Plan A should document that only the *request* body is buffered, and the *response* is streamed. Line 64 says "Buffer incoming request body" but line 79 says "pipe() handles SSE natively" — this could confuse a reader into thinking the response is also buffered.

---

## 6. Model validation on chat completions

**Both plans skip it correctly.** Neither validates the model ID on POST `/v1/chat/completions`:
- Plan A: no mention of model validation on chat completions — forwards everything
- Plan B: explicitly discusses (lines 296-310) and decides against it: "trust the `/v1/models` filtering + the user's `OPENAI_MODEL` env var"

**Should Plan A add it? No,** for the same reasons Plan B identifies:
1. **Model list filtering is sufficient.** openclaude discovers available models from the filtered `/v1/models` response and auto-selects the first free model. The default path is correct.
2. **User override is intentional.** If a user sets `OPENAI_MODEL=claude-sonnet-4-2025-05-14`, they're explicitly asking for a paid model — the proxy shouldn't gatekeep.
3. **Buffering cost.** Validation requires buffering the POST body, which adds latency. Plan A already buffers, so this cost is already paid — but adding validation logic adds code complexity.
4. **False positives.** If a new free model appears between cache refreshes, validation would incorrectly reject it.

**If we wanted belt-and-suspenders**, add a soft warning in the chat completion handler when the model isn't in the free list, but still forward the request. Not worth the code for v1.

---

## 7. Overall recommendation

Plan A is the better design. Key wins:

| Criterion | Winner | Why |
|-----------|--------|-----|
| Architecture simplicity | Plan A | Single file, no IPC, atomic lifecycle |
| Port management | Plan A | `port: 0` eliminates conflicts |
| Header session continuity | Plan A | Stable session/project IDs match pi-opencode-zen |
| Auth handling | Plan A | Strips Authorization before upstream |
| SSE/streaming | Tie | Both use `pipe()` for response |
| Hop-by-hop headers | Plan A | Correctly strips `content-length`, preserves `transfer-encoding` |
| POST body handling | Plan B | `req.pipe()` is zero-copy, but Plan A's buffer is harmless |
| Error resilience | Plan B | `headersSent` check is more robust |
| Exit code propagation | Plan A | Passes child exit code through |
| Force-kill timeout | Plan A | 2s timeout prevents hang on shutdown |
| CORS headers | Plan B | Sets `Access-Control-Allow-Origin` (minor) |
| Fallback robustness | Plan B | Preserves stale cache on fetch failure |

**Recommendation: Take Plan A as the base, then patch in fixes from Plan B:**

1. Add a `headersSent` guard in the upstream error handler (Plan B line 217)
2. Add CORS headers on intercepted model list (Plan B line 176)
3. Add `Access-Control-Allow-Origin` for completeness
4. Fix the body-buffering documentation to clarify request-only, not response
5. Consider adding `/zen/v1` prefix filtering for safety (Plan B line 141)

The single-file dynamic-port approach is the right foundation. Plan B's two-file design is more complex without commensurate benefit.

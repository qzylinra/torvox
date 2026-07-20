# Code Review: free-zen.py & test_free_zen.py

## Summary

Two Python files implementing a local HTTP proxy that filters OpenCode Zen to free models for openclaude. 15/15 tests pass. The code works end-to-end (models served, chat completions streamed, openclaude connection confirmed), but has several issues in the proxy handler layer — the test suite does not cover the `ZenProxyHandler` class at all, and the streaming/response path has structural bugs.

---

## Issues

### B-1: `_stream` strips `Transfer-Encoding` — SSE responses lack framing (high)

`_stream()` at `free-zen.py:104` strips `transfer-encoding` from upstream headers before forwarding. The upstream `/chat/completions` endpoint uses `Transfer-Encoding: chunked` for SSE. Without any content-length or transfer-encoding, the downstream HTTP library cannot determine response boundaries. Whether this works depends on the client's HTTP library and connection-close behavior; it is a compliance bug.

**Severity: high** — breaks HTTP/1.1 framing for streaming responses.

### B-2: `_send_regular` strips `Content-Encoding` — compressed responses break (high)

Both `_send_regular()` and `_stream()` strip `content-encoding` from the forwarded headers. If the upstream returns gzipped JSON (common for `/v1/models`), the proxy forwards the compressed bytes to the client without the `Content-Encoding: gzip` header, causing a parse failure.

In practice, `urllib.request.urlopen` does not auto-decompress, and the upstream likely serves these endpoints uncompressed (confirmed working manually), but this is a brittle assumption.

**Severity: high** — latent failure on any upstream config change.

### B-3: `--no-color` argument is dead code (medium)

`free-zen.py:194-195` defines `--no-color` in argparse, but `args.no_color` is never read anywhere in the code. The `--list` output does not emit ANSI codes. This is dead code that misleads users.

**Severity: medium** — misleading CLI, violates "no dead code" convention in AGENTS.md.

### B-4: Unused module-level annotations shadow real type (low)

`_free_model_ids: Set[str]` and `_free_models: List[Dict[str, str]]` are annotated as module-level types, but `main()` reassigns them with `global`. The types are consistent with the reassignments, but annotating mutable globals that are later `global`-reassigned is confusing and bypasses type-checker guarantees.

**Severity: low** — clarity issue.

### B-5: `_send_regular` sends raw upstream body without decompression check (medium)

`free-zen.py:66-67` reads `data = upstream.read()` from an `HTTPResponse`. `urllib.request.urlopen` does not auto-decompress. If the upstream returns gzipped content, `data` is compressed bytes. The method then re-encodes filtered JSON with `json.dumps(doc).encode()`, discarding the original compressed body. But the exception path (`HTTPError`) calls `e.read()` and passes `e.headers` through — if the error body was compressed, stripping `Content-Encoding` breaks the client there too.

**Severity: medium** — affects error-path reliability.

---

## Test Coverage Gaps

### T-1: `ZenProxyHandler` has zero coverage (high)

All 15 tests mock `ThreadingHTTPServer` to avoid actually starting a server. The `ZenProxyHandler` class (`free-zen.py:46-126`) — header injection, model filtering in responses, streaming, error handling — is entirely untested.

**Severity: high** — core proxy logic has no automated verification.

### T-2: No test for response model filtering correctness (high)

`_proxy()` at `free-zen.py:64-79` filters `/models` responses by matching model IDs against `_free_model_ids`. There is no test that the filtered response contains correct JSON, that non-matching models are removed, or that non-`/models` paths pass through unmodified.

**Severity: high** — the main feature (free-model-only filter) is untested at the proxy level.

### T-3: No test for streaming path (high)

`_stream()` is never exercised by tests. Chunk read loop, flush, connection lifecycle, and error handling during streaming are all untested.

**Severity: high** — streaming is core to the chat completions use case.

### T-4: No test for error response forwarding (medium)

`_proxy()` catches `HTTPError` (line 82) and generic `Exception` (line 84). Neither path has tests verifying status code, headers, or body are forwarded correctly.

**Severity: medium** — error resilience unverified.

### T-5: No test for `--port` with explicit value (low)

`free-zen.py:174-178` defines `--port` but no test exercises it. The default port=0 (random) is implicitly tested via mocks.

**Severity: low** — low risk, but a gap.

### T-6: No test for `--timeout` (low)

`free-zen.py:188-192` defines `--timeout` but no test exercises it.

**Severity: low** — low risk.

### T-7: `test_uses_best_model` mocks server address as `0` (low)

`test_free_zen.py:154` sets `server_addr=("127.0.0.1", 0)`. The env export output says `http://127.0.0.1:0` which is not a valid listen address. The test passes because it only checks string containment, but the mock value is misleading.

**Severity: low** — test mock quality issue.

### T-8: Brittle module import via `importlib` (low)

`test_free_zen.py:9-15` uses `importlib.util.spec_from_file_location` to load a module with a hyphen in its filename. This is non-standard and fragile (depends on `sys.path.insert(0, ".")`).

**Severity: low** — works but unidiomatic.

---

## Security

### S-1: No upstream response size limit (medium)

`_proxy()` reads the full upstream response body into memory (`upstream.read()`) for the `/models` path. A malicious or malfunctioning upstream returning a multi-GB response causes OOM.

**Severity: medium** — resource exhaustion vector (local machine).

### S-2: Broad exception catch leaks internal error details to client (low)

`free-zen.py:84-85` catches all `Exception` and sends `str(e)` to the client via `send_error(502, ...)`. Internal details (network errors, traceback context) leak.

**Severity: low** — localhost-only proxy limits blast radius.

### S-3: Path concatenation without sanitization (info)

`ZEN_BASE_URL + self.path` at `free-zen.py:57` concatenates the incoming request path directly. URL components like query strings are correctly handled by `urllib.request.Request`, but path traversal or encoded characters are not validated. Listening on `127.0.0.1` limits exposure.

**Severity: info** — localhost-only, low risk.

### S-4: `urllib.request` follows redirects by default (info)

`urllib.request.urlopen` follows up to 30 redirects. An upstream compromise could redirect the proxy to an internal service. Limited to localhost listening.

**Severity: info** — defense-in-depth concern.

---

## Code Quality

### C-1: `signal.pause()` is Unix-only (low)

`free-zen.py:272` uses `signal.pause()`, which exists only on Unix. Falls back to `KeyboardInterrupt` handling, but `signal.pause()` itself will raise `AttributeError` on Windows.

**Severity: low** — not Windows-targeted, but unstated.

### C-2: Module-level globals with `global` mutation (low)

`_free_model_ids` and `_free_models` are module-level globals modified via `global` in `main()`. This works for the single-use CLI pattern but is not thread-safe.

**Severity: low** — single-threaded use case.

### C-3: `log_message` suppression hides diagnostics (low)

`free-zen.py:125-126` suppresses all logging. Debugging proxy issues (connection drops, malformed requests) requires code modification.

**Severity: low** — intentional for clean output, but should be toggleable.

### C-4: No graceful shutdown on SIGTERM (low)

`signal.pause()` only handles `KeyboardInterrupt` (SIGINT). SIGTERM kills the process without clean server shutdown. `ThreadingHTTPServer.__enter__` context manager could be used for cleaner lifecycle.

**Severity: low** — minor operational concern.

---

## Requirements Compliance

| Req | Status | Notes |
|-----|--------|-------|
| No openclaude config file modification | ✅ | Env vars only |
| No OPENCODE_API_KEY set | ✅ | Only OPENAI_API_KEY="" |
| Python stdlib only | ✅ | `http.server`, `urllib`, `json`, `uuid`, etc. |
| Dynamic model fetch per run | ✅ | `fetch_free_models()` on every `main()` call |
| Free models only (cost.input == 0, not deprecated) | ✅ | Correct filter logic |
| Same HTTP headers as real opencode client | ✅ | User-Agent + 4 x-opencode-* headers with UUIDs |
| Works with openclaude config | ✅ | Verified manually |

---

## Recommendations

1. **Fix `_stream`**: Keep `Transfer-Encoding: chunked` in the forwarded response for streaming endpoints. For SSE, the upstream chunked encoding should pass through.
2. **Fix `_send_regular` content-encoding**: Either auto-decompress on read and strip `Content-Encoding`, or forward the header as-is. Don't strip without decompressing.
3. **Remove `--no-color` dead code**: It's defined but never referenced.
4. **Add unit tests for `ZenProxyHandler`**: Test header injection, model filtering, streaming, and error responses via `urllib.request` against a local server instance (the `ThreadingHTTPServer` is already created in `main()` — extract and test).
5. **Add size limit on upstream reads**: Cap at ~10 MiB for `/models` to prevent OOM.
6. **Replace `signal.pause()` with `server.serve_forever()` in main thread**: Keeps the main thread in `serve_forever()` and moves the signal handler to a clean shutdown path.

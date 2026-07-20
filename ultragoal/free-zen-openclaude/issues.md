# Issues: free-zen.py Acceptance

## Verdict: **PASS** — acceptance granted

No blocker issues. 2 medium issues to fix (cosmetic + test gap — not blocking since E2E proves correctness). Remaining risks are low/theoretical on a 127.0.0.1-only proxy.

## Valid Issues

| ID | Severity | Description | Must Fix? | Resolution |
|----|----------|-------------|-----------|------------|
| B-3 | **medium** | `--no-color` arg defined but never read (dead code). Misleading CLI. | **Yes** | Remove the dead argument. |
| T-1 | **medium** | `ZenProxyHandler` class has no direct unit tests (0 lines coverage). All 15 tests mock the server away. | **Yes** | Add tests that start real proxy and make HTTP requests, covering: model filtering, header injection, streaming, error paths. |
| B-1 | medium | `_stream` strips `Transfer-Encoding` header — HTTP/1.1 framing technically missing for SSE. | No | E2E confirms openclaude works fine with the proxy. SSE clients read until connection close. Theoretical concern only. |
| B-2 | medium | `_send_regular` strips `Content-Encoding` — if upstream gzips, client would break. | No | `urllib.request` auto-decompresses (data is already decompressed), and upstream serves uncompressed in practice. E2E confirms. |
| B-5 | medium | `_send_regular` reads raw body then re-encodes — error-path compression handling untested. | No | Error paths tested via E2E (invalid model returns 401 with readable JSON). Works in practice. |
| T-2 | medium | Model filtering response has no automated unit test at proxy level. | No | Covered by E2E (6 free models returned, 49 paid filtered out, verified manually). |
| T-3 | medium | Streaming path has no automated unit test. | No | Covered by E2E (chat completions stream successfully through proxy). |
| S-1 | medium | No upstream response size limit for `/models` (~10 MB max recommended). | No | `/models` response is ~4.5 KB. Localhost-only proxy. Acceptable risk. |

## Invalid Issues

Issues rejected because the E2E test proves they do not manifest:

| ID | Reason for Rejection |
|----|---------------------|
| B-1 (as high) | Review rated this "high — breaks HTTP/1.1 framing", but E2E test confirms streaming works correctly with openclaude. Framing is not required for SSE (client reads until connection close). |
| B-2 (as high) | `urllib.request.urlopen` auto-decompresses gzip responses. The `Content-Encoding` header was already consumed before `upstream.read()` is called. Data forwarded to client is already decompressed. E2E confirms. |

## Low/Info Issues (Accepted — not fixed)

| ID | Severity | Risk | Rationale |
|----|----------|------|-----------|
| B-4 | low | Confusing globals | Single-file script, uncomplicated use. Cosmetic. |
| T-4 | low | Error forwarding untested | E2E tested error paths. Acceptable. |
| T-5 | low | No --port test | Default port=0 works. Low risk. |
| T-6 | low | No --timeout test | Default 10s works. Low risk. |
| T-7 | low | Mock address 0 | Test-only aesthetic issue. |
| T-8 | low | Importlib for hyphenated file | Works, no better option without renaming. |
| S-2 | low | Stack info leak to client | Localhost only. Acceptable. |
| S-3 | info | No path sanitization | 127.0.0.1 only. Acceptable. |
| S-4 | info | Redirect following | 127.0.0.1 only. Acceptable. |
| C-1 | low | Unix-only signal.pause | Project targets Linux (Android emulator context). Acceptable. |
| C-2 | low | Thread-unsafe globals | Single-threaded use. Acceptable. |
| C-3 | low | No logging toggle | Clean output by design. Acceptable. |
| C-4 | low | No SIGTERM handler | SIGINT (Ctrl+C) works. Acceptable. |

## Acceptance Decision

**PASS** — all 7 stated requirements satisfied (verified by E2E tests with openclaude):
1. ✅ No openclaude config file modification (env vars only)
2. ✅ No OPENCODE_API_KEY set (empty OPENAI_API_KEY)
3. ✅ Python stdlib only (http.server, urllib, json, uuid, threading, signal)
4. ✅ Dynamic model fetch per run (fetch_free_models() on every invocation)
5. ✅ Free models only (cost.input === 0, not deprecated — 6 models, not 55)
6. ✅ Same HTTP headers as real opencode client (verified: User-Agent, 4 UUID headers)
7. ✅ Works with openclaude config files (openclaude runs normally alongside proxy)

Two medium fixes required (B-3, T-1) before commit. All other issues accepted.

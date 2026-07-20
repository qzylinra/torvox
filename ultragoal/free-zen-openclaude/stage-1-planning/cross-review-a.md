# Cross-Review: Plan A vs Plan B

Reviewer: Plan A (sub-agent writing the "Plan A" design doc).
Target: Plan B.

---

## 1. Where Plan B is Better (Plan A Should Adopt)

### 1a. `OPENAI_API_KEY=""` — explicit empty

Plan A omits this, claiming "some .env parsers could have issues." That's specious — we generate `export` statements for `eval`, not `.env` files. Plan B is correct: explicitly setting it to empty **protects against an accidentally-set existing `OPENAI_API_KEY`** in the user's shell. If a user already has `OPENAI_API_KEY=sk-...` in their environment from another project, Plan A's output leaves it in place and that key leaks to the zen gateway in the `Authorization` header. Plan B's `export OPENAI_API_KEY=` explicitly clears it.

**Adopt**: Include `OPENAI_API_KEY=""` in exports.

### 1b. `OPENAI_API_FORMAT=chat_completions`

Plan A doesn't set this. Plan B does. If openclaude's openai shim has an internal default that differs (or changes in a future release), Plan A breaks silently. Plan B is defensive and self-documenting.

**Adopt**: Include `OPENAI_API_FORMAT=chat_completions`.

### 1c. `--list` → stderr

Plan A outputs `--list` to stdout. Plan B outputs to stderr. Plan B's reasoning: stdout must remain clean for `eval "$(free-zen.py)"` — but `--list` and default mode are mutually exclusive args, so the practical concern is about accidental redirection. Still, Plan B's approach means `python free-zen.py --list > /tmp/somefile` won't produce an empty file and confuse the user. Stderr for informational output is the safer default in a script whose primary output format is shell exports.

**Adopt**: Send `--list` output to stderr.

### 1d. Richer model metadata

Plan A outputs only model IDs. Plan B extracts `context_window`, `max_output`, `reasoning`, and `vision` from models.dev and includes them in `--json` and `--list` output. This makes `--json` useful for external tooling and `--list` informative for human readers.

**Adopt**: Extract richer metadata.

### 1e. Test plan

Plan B has a thorough testing plan (unit + integration + e2e). Plan A has none. This is Plan A's biggest gap — a script that makes network calls and parses live JSON needs tests for error paths, edge cases, and mock responses.

**Adopt**: The full test plan structure from Plan B.

### 1f. Exit code 2 for usage errors

Plan B distinguishes exit code 1 (network/parse failure) from exit code 2 (`--launch` binary not found). Plan A uses exit 1 for everything. Following conventions (EXIT codes 1=error, 2=usage) is better for scripters.

**Adopt**: Use exit code 2 for usage/launch errors.

### 1g. Edge case documentation

Plan B's §12 is a comprehensive table of edge cases (null cost, structural changes, slow network, Python version compatibility). Plan A omits this entirely.

**Adopt**: Add an edge cases section.

### 1h. Error message prefix

Plan B prefixes error messages with `free-zen:`. Plan A uses bare `error:`. In a composite script environment, the prefix identifies the source.

**Adopt**: Use `free-zen:` prefix.

### 1i. `--no-color` flag

Useful for scripting and accessibility. Plan A doesn't have it.

**Adopt**: Add `--no-color`.

### 1j. Default timeout 10s (not 30s)

30s is long for a configuration tool. Plan B's 10s is more appropriate — if models.dev is that slow, the user probably has a network problem worth surfacing quickly.

**Adopt**: Default timeout 10s.

### 1k. Prefix-based default model selection

Plan A uses a hardcoded priority list of exact model IDs. Plan B uses prefix matching (`m["id"].startswith(prefix)`). This is more resilient when model IDs get version suffixes (e.g., `deepseek-v4-flash-free-v2`).

**Adopt**: Prefix-based matching.

### 1l. Python 3.8+ compatibility

Plan A targets 3.12+, Plan B 3.8+. No reason to require bleeding-edge Python for a script using only stdlib features available since 3.6. Plan B's target is more inclusive.

**Adopt**: Target 3.8+.

---

## 2. Where Plan B is Worse (Plan A Gets Right)

### 2a. `--launch` mode — unnecessary complexity

Plan B adds `--launch` which calls `os.execvpe` to replace the process with openclaude. This is a **convenience feature that violates the separation of concerns** that makes the script composable. The script's job is to **configure** — to generate environment configuration. Launching is openclaude's job.

- `--launch` duplicates functionality the user already has (`eval "$(free-zen.py)" && openclaude`)
- `os.execvpe` means the script cannot print any post-launch diagnostics
- It introduces exit code 2 and `--launch-cmd` complexity that serves no essential purpose

If a user wants a one-liner, they can write an alias or shell function. The script should not become a process manager.

**Keep Plan A's approach**: No `--launch`. Print exports, exit.

### 2b. `--probe` flag — missing from Plan B

Plan A's `--probe` is an optional diagnostic tool that hits `GET /v1/models` on the zen gateway to verify reachability and model ID consistency. This is valuable:
- When models.dev and the zen gateway are out of sync, it surfaces the discrepancy
- When debugging connectivity issues, it isolates whether the problem is models.dev or the zen endpoint

Plan B has no equivalent. Without it, the only way to debug endpoint issues is to run the script, then run openclaude, then interpret openclaude's error.

**Keep Plan A's approach**: Include `--probe` as an optional diagnostic.

### 2c. Shared UUID for session+project — fragile

Plan A generates 3 independent UUIDs (session, project, request). Plan B generates 2 UUIDs and shares one across session and project by slicing at different offsets (`uid[:26]` for session, `uid[26:52]` for project). This means the session and project values are tied to the same random seed — if `len(uid) < 52` (vanishingly unlikely with hex UUID, but possible with a hypothetical future UUID format change), the project value would be silently truncated.

Plan A's approach is simpler and more robust. Generating 3 UUIDs costs essentially nothing (3x `uuid.uuid4()` is ~3µs).

**Keep Plan A's approach**: 3 independent UUIDs.

### 2d. Usage comment in shell output

Plan A's `format_env_exports()` includes a comment line (`# Usage: eval $({__file__}) && openclaude`) in the generated shell output. Plan B's output is bare exports. The comment helps users who capture the output to a file (`python free-zen.py > env.sh`) — they can see where it came from and how to use it.

**Keep Plan A's approach**: Include a usage comment in the export output.

### 2e. `__file__` in help text

Plan A uses `__file__` to refer to itself in generated help text. Plan B hardcodes the filename. Renaming the script breaks Plan B's messaging but not Plan A's.

**Keep Plan A's approach**: Use `__file__` or `sys.argv[0]` for self-references.

---

## 3. Items Both Plans Miss

### 3a. Hardcoded fallback list as last resort

Both plans follow C1 ("dynamic fetch every run, no hardcoded models"). But a **hardcoded list of known-free model IDs** (not pricing or metadata, just IDs) as a fallback when models.dev is unreachable would be useful. Without it, the script is useless when offline — even though the user could still connect to the zen gateway with a known-good model ID. This doesn't violate C1 (C1 forbids hardcoded *pricing* to avoid stale data, not a model-ID-only fallback).

### 3b. Configurable base URL

Neither plan allows overriding the base URL (e.g., for self-hosted zen or testing against a staging endpoint). A `--base-url` flag would be useful for development and for users behind proxies.

### 3c. Warning when default model changes

If models.dev updates and `deepseek-v4-flash-free` is no longer free, Plan A/B silently picks the next best model. A warning like `free-zen: warning: preferred model 'deepseek-v4-flash-free' is no longer free, using 'nemotron-3-ultra-free'` would alert the user.

---

## 4. Summary Table

| Aspect | Plan A | Plan B | Verdict |
|--------|--------|--------|---------|
| `--launch` mode | ❌ | ✅ | **Plan A** — out of scope, violates separation of concerns |
| `OPENAI_API_KEY=""` | ❌ Omitted | ✅ Set to empty | **Plan B** — prevents key leakage |
| `OPENAI_API_FORMAT` | ❌ Omitted | ✅ Set | **Plan B** — defensive |
| `--list` destination | stdout | stderr | **Plan B** — safer for eval-centric design |
| `--probe` flag | ✅ | ❌ | **Plan A** — valuable diagnostic |
| Richer metadata | ❌ | ✅ | **Plan B** — more useful JSON/list output |
| Test plan | ❌ | ✅ | **Plan B** — critical gap in Plan A |
| Exit codes | Always 1 | 1, 2 | **Plan B** — more conventional |
| Edge cases | ❌ None | ✅ Comprehensive | **Plan B** |
| Error prefix | `error:` | `free-zen:` | **Plan B** |
| Default model selection | Hardcoded list | Prefix matching | **Plan B** — more resilient |
| Python target | 3.12+ | 3.8+ | **Plan B** — more compatible |
| UUID scheme | 3 independent | 2 shared | **Plan A** — simpler, more robust |
| Usage comment in exports | ✅ | ❌ | **Plan A** — helpful for users |
| `--no-color` | ❌ | ✅ | **Plan B** |
| Timeout default | 30s | 10s | **Plan B** |
| `--launch-cmd` | N/A | ✅ | Neutral (tied to `--launch`) |

---

## 5. Recommendations for Plan A

### Must adopt from Plan B

1. **`OPENAI_API_KEY=""`** — prevents accidental key leakage
2. **`OPENAI_API_FORMAT=chat_completions`** — defensive default
3. **`--list` → stderr** — keeps stdout clean for eval
4. **Richer metadata extraction** — context_window, max_output, reasoning, vision
5. **Full test plan** — unit tests with mocks, integration tests
6. **Exit code 2** for usage errors
7. **Edge cases section** in design doc
8. **`free-zen:` error prefix**
9. **Prefix-based default model matching**
10. **Python 3.8+ target**

### Keep from Plan A

1. **No `--launch`** — stay focused on configuration output
2. **`--probe` flag** — optional diagnostic is valuable
3. **3 independent UUIDs** — simpler, no edge-case concerns
4. **Usage comment in shell exports**
5. **`__file__` for self-reference**

### Add that both missed

1. Hardcoded model-ID-only fallback list (C1-compliant, IDs ≠ pricing)
2. `--base-url` flag for staging/testing
3. Warning on preferred-model-not-free-anymore

---

## 6. Overall Assessment

Plan B is the **stronger design** overall. It is more thoughtful about production concerns: error handling, edge cases, testability, compatibility, and defensive defaults. Plan A is simpler but has significant gaps — the biggest being no test plan and the `OPENAI_API_KEY` omission that could leak keys.

Plan A's strengths are its discipline around scope (no `--launch`, no process management) and the `--probe` diagnostic. These should be kept.

The merged design should be: **Plan A's architecture + Plan B's rigor** — Plan A's scope discipline and diagnostic features, with Plan B's defensive env vars, richer metadata, test plan, edge case handling, and compatibility choices.

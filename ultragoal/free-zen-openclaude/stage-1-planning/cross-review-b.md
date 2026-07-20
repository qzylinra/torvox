# Cross-Review: Plan B reviews Plan A

## Summary

Plan A is a solid, minimal script. Plan B is more opinionated and user-facing. The main philosophical difference: Plan A treats the script as a **utility that prints strings for eval**; Plan B treats it as a **launcher that manages the openclaude lifecycle** (hence `--launch`).

---

## 1. Points where Plan A differs — which is better?

| Dimension | Plan A | Plan B | Verdict |
|-----------|--------|--------|---------|
| `--launch` mode | ❌ Missing | ✅ `os.execvpe` | **Plan B** — this is the UX users actually want. `eval $(...) && openclaude` is one more step that can fail. |
| `OPENAI_API_KEY=""` | ❌ Omitted | ✅ Explicit empty string | **Plan B** — safer. If the user has `OPENAI_API_KEY` set globally (e.g., from their shell rc), omitting `export` means the old value leaks through, causing auth failures at the zen gateway (which expects empty auth). Plan A's rationale ("setting it could cause issues with .env parsers") is weak — the parser that matters is the shell's own `eval`, which handles `KEY=""` fine. |
| `OPENAI_API_FORMAT=chat_completions` | ❌ Missing | ✅ Set explicitly | **Plan B** — documents the correct format. Some openclaude versions may need this hint. Harmless to include. |
| `--list` output destination | stdout | stderr | **Plan B** — stdout must be clean for `eval`. Plan A's `--list` to stdout is fine there (it's an exclusive flag), but Plan A's default mode also uses stdout only. The real problem is that Plan A has no user guidance on stderr, so after `eval $(python free-zen.py)` the user sees nothing and may wonder if it worked. Plan B's stderr message ("found 6 free models…") confirms success. |
| `--probe` flag | ✅ Exists | ❌ Missing | **Plan A** — useful diagnostic. However, it's a niche debugging tool; the user can always run `curl https://opencode.ai/zen/v1/models` themselves. Low priority. |
| UUID generation | 3 separate `uuid.uuid4()` calls | 1 shared UUID + 1 extra for request | **Draw** — Plan A wastes entropy but is simpler to read. Plan B is more efficient but the slicing logic (`uid[26:52]`) is mildly clever and risks confusion. Neither matters for correctness. I slightly prefer Plan A's clarity here. |
| Default timeout | 30s | 10s | **Plan B** — for a CLI tool that's consumed synchronously in `eval`, 10s is enough. 30s makes the `eval` line feel hung. |
| `--list` output richness | Just model IDs | Table with ctx/max info | **Plan B** — far more useful for the user comparing models. |
| Model data structure | `dict[str, dict]` (raw) | `list[dict]` (normalized) | **Plan B** — the normalized list with `id`, `name`, `context_window`, `reasoning`, `vision` fields is more useful for JSON consumers and for the pretty table. |
| Error exit codes | 0 or 1 | 0, 1, or 2 | **Plan B** — distinguishing "openclaude not found" (exit 2) from "network error" (exit 1) is helpful for scripting. |

---

## 2. Potential issues in Plan A

1. **`OPENAI_API_KEY` omission is risky**: If the user's shell has `OPENAI_API_KEY` set (e.g., from a previous OpenAI project), it leaks into openclaude's requests. The zen gateway receives `Bearer <real-key>` instead of `Bearer `, which may cause different behavior or rate limiting. Setting it to `""` explicitly is the defensive choice.

2. **No user confirmation on default mode**: `python free-zen.py` prints env vars to stdout, but the user sees nothing after `eval` — no "it worked" feedback. Plan B sends a confirmation message to stderr.

3. **`--list` to stdout is a trap**: If a user accidentally pipes `--list` output into `eval`, they get shell errors (model IDs are not valid `export` statements). Plan B's stderr approach makes this impossible — `--list` never contaminates stdout.

4. **No `--launch` mode**: Users must invoke `eval "$(...)" && openclaude` manually. This is error-prone (forgot `eval`? forgot `&&`?). The whole point of the script is to make free-model usage frictionless.

5. **`--probe` adds complexity for marginal value**: It's a one-time debugging flag that most users will never use. Adds a separate HTTP request path, error handling branch, and a warning print path. The value (confirming the zen endpoint is up) can be obtained via a simple `curl` one-liner.

---

## 3. Things Plan A does better that Plan B should adopt

1. **`fetch_json()` factoring**: Plan A separates `fetch_json()` cleanly from the model-specific `fetch_models_dev_info()`. Plan B embeds URL construction inside `fetch_json` callers. Plan A's separation is cleaner.

2. **`FREE_ZEN_MODELS_DEV_URL` env var for testing**: Plan A's offline testing support via `$FREE_ZEN_MODELS_DEV_URL` is smart. Plan B should adopt this.

3. **`filter_free_models()` simplicity**: Plan A's filter is a flat `if cost.get("input", 0) != 0: continue`. Plan B wraps it in multiple `if not isinstance(...)` guards that may be overly defensive for a known API shape.

4. **UUID readability**: Plan A's 3 separate `uuid.uuid4()` calls are dead simple to read. Plan B's shared-UUID-with-offset trick requires a comment.

---

## 4. Things Plan A misses that Plan B gets right

1. **`--launch` mode**: The single biggest UX improvement. Plan A requires manual eval-and-openclaude; Plan B does it in one command.

2. **Explicit `OPENAI_API_KEY=""`**: Critical defense against leaked credentials.

3. **Explicit `OPENAI_API_FORMAT=chat_completions`**: Documents the expected API format for the zen gateway.

4. **`--list` output polishing**: Rich table with context/max tokens, "← default" marker, stderr output. Plan A's `--list` is bare model IDs only.

5. **User guidance on stderr**: Plan B tells the user what happened (how many models found, which one is default, how to run). Plan A is silent.

6. **`--launch-cmd` customization**: Useful for users with custom openclaude installations or wrappers.

7. **`--no-color` flag**: Considerate for scripting/CI environments.

8. **Exit code 2 for not-found**: Proper distinction between operational errors and configuration errors.

9. **Normalized model data format**: The `find_free_models()` → `list[dict]` with structured fields (`context_window`, `max_output`, `reasoning`, `vision`) is more useful for both `--json` output and the pretty table than Plan A's raw dict passthrough.

---

## 5. Recommendations for Plan B (self-critique)

| Issue | Fix |
|-------|-----|
| UUID offset slicing `uid[26:52]` is brittle if Python's `uuid.uuid4().hex` changes length (it won't, but clever code is harder to audit) | Use two UUIDs instead of the offset trick. Simpler is better. |
| `find_free_models()` defensive isinstance checks add noise | Collapse to Plan A's simpler `cost.get("input", 0) != 0` approach |
| No offline test mode | Adopt Plan A's `FREE_ZEN_MODELS_DEV_URL` env var pattern |
| No `--probe` equivalent (low priority) | Can add a `--check` flag that does a HEAD request instead of full GET, but not essential |

---

## 6. Recommendations for Plan A

1. **Add `--launch` mode** — it's the single most impactful UX improvement.
2. **Set `OPENAI_API_KEY=""` explicitly** — protects against leaked env vars.
3. **Set `OPENAI_API_FORMAT=chat_completions`** — documents the format for the zen gateway.
4. **Send `--list` to stderr** — keeps stdout eval-safe.
5. **Add user guidance on stderr** — "found N free models" confirmation.

---

## 7. Final Verdict

**Plan B is the stronger design**, primarily because of `--launch` mode, explicit env var defense (`OPENAI_API_KEY=""`, `OPENAI_API_FORMAT`), and the polished `--list` UX. Plan A's minimalism is elegant on paper but less practical in daily use.

Plan A wins on simplicity in UUID generation and filtering logic. Plan B should backport those simpler approaches (two UUIDs, flat `.get()` filtering).

If the goal is "a script engineers will actually use without thinking," Plan B wins. If the goal is "a script so simple it fits in a tweet," Plan A wins.

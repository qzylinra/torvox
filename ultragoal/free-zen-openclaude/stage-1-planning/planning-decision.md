# Planning Decision: free-zen.py

## Ruling

**Designate Plan B's agent** to write the integrated final plan. Plan B is the stronger design overall — better UX, defensive defaults, testability, and edge-case handling. Plan A's selected features are patched into Plan B.

## Resolution of Disagreements (3 rounds max)

| # | Issue | Plan A | Plan B | Ruling | Reasoning |
|---|-------|--------|--------|--------|-----------|
| 1 | `--launch` mode | ❌ Exclude (separation of concerns) | ✅ Include (best UX) | **Include** as optional flag | User said "no manual env var setup" — `--launch` directly addresses this. Add as `--launch` flag, not default. |
| 2 | `OPENAI_API_KEY=""` | ❌ Omit | ✅ Set explicitly | **Plan B wins** — set explicitly | Prevents accidental key leakage from shell env. Stronger defense. |
| 3 | `OPENAI_API_FORMAT` | ❌ Omit | ✅ Set | **Plan B wins** — set to `chat_completions` | Harmless, self-documenting, defensive against default changes. |
| 4 | `--list` destination | stdout | stderr | **Plan B wins** — stderr | Stdout must be clean for `eval`. This is the script's primary contract. |
| 5 | `--probe` flag | ✅ Include | ❌ Exclude | **Keep from Plan A** | Valuable one-shot diagnostic. Low code cost. |
| 6 | UUID scheme | 3 independent | 2 shared (offset) | **Keep from Plan A** — 3 independent | Simpler, no edge cases, 3µs cost is irrelevant. |
| 7 | Metadata richness | Minimal (IDs only) | Full (ctx, max, reasoning, vision) | **Plan B wins** — full metadata | Richer output for `--list`, `--json`, external tooling. |
| 8 | Test plan | None | Full (unit+integration) | **Plan B wins** — must include | Critical for a network-dependent script. Non-negotiable. |
| 9 | Exit codes | 0, 1 | 0, 1, 2 | **Plan B wins** — 3 codes | Conventional. 2 for usage/not-found. |
| 10 | Error prefix | `error:` | `free-zen:` | **Plan B wins** — prefixed | Identifies source in composite scripts. |
| 11 | Model selection | Exact ID list | Prefix matching | **Plan B wins** — prefix | Resilient to version suffixes. |
| 12 | Python target | 3.12+ | 3.8+ | **Plan B wins** — 3.8+ | All features exist in 3.8. No reason to require newer. |
| 13 | Usage comment in exports | ✅ | ❌ | **Keep from Plan A** | Helps users who capture output to file. |
| 14 | `FREE_ZEN_MODELS_DEV_URL` | ✅ | ❌ | **Keep from Plan A** | Enables offline testing — important for CI. |
| 15 | Timeout default | 30s | 10s | **Plan B wins** — 10s | Configuration tool should fail fast on network issues. |
| 16 | `--no-color` | ❌ | ✅ | **Plan B wins** — include | Basic scripting courtesy. |
| 17 | `--launch-cmd` | N/A | ✅ | **Include** with `--launch` | Required for custom paths / testing. |
| 18 | Python 3.10+ type hints | `dict`/`list` | `dict[str, str]` etc. | **Plan B's type hints** for documentation | Convenient for readers, no runtime cost. |

## Key Directives for the Integrated Plan

1. **Architecture**: Single-file, stdlib only, functions with docstrings
2. **Output modes**: Default (env exports), `--list`, `--json`, `--launch`, `--probe`
3. **Env vars**: `CLAUDE_CODE_USE_OPENAI=1`, `OPENAI_BASE_URL=...`, `OPENAI_API_KEY=`, `OPENAI_MODEL=...`, `OPENAI_API_FORMAT=chat_completions`
4. **Headers**: 3 independent UUIDs, fixed User-Agent, x-opencode-* fields
5. **Filtering**: `cost.input == 0`, `status != "deprecated"`, no hardcoded pricing
6. **Error handling**: Exit 1 for network/parse, exit 2 for usage/binary-not-found
7. **Testing**: `test_free_zen.py` with unit tests (unittest + mock), integration test script
8. **Diagnostics**: `--probe` hits `/v1/models` on zen, warns if selected model is missing
9. **Offline test support**: `FREE_ZEN_MODELS_DEV_URL` env var override
10. **Hardcoded model ID fallback**: Include a list of known-free model IDs (just IDs, no pricing) as last-resort fallback when models.dev is unreachable — with a loud warning on stderr

## Plan B Agent: Integration Task

Integrate both plans according to this ruling and write the final plan to `ultragoal/free-zen-openclaude/stage-1-planning/plan.md`.

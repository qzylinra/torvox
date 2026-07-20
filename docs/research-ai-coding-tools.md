# AI Coding CLI Tools — Research Report for Rust+wgpu+Kotlin Android Project

**Date:** 2026-07-20
**Context:** Selecting a CLI-based AI coding agent for the Torvox project (Rust + wgpu + Kotlin + Android/Gradle + Nushell scripts), with **DeepSeek V4 Flash** as the sole underlying model. Focus on architecture, code quality, and task capability — not pricing, speed, or ease of use.

---

## Table of Contents

1. [Evaluation Criteria](#evaluation-criteria)
2. [Tool Evaluations](#tool-evaluations)
   - [Claude Code (Anthropic)](#1-claude-code-anthropic)
   - [Codex CLI (OpenAI)](#2-codex-cli-openai)
   - [opencode](#3-opencode)
   - [openclaude](#4-openclaude)
   - [just-every/code](#5-just-everycode)
   - [claude-code-best/claude-code](#6-claude-code-bestclaude-code)
   - [Aider](#7-aider)
   - [Continue](#8-continue)
   - [Tabby](#9-tabby)
   - [Cody (Sourcegraph)](#10-cody-sourcegraph)
   - [Cursor](#11-cursor)
   - [Windsurf](#12-windsurf)
   - [Cline](#13-cline)
   - [Kilo Code](#14-kilo-code)
3. [Summary Comparison Table](#summary-comparison-table)
4. [Recommendation](#recommendation)

---

## Evaluation Criteria

For a **Rust + wgpu + Kotlin + Android (Gradle)** project with **DeepSeek V4 Flash** model:

| Criterion | Why It Matters |
|-----------|----------------|
| **Multi-language support** | Rust `#![no_std]`, Kotlin/Jetpack Compose, JNA FFI bridge, Gradle build files, Nushell scripts |
| **Shell/script execution** | Must run `cargo test`, `cargo clippy`, `./gradlew`, Nushell scripts as part of workflow |
| **Long-context, multi-file editing** | Coordinated changes across crate boundaries, bridge type sync, JNI |
| **Project-specific conventions** | AGENTS.md / CLAUDE.md / .clinerules — must follow strict rules (no_std, no unsafe, no anyhow, etc.) |
| **Architecture / agent orchestration** | Sub-agents, planning mode, autonomous multi-step task execution |
| **DeepSeek V4 Flash model support** | How well the tool integrates with non-native models (OpenAI-compatible API) |
| **Lint-after-edit** | Auto-run cargo clippy, spotlessCheck after edits to catch regressions |
| **Maturity / ecosystem** | Community size, plugin ecosystem, documentation depth |

---

## Tool Evaluations

### 1. Claude Code (Anthropic)

**Description:** Anthropic's official terminal-native coding agent. The original CLAUDE.md convention project. The most mature CLI coding agent with hooks, sub-agents, skills, and dynamic workflows.

- **Source:** https://docs.anthropic.com/en/docs/claude-code/overview
- **Source (dynamic workflows):** https://docs.anthropic.com/en/docs/claude-code/overview#run-agent-teams-and-build-custom-agents

**Key Architectural Strengths:**
- **CLAUDE.md convention** — the originator of project-specific agent instruction files. Supports hierarchical placement (repo root, subdirectory, `~/.claude/CLAUDE.md`). Auto-memory learns build commands and patterns across sessions.
- **Sub-agents / agent teams** — lead agent coordinates work, assigns subtasks, merges results. Multiple Claude Code agents can work on different parts of a task simultaneously.
- **Hooks system** — shell commands run before/after actions (e.g., auto-format after edit, lint before commit). This is the most mature hook system.
- **Dynamic workflows** — a workflow harness for orchestrating sub-agents at scale.
- **MCP support** — standard Model Context Protocol for tool/context extension.
- **Background agents** — long-running tasks on remote infrastructure.

**Weaknesses for This Project:**
- **Claude-only model** — Claude Code is hard-coded to Anthropic's models. It does NOT natively support DeepSeek or other third-party providers. Third-party provider support requires an AI gateway (OpenRouter, LiteLLM, etc.) that translates the Anthropic Messages API to OpenAI-compatible format — this adds a translation layer and may degrade tool-calling fidelity.
- **Closed source** — no way to modify internals if needed.
- **No LSP integration** — unlike opencode, does not natively load language servers for the LLM.

**DeepSeek V4 Flash Support:**
Poor natively. Requires an AI gateway like OpenRouter to translate Anthropic Messages API to OpenAI-compatible. The gateway must handle tool-call format translation, which can break complex agentic workflows. See: https://futureagi.com/blog/openai-codex-cli-multiple-model-providers-gateway-setup-2026

**AGENTS.md / CLAUDE.md Support:**
Excellent — originated the format. Supports hierarchical CLAUDE.md, auto-generated `/init`, auto-memory.

**Autonomous Multi-Step Capability:**
Excellent — sub-agents, dynamic workflows, `/goal`, hooks, background sessions.

**Verdict:** Best-in-class agent architecture, but crippled for this project because it cannot natively use DeepSeek V4 Flash. The gateway translation layer for tool calling is a reliability risk for complex Rust/Kotlin multi-file edits.

---

### 2. Codex CLI (OpenAI)

**Description:** OpenAI's lightweight coding agent (written in Rust, 96.6% Rust in the repo). Runs in the terminal. Uses OpenAI's Responses API by default.

- **Source:** https://github.com/openai/codex
- **Source (config):** https://developers.openai.com/codex/config-advanced

**Key Architectural Strengths:**
- **Written in Rust** — the tool itself is Rust. This means first-class understanding of Rust projects at the tool level.
- **Custom model providers** — supports `[model_providers.<id>]` in `config.toml` for third-party OpenAI-compatible APIs. Can point at any OpenAI-compatible endpoint.
- **AGENTS.md support** — reads `AGENTS.md` from project root and subdirectories. Repository-wide conventions.
- **Sandboxed execution** — network access disabled by default, multiple approval modes.
- **Multi-agent V2** — explicit thread caps, wait-time controls, root/subagent context hints.
- **Skills and plugins** — marketplaces for packaged instructions and tools.
- **Profile-based config** — `--profile` for different model configurations.

**Weaknesses for This Project:**
- **OpenAI-first design** — the Responses API is the default wire protocol. Non-OpenAI providers using Chat Completions API need `wire_api = "chat"` config and may not have full feature parity (tools, streaming, etc.).
- **No native LSP integration** — doesn't load language servers.
- **Multi-agent is less mature** than Claude Code's sub-agent system.
- **No hooks system** comparable to Claude Code.
- **DeepSeek compatibility requires explicit provider config** and may lose some tool-calling fidelity compared to native OpenAI models.

**DeepSeek V4 Flash Support:**
Good — Codex CLI supports custom model providers via `[model_providers.<id>]` blocks. DeepSeek can be configured as an OpenAI-compatible endpoint with `wire_api = "chat"`. However, Codex was designed for OpenAI's Responses API, and the Chat Completions adapter may not handle all tool-calling patterns identically. Source: https://developers.openai.com/codex/config-advanced and https://ofox.ai/blog/codex-cli-multi-provider-setup-via-config-toml

**AGENTS.md / CLAUDE.md Support:**
Good — reads AGENTS.md. Supports `AGENTS.md` as the project-level convention.

**Autonomous Multi-Step Capability:**
Good — MultiAgentV2, `/goal` (persisted goals with state), background sessions, codex exec.

**Verdict:** Viable. Rust-native tool, supports custom models via config, has multi-file editing and agent orchestration. Main risk: the Responses API → Chat Completions translation for non-OpenAI models may cause tool-calling issues on complex multi-step tasks.

---

### 3. opencode

**Description:** Open source AI coding agent (MIT license). Terminal + Desktop + IDE. Provider-agnostic — supports 75+ LLM providers. The current tool running this session.

- **Source:** https://opencode.ai
- **Source (docs):** https://opencode.ai/docs
- **Source (agents):** https://opencode.ai/docs/agents/
- **Source (rules):** https://opencode.ai/docs/rules/
- **Source (config):** https://opencode.ai/docs/config/
- **Source (GitHub):** https://github.com/anomalyco/opencode — 160K+ stars

**Key Architectural Strengths:**
- **Provider-agnostic** — natively supports any OpenAI-compatible API. This is a first-class feature, not an afterthought. DeepSeek V4 Flash works trivially by setting the provider to any OpenAI-compatible endpoint.
- **LSP integration** — automatically loads the right LSP servers for the LLM (gopls, typescript-language-server, rust-analyzer, etc.). This gives the model real-time diagnostics and code intelligence.
- **AGENTS.md native** — opencode created the AGENTS.md open standard (alongside agents.md community). Has `/init` to auto-generate project context. Supports hierarchical rules (project AGENTS.md, global `~/.config/opencode/AGENTS.md`, CLAUDE.md fallback).
- **Dual-agent system** — two primary agents (Build with full tools, Plan with read-only). Three sub-agents (General, Explore, Scout). Subagents can be manually invoked via `@mention`.
- **Custom agents** — define agents with specific models, prompts, and tool restrictions via `opencode.json` or `.opencode/agents/` markdown files.
- **Skills system** — reusable packaged workflows.
- **Context compaction with pruning** — auto-compact sessions when context is full, optional pruning to save tokens.
- **Plugin system** — extends with custom tools, hooks, integrations. Community has built 258+ sub-agents.
- **MCP support** — standard Model Context Protocol.
- **Open source (MIT)** — full control over the tool if needed.
- **Nushell-compatible** — can configure any shell, works with Nushell scripts.

**Weaknesses for This Project:**
- **Younger than Claude Code** — less battle-tested for extreme enterprise scenarios. The sub-agent system is less sophisticated than Claude Code's dynamic workflows.
- **Smaller ecosystem** — fewer community plugins/skills than Claude Code.
- **No background agent infrastructure** — no cloud-based background execution (unlike Claude Code's Routines or Codex Cloud).
- **Subagent depth configurable but limited** — default depth is 1 (primary → subagent only).

**DeepSeek V4 Flash Support:**
Excellent — opencode is provider-agnostic by design. DeepSeek is an OpenAI-compatible API, so it works natively. Configure via `/connect` or `opencode.json`. No gateway translation needed, no tool-calling degradation. 75+ LLM providers supported through Zen and direct API keys.

**AGENTS.md / CLAUDE.md Support:**
Excellent — native AGENTS.md, `/init` auto-generation, hierarchical resolution (project → global → CLAUDE.md fallback), custom instructions arrays in opencode.json, remote URLs, glob patterns for monorepo support.

**Autonomous Multi-Step Capability:**
Good — dual agents (Build/Plan), sub-agents (General/Explore/Scout), custom agents with specific models/tools, skills system for reusable workflows. Less mature than Claude Code's dynamic workflows but adequate for complex multi-step tasks. Community has built 258+ specialized sub-agents externally.

**Verdict:** Excellent fit for this project. Provider-agnostic architecture means DeepSeek V4 Flash works without translation layers or gateway hacks. LSP integration gives the model real-time diagnostics. AGENTS.md support is mature. Open source gives full control. The main trade-off is that the sub-agent orchestration is less sophisticated than Claude Code.

---

### 4. openclaude

**Description:** Open-source (MIT) fork of Claude Code that supports multiple LLM providers. Node.js-based (TypeScript, 99%). 30.2K stars.

- **Source:** https://github.com/Gitlawb/openclaude

**Key Architectural Strengths:**
- **Provider-agnostic** — supports OpenAI-compatible APIs, Gemini, GitHub Models, Codex OAuth, Ollama, etc. DeepSeek can be used via OpenAI-compatible endpoint.
- **Claude Code feature parity** — inherits Claude Code's architecture (bash tools, file tools, grep, glob, agents, tasks, MCP, web tools).
- **Agent routing** — route different agents to different models, per-agent provider/model overrides.
- **Background sessions** — `--bg` for long-running detached tasks.
- **Repo map** — structural codebase map (PageRank-ranked) auto-injected into context.
- **Web search/fetch** — built-in web search for non-Anthropic models via DuckDuckGo.

**Weaknesses for This Project:**
- **Node.js runtime overhead** — not as lean as Rust-based tools.
- **Claude Code derived** — the multi-provider support is retrofitted onto a Claude Code codebase, may have edge cases in tool-calling translation.
- **Smaller ecosystem** than Claude Code or opencode.
- **Security model less mature** — no hooks system.
- **VS Code extension is bundled** but the CLI is the primary interface.

**DeepSeek V4 Flash Support:**
Good — supports OpenAI-compatible providers natively. Uses `CLAUDE_CODE_USE_OPENAI=1` or `/provider` for setup. The underlying architecture was designed for Anthropic's Messages API, so OpenAI-compatible translation is done client-side.

**AGENTS.md / CLAUDE.md Support:**
Good — inherits Claude Code's AGENTS.md/CLAUDE.md support. Has its own `.openclaude-profile.json` for per-user config.

**Autonomous Multi-Step Capability:**
Good — inherits Claude Code's agent system (bash/file/grep/glob tools, MCP, sub-agents). Adds agent routing by model type.

**Verdict:** Viable. Strong provider support, Claude Code-class agentic capabilities. The main concern is that the multi-provider support is retrofitted onto a Claude-native architecture, which may create edge cases with tool-calling for non-Anthropic models.

---

### 5. just-every/code ("Every Code")

**Description:** Community fork of OpenAI's Codex CLI with added features: browser integration, multi-agents, theming, reasoning control, Auto Drive orchestration. Rust-based (96.8%). 3.8K stars.

- **Source:** https://github.com/just-every/code

**Key Architectural Strengths:**
- **Rust-native** — same core as Codex CLI, fast and efficient.
- **Multi-agent commands** — `/plan`, `/code`, `/solve` coordinate multiple agents.
- **Auto Drive** — multi-agent orchestration with self-healing.
- **Auto Review** — background ghost-commit watcher that reviews changes in a separate worktree.
- **Browser integration** — CDP support, headless browsing.
- **Theme system** — switch between accessible presets.
- **Safety modes** — read-only, approvals, workspace sandboxing.
- **MCP support** — extend with filesystem, DBs, APIs.
- **AGENTS.md/CLAUDE.md support** — reads both.

**Weaknesses for This Project:**
- **Small community** — 3.8K stars, much less battle-tested than alternatives.
- **Codex fork** — inherits Codex's OpenAI-first architecture; multi-provider support is less mature.
- **Rapidly changing** — 434 releases, alpha-level stability.
- **Limited documentation** compared to mainstream tools.

**DeepSeek V4 Flash Support:**
Good — inherits Codex CLI's custom model provider support. Can use any OpenAI-compatible API.

**AGENTS.md / CLAUDE.md Support:**
Good — supports both AGENTS.md and CLAUDE.md.

**Autonomous Multi-Step Capability:**
Excellent on paper — Auto Drive orchestration, multi-agent commands, Auto Review in parallel worktrees. But unproven at scale.

**Verdict:** Interesting but risky for a production project. The Auto Drive multi-agent system is ambitious but the tool's rapid release cadence (434 versions) and small community make it less reliable than established alternatives.

---

### 6. claude-code-best/claude-code (CCB)

**Description:** Chinese community-maintained fork of Claude Code that adds multi-provider support, Artifacts, Ultracode multi-agent orchestration, Goal system, and more. TypeScript (99.8%). 21.4K stars.

- **Source:** https://github.com/claude-code-best/claude-code

**Key Architectural Strengths:**
- **Full Claude Code compatibility** — all original features preserved (dynamic workflows, hooks, etc.).
- **Multi-provider support** — `/login` supports Anthropic, OpenAI, Gemini, Grok, and any compatible API. DeepSeek can be used through Anthropic Compatible or OpenAI Compatible paths.
- **Ultracode multi-agent orchestration** — injects workflow orchestration scripts with deterministic JS runtime for agent/pipeline/parallel/phase workflows.
- **Goal system** — `/goal` for persistent multi-step objectives with token budget.
- **Pipe IPC** — multi-instance collaboration across machines.
- **Remote Control** — self-hosted Docker for remote access.
- **Web Search** — built-in search tool (Bing/Brave).
- **ACP Protocol** — supports Zed, Cursor IDE integration.
- **Langfuse monitoring** — enterprise agent observability.
- **Poor Mode** — reduces concurrent requests for constrained models.

**Weaknesses for This Project:**
- **Chinese-language documentation** — primary docs are in Chinese; English README available but less comprehensive.
- **Node.js runtime** — TypeScript/Bun, not as lean as Rust-based tools.
- **Fork maintenance risk** — depends on community maintainers to keep up with upstream Claude Code changes.
- **Stability** — 802 commits, active development but less battle-tested than mainstream tools.
- **Security** — some features (Remote Control, Pipe IPC) may introduce attack surface.

**DeepSeek V4 Flash Support:**
Excellent — CCB explicitly supports DeepSeek through its custom provider system. The `/login` command supports "Anthropic Compatible" providers which can point at DeepSeek's API. Has explicit feature flags for provider customization.

**AGENTS.md / CLAUDE.md Support:**
Excellent — full Claude Code compatibility for CLAUDE.md, plus AGENTS.md support.

**Autonomous Multi-Step Capability:**
Excellent — Ultracode workflow engine, Goal system, Pipe IPC multi-instance orchestration, sub-agents.

**Verdict:** Technically impressive with the best multi-provider support of any Claude Code fork. The Ultracode orchestration and Goal system are genuinely innovative. However, the Chinese-language primary documentation and fork maintenance risk make it less suitable for a Western production team without Chinese language support.

---

### 7. Aider

**Description:** Python-based terminal AI pair programmer. 47.5K stars. The most established open-source AI coding tool.

- **Source:** https://github.com/Aider-AI/aider
- **Source (architecture):** https://deepwiki.com/Aider-AI/aider/2-core-architecture
- **Source (DeepSeek):** https://aider.chat/docs/llms/deepseek.html

**Key Architectural Strengths:**
- **Repository map** — tree-sitter-based codebase map with PageRank ranking. Understands symbol definitions and references across the entire codebase, not just open files.
- **100+ language support** — via tree-sitter grammars. Works with Rust, Kotlin, and most languages.
- **Multi-format editing** — search/replace blocks, whole-file rewrites, unified diffs, patch format, architect-delegated editing. Different models get tuned edit formats.
- **Model abstraction via litellm** — unified interface for 100+ models including DeepSeek, OpenAI, Anthropic, Gemini, local models via Ollama. This is first-class, not retrofitted.
- **Git-native workflow** — every edit is auto-committed with descriptive messages. Clean audit trail, `/undo`, `/diff`.
- **Lint-test-repair loop** — automatically lint and test after every change, fix problems detected by linters/tests.
- **Four chat modes** — code (direct edits), architect (plan-then-code), ask (questions), help.
- **Architect mode** — the LLM proposes a plan, then a different model (or the same) implements it. This is effectively a sub-agent pattern.

**Weaknesses for This Project:**
- **Python-based** — significant runtime overhead compared to Rust or TypeScript tools. Startup latency, memory usage.
- **No native sub-agents** — architect mode is the closest equivalent, but no parallel agent execution.
- **No MCP support** — doesn't implement Model Context Protocol for tool extension.
- **No LSP integration** — uses tree-sitter for code understanding, not real-time LSP diagnostics.
- **No hooks system** for pre/post-edit automation.
- **Single-process architecture** — no parallel execution, no background agents.
- **No file watcher** for automated workflows (though it has `--watch-files` for IDE integration).

**DeepSeek V4 Flash Support:**
Excellent — Aider supports DeepSeek as a first-class provider. `aider --model deepseek/deepseek-v4-flash` works natively through litellm. Source: https://aider.chat/docs/llms/deepseek.html and https://www.aimadetools.com/blog/deepseek-v4-aider-setup

**AGENTS.md / CLAUDE.md Support:**
Good — supports `.aider.conf.yml` for project config. Supports AGENTS.md conventions via `/read` or `--read` for loading project context. Has `--conventions-file` for coding standards. Not as deeply integrated as opencode or Claude Code but functional.

**Autonomous Multi-Step Capability:**
Moderate — architect mode (plan-then-implement), lint-test-repair loop, git auto-commit. No parallel sub-agents, no background agents, no workflow orchestration. Good for single-threaded multi-step tasks but not for parallel agent teams.

**Verdict:** Strong contender for code editing quality due to the repo map and multi-format editing architecture. Excellent DeepSeek support through litellm. The main limitations for this project are: (1) no sub-agents for parallel work, (2) Python overhead, (3) no MCP/LSP integration. Best suited for pair programming sessions rather than autonomous multi-step orchestration.

---

### 8. Continue

**Description:** Open-source coding agent (VS Code extension, CLI, JetBrains plugin). TypeScript (83.9%). 35K stars. **Project is no longer actively maintained (final 2.0.0 release).**

- **Source:** https://github.com/continuedev/continue

**Key Architectural Strengths:**
- **Multi-IDE support** — VS Code, JetBrains, CLI.
- **Model-agnostic** — supports any LLM provider.
- **Custom prompts** — automate key tasks with premade prompts.
- **Context-aware** — uses Sourcegraph-style context from both local and remote codebases.

**Weaknesses for This Project:**
- **NO LONGER ACTIVELY MAINTAINED** — the repo is read-only. No new features, no bug fixes. This is a disqualifying issue for a production project.
- **IDE-centric** — designed as an extension, not a terminal-native agent.
- **No sub-agents** — no multi-agent orchestration.
- **No MCP support** in the way newer tools implement it.

**DeepSeek V4 Flash Support:**
Good (when it was maintained) — model-agnostic architecture.

**AGENTS.md / CLAUDE.md Support:**
Moderate — has `.continue/config.json` for project config.

**Autonomous Multi-Step Capability:**
Poor — primarily a chat/completion tool, not an autonomous agent.

**Verdict:** Disqualified — project is read-only, no longer maintained.

---

### 9. Tabby

**Description:** Self-hosted AI coding assistant — a GitHub Copilot alternative. Rust-based (92.9%). 33.8K stars.

- **Source:** https://github.com/TabbyML/tabby

**Key Architectural Strengths:**
- **Self-hosted** — no cloud dependency, full data control.
- **Rust-native** — fast, efficient inference.
- **Consumer-grade GPU support** — can run on consumer hardware.
- **OpenAPI interface** — easy to integrate.
- **RAG-based code completion** — repository context for completions.
- **Answer Engine** — team knowledge base.

**Weaknesses for This Project:**
- **Not a coding agent** — Tabby is a code completion server (like GitHub Copilot), NOT an agentic coding tool. It doesn't edit files, run commands, or do multi-step tasks.
- **No multi-file editing** — provides completions and chat but doesn't make autonomous changes.
- **No sub-agents** — not designed for multi-step orchestration.
- **No CLI agent mode** — primarily a server with IDE extensions.
- **Doesn't match the evaluation criteria** — this is a fundamentally different category of tool.

**DeepSeek V4 Flash Support:**
Good — Tabby supports many models through its own model registry. Can be configured with custom models.

**AGENTS.md / CLAUDE.md Support:**
None — Tabby doesn't read project instruction files.

**Autonomous Multi-Step Capability:**
None — Tabby is a completion/chat server, not an agent.

**Verdict:** Wrong category — Tabby is a self-hosted Copilot alternative, not an agentic coding CLI. Not suitable for this project's requirements.

---

### 10. Cody (Sourcegraph)

**Description:** AI coding assistant from Sourcegraph. Available as VS Code, JetBrains, Visual Studio extensions, CLI, and Web app.

- **Source:** https://sourcegraph.com/cody

**Key Architectural Strengths:**
- **Sourcegraph Search integration** — context from both local and remote codebases using Sourcegraph's code search engine.
- **Multi-IDE support** — VS Code, JetBrains, Visual Studio, Web, CLI.
- **Context-aware** — full codebase context via Sourcegraph indexing.
- **Auto-edit** — suggests code changes based on cursor movements.

**Weaknesses for This Project:**
- **Sourcegraph dependency** — requires Sourcegraph instance for full context. Remote codebase context is the main selling point, not local agentic capabilities.
- **Not a terminal-native agent** — CLI exists but is less capable than the IDE extensions.
- **No sub-agents** — no multi-agent orchestration.
- **No MCP support** — limited extensibility.
- **No hooks system** for automation.
- **Enterprise-focused** — many features locked behind Sourcegraph Enterprise.

**DeepSeek V4 Flash Support:**
Moderate — supports latest LLMs but the model selection is managed through Cody's infrastructure, not fully user-controlled.

**AGENTS.md / CLAUDE.md Support:**
Limited — has its own configuration system but doesn't deeply integrate with AGENTS.md/CLAUDE.md conventions.

**Autonomous Multi-Step Capability:**
Poor — primarily a chat/completion tool. Cody CLI is a thin client for asking questions, not an autonomous agent.

**Verdict:** Poor fit — designed for Sourcegraph-powered codebase context, not terminal-native autonomous coding. No sub-agents, no multi-step orchestration. The CLI is limited compared to true coding agents.

---

### 11. Cursor

**Description:** AI-powered code editor (VS Code fork) by Anysphere. Agent mode via Composer. $2B+ ARR, 50K enterprise teams.

- **Source:** https://cursor.com
- **Source (Composer architecture):** https://buildfastwith.ai/cursor-composer-guide

**Key Architectural Strengths:**
- **Composer Agent mode** — multi-file editing with autonomous codebase navigation, terminal command execution, diff review.
- **Composer 1.5** — thinking model with adaptive depth, self-summarization for long tasks.
- **Excellent autocomplete** — "Tab Tab Tab" workflow predicting 3-5 lines ahead.
- **Custom model support** — can configure any OpenAI-compatible API as a model provider. DeepSeek V4 Flash works as a custom model.
- **Large user base** — most popular AI IDE, extensive community.
- **Agent mode in IDE** — autonomous multi-file editing with self-healing.

**Weaknesses for This Project:**
- **GUI IDE, not CLI** — Cursor is a full IDE, not a terminal-based CLI agent. Cannot be used in headless CI/CD pipelines or automated workflows.
- **Proprietary / closed source** — no control over the agent architecture.
- **Subscription cost** — $20+/month per developer.
- **No native Nushell support** — VS Code fork, terminal is secondary.
- **No hooks system** for pre/post-edit automation.
- **Less suitable for automated multi-step orchestration** — designed for interactive editing sessions, not autonomous agent teams.

**DeepSeek V4 Flash Support:**
Good — can be configured as a custom OpenAI-compatible model in Cursor Settings → Models → Add Custom Model. However, Cursor's custom model support is less feature-complete than native models.

**AGENTS.md / CLAUDE.md Support:**
Good — supports `.cursorrules` files. Also reads AGENTS.md.

**Autonomous Multi-Step Capability:**
Good for an IDE — Composer Agent mode handles multi-file edits and terminal commands. But not as sophisticated as Claude Code's sub-agents or opencode's agent system for fully autonomous task execution.

**Verdict:** Excellent IDE for interactive development with AI, but not suitable as a CLI-based autonomous coding agent for automated workflows. Cannot be scripted, has no headless mode, and is designed for interactive use. Better as a companion tool than the primary automation agent.

---

### 12. Windsurf

**Description:** AI-native IDE by Codeium (acquired by Cognition/Devin). Cascade agentic system. Free tier available.

- **Source:** https://windsurf.com
- **Source (architecture):** https://devstarsj.github.io/ai-tools/2026-04-10-Windsurf-AI-IDE-Complete-Guide-2026

**Key Architectural Strengths:**
- **Cascade agent** — flow-aware, multi-step, terminal-integrated autonomous agent.
- **Multi-file editing** — coordinated changes across entire repository.
- **Terminal integration** — run commands and observe output, auto-fix build errors.
- **Codebase awareness** — indexes project structure, imports, git diff awareness.
- **Proprietary SWE-1.5 model** — near-frontier performance at zero marginal cost.
- **Supercomplete** — intent-aware multi-line predictions.
- **Free tier** — 200 flow actions/month.

**Weaknesses for This Project:**
- **GUI IDE, not CLI** — like Cursor, Windsurf is a full IDE, not a CLI tool. Cannot be used headlessly.
- **Proprietary / closed source** — no control.
- **Acquisition risk** — acquired by Cognition (Devin). Roadmap uncertain. Engineering team that built Cascade is now at Google.
- **Model routing is managed** — Windsurf auto-selects models, less user control over model choice.
- **Limited custom model support** — primarily uses Windsurf-managed models, not fully user-configurable.
- **No native sub-agents** — Cascade is a single agent.
- **No hooks system** for automation.

**DeepSeek V4 Flash Support:**
Moderate — Windsurf uses managed model routing; custom OpenAI-compatible endpoints may work but are not first-class. Some users report configuration via OpenAI-compatible API relay.

**AGENTS.md / CLAUDE.md Support:**
Limited — has project-level config but no deep AGENTS.md/CLAUDE.md integration.

**Autonomous Multi-Step Capability:**
Good for an IDE — Cascade Flow mode handles multi-step tasks. But single-agent, no sub-agent orchestration.

**Verdict:** Like Cursor, unsuitable as a CLI-based autonomous agent. Excellent IDE, wrong category. Acquisition uncertainty makes it a platform risk.

---

### 13. Cline

**Description:** Open-source (MIT) VS Code extension for autonomous coding. VS Code-centric. Supports any model provider via custom API endpoints.

- **Source:** https://github.com/cline/cline
- **Source (Cline guide):** https://singularitymoments.com/cline-ai-code-assistant-guide-2026

**Key Architectural Strengths:**
- **Model-agnostic** — supports any OpenAI-compatible API, any provider. DeepSeek is explicitly recommended for cost-sensitive high-volume work.
- **MCP support** — GitHub MCP, Postgres MCP, etc. Extensible.
- **Browser automation** — Puppeteer integration.
- **Cheap with DeepSeek** — explicitly designed for BYO-model, works great with cheap providers.
- **`CLAUDE.md` and `.clinerules`** — project-specific conventions.
- **Transparent** — shows every action, every tool call.
- **Active open-source community** — MIT license.

**Weaknesses for This Project:**
- **VS Code extension** — requires VS Code or a VS Code fork (Cursor, Windsurf). Not a standalone CLI.
- **No headless mode** — cannot be used in CI/CD pipelines.
- **Not designed for multi-agent orchestration** — single agent with tool access.
- **No sub-agents** — no parallel task execution.
- **No hooks system** for pre/post-edit automation.
- **IDE-dependent** — cannot run in a terminal without VS Code.

**DeepSeek V4 Flash Support:**
Excellent — Cline explicitly recommends DeepSeek for cost-sensitive work. Configure via OpenAI-compatible provider settings. Known to work well.

**AGENTS.md / CLAUDE.md Support:**
Good — reads `.clinerules` and CLAUDE.md files. Supports project-specific conventions.

**Autonomous Multi-Step Capability:**
Moderate — shows every action, runs tools, makes edits. But single-agent, sequential execution. No sub-agent delegation.

**Verdict:** Strong for VS Code-integrated autonomous coding with DeepSeek, but limited by the VS Code dependency and lack of headless/CLI mode. Cannot serve as the primary CI/CD or terminal-based automation agent for this project.

---

### 14. Kilo Code

**Description:** Open-source AI coding agent (fork of opencode/Continue lineage). Supports 500+ models, VS Code extension + CLI. 3M+ developers.

- **Source:** https://kilo.ai

**Key Architectural Strengths:**
- **500+ model support** — extreme model flexibility.
- **Multiple modes** — Orchestrator, Architect, Debug, Code modes.
- **VS Code extension + CLI** — can be used in both modes.
- **Autocomplete** — competes with Cursor Tab and GitHub Copilot.
- **Cost optimizations** — 10-50x cost savings via model choice.

**Weaknesses for This Project:**
- **Less mature** — newer than opencode or Claude Code.
- **Smaller community** — fewer plugins, skills, and community resources.
- **Documentation less comprehensive** than established tools.
- **Mode-based architecture** — sequential mode switching, not true parallel sub-agents.

**DeepSeek V4 Flash Support:**
Excellent — supports DeepSeek as a first-class provider among 500+ models.

**AGENTS.md / CLAUDE.md Support:**
Good — inherits opencode's AGENTS.md support (Kilo is a fork).

**Autonomous Multi-Step Capability:**
Good — Orchestrator mode for multi-step tasks, Architect→Code mode flow. But less sophisticated than Claude Code's dynamic workflows.

**Verdict:** Promising but less established. The opencode lineage means solid architecture, but the smaller ecosystem and newer status make it riskier for a production project than either opencode or Aider.

---

## Summary Comparison Table

| Tool | Category | DeepSeek V4 Flash | AGENTS.md | Sub-Agents | LSP | CLI-Native | Open Source | Maturity |
|------|----------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| **Claude Code** | CLI Agent | ❌ Gateway | ✅✅ | ✅✅ | ❌ | ✅ | ❌ | 🏆 |
| **Codex CLI** | CLI Agent | ✅ Config | ✅ | ✅ | ❌ | ✅ | ✅ | ⭐⭐⭐ |
| **opencode** | CLI Agent | ✅✅ Native | ✅✅ | ✅ | ✅✅ | ✅ | ✅ MIT | ⭐⭐⭐ |
| **openclaude** | CLI Agent | ✅ Config | ✅ | ✅ | ❌ | ✅ | ✅ MIT | ⭐⭐ |
| **just-every/code** | CLI Agent | ✅ Config | ✅ | ✅ | ❌ | ✅ | ✅ Apache | ⭐ |
| **claude-code-best** | CLI Agent | ✅✅ Native | ✅ | ✅✅ | ❌ | ✅ | ✅ | ⭐⭐ |
| **Aider** | CLI Pair Prog | ✅✅ Native | ✅ | ❌ | ❌ | ✅ | ✅ Apache | ⭐⭐⭐ |
| **Continue** | IDE Agent | ✅ | ❌ | ❌ | ❌ | Partial | ✅ | ⛔ Abandoned |
| **Tabby** | Completion Svr | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ Apache | ⭐⭐ |
| **Cody** | IDE Assistant | ✅ | ❌ | ❌ | ❌ | Partial | ❌ | ⭐⭐ |
| **Cursor** | AI IDE | ✅ Custom | ✅ | ❌ | ✅ | ❌ | ❌ | ⭐⭐⭐ |
| **Windsurf** | AI IDE | ✅ Relay | ❌ | ❌ | ✅ | ❌ | ❌ | ⭐⭐ |
| **Cline** | VS Code Agent | ✅✅ | ✅ | ❌ | ✅ | ❌ | ✅ MIT | ⭐⭐⭐ |
| **Kilo Code** | IDE+CLI Agent | ✅✅ | ✅ | ❌ | ✅ | Partial | ✅ | ⭐⭐ |

**Legend:** ✅✅ = Excellent native support, ✅ = Good, ❌ = None/Poor

---

## Recommendation

### 🥇 First Choice: opencode

**Why opencode wins for this project:**

1. **DeepSeek V4 Flash works trivially** — opencode is provider-agnostic by design. DeepSeek as an OpenAI-compatible API is a first-class configuration, not a retrofitted hack. No gateway translation, no tool-calling degradation.

2. **AGENTS.md is native** — the project already has `AGENTS.md` with extensive rules (no_std, no unsafe in terminal-core, no anyhow, no abbreviations, etc.). opencode's `/init` auto-generates these. The rules system supports hierarchical resolution, glob patterns, and remote URLs — ideal for a complex monorepo.

3. **LSP integration** — opencode is the only CLI-native agent that natively loads LSP servers (rust-analyzer, etc.). This gives the model real-time diagnostics during edits, which is critical for Rust's strict type system and `#![no_std]` constraints.

4. **Open source (MIT)** — full code visibility and control. Can be modified if project-specific needs arise.

5. **Dual-agent system + sub-agents** — Build mode for coding, Plan mode for analysis, sub-agents for parallel exploration. Sufficient for multi-step tasks.

6. **Plugin ecosystem** — 258+ community sub-agents, skills system, MCP support.

7. **Multi-session** — multiple agents can work in parallel on the same project.

### 🥈 Second Choice: Aider

**Why Aider is second:**

1. **Excellent DeepSeek V4 Flash support** — first-class through litellm. This is actively tested and documented.

2. **Repo map with tree-sitter** — the codebase map with PageRank ranking is genuinely useful for understanding crate boundaries and dependencies. Works with Rust, Kotlin, and 100+ languages.

3. **Lint-test-repair loop** — auto-runs linters/tests after every edit and fixes failures. This maps directly to the project's "lint after every file change" requirement.

4. **Git-native workflow** — clean audit trail, easy to review/revert AI changes.

5. **Architect mode** — plan-then-implement pattern, useful for cross-crate coordination.

**Where Aider loses to opencode:** No sub-agents for parallel work, no LSP integration, Python runtime overhead, no MCP support, single-process architecture.

### Why not Claude Code

Claude Code has the best agent architecture (dynamic workflows, sub-agents, hooks) but it's **Claude-only**. Using DeepSeek V4 Flash requires an AI gateway that translates the Anthropic Messages API to OpenAI-compatible format. This translation layer is brittle for complex tool-calling workflows — exactly the multi-file edits with lint/test loops this project needs. The gateway risk outweighs the architectural advantage.

### Why not Codex CLI

Codex CLI comes closest architecturally (Rust-native, custom providers, agent orchestration). But it's OpenAI-first — the Responses API assumption means non-OpenAI providers need adapter configuration. DeepSeek via Chat Completions API may have edge cases in tool-calling fidelity. opencode's provider-agnostic design is cleaner.

### Summary

For a Rust+wgpu+Kotlin Android project **using DeepSeek V4 Flash**, the best tool is **opencode** due to its native provider-agnostic architecture, AGENTS.md support, LSP integration, and open-source flexibility. The second best is **Aider** for its repo map and lint-test-repair loop, though it lacks sub-agents and LSP.

If the model constraint were removed (i.e., Claude 4 Sonnet/Opus were available), **Claude Code** would be the top choice for its superior agent orchestration, but the DeepSeek compatibility requirement gives opencode the decisive edge.

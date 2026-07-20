# Roadmap: opencode-free-setup

## Goal

Create a lightweight, high-performance Node.js reverse proxy that:
1. Dynamically fetches free model IDs from `models.dev/api.json`
2. Acts as a standalone HTTP proxy to `opencode.ai/zen/v1`
3. Injects the **exact same HTTP headers** as pi-opencode-zen (`User-Agent: opencode/latest/1.3.15/cli`, `x-opencode-client`, `x-opencode-session`, `x-opencode-project`, `x-opencode-request`)
4. Intercepts `/v1/models` to filter to free models only
5. Does NOT require openclaude to be installed (standalone proxy)
6. Does NOT spawn any processes
7. Does NOT modify any profile files
8. Works with ANY OpenAI-compatible client

## Stages

### Stage 1-4: Complete (from previous rounds)
Planning, implementation, acceptance, and commit of the initial proxy script.

### Stage 5: Refinement — Standalone High-Performance Proxy
**Objective:** Transform the existing proxy to remove all openclaude dependencies, add performance optimizations, and ensure latest OpenAI protocol support.

**Requirements:**
- Remove `spawn()` and `node:child_process` dependency
- Remove `--bg` mode, signal handlers, and child process lifecycle management
- Proxy runs as a standalone HTTP server (process stays alive until killed)
- High performance: streaming, zero buffering, proper backpressure
- Latest OpenAI protocol: support `/v1/chat/completions`, `/v1/models`, streaming SSE
- All existing features preserved: header injection, model filtering, anti-detection, zero config file impact
- Auto-fetch free model list on each startup

**Acceptance:**
- Script has zero Node.js child_process dependency
- Script runs standalone: `node proxy.mjs`
- Any HTTP client can connect and receive completions
- Model list returns only free models
- Streaming SSE works correctly
- High performance verified (no buffering, direct piping)
- All previous requirements still met

# openclaude-zen-free

A tiny, **zero-dependency** local gateway that lets [OpenClaude](https://github.com/Gitlawb/openclaude)
use only the **free** [OpenCode Zen](https://opencode.ai/zen) models — no API key, no
payment, no `.env`, no `OPENCODE_API_KEY`.

It is a local HTTP proxy written in Go (standard library only). It is **not** a fork or
wrapper of OpenClaude and does not depend on OpenClaude at all. It speaks the same
OpenAI-compatible protocol that OpenClaude's OpenAI provider expects, and forwards only
`GET /v1/models` and `POST /v1/chat/completions` to the OpenCode Zen gateway — stripping
any client-supplied `Authorization` so requests go out keyless (free tier).

## What it does

- **Lists only free models.** `GET /v1/models` is intercepted and returns the current
  free model list, computed dynamically on every call (live Zen list ∩ free rule).
- **Enforces free-only.** Any chat request whose `model` is not in the free set is
  rejected with `400` before reaching the gateway — so a paid model can never be charged.
- **No key needed.** The client's `Authorization` is dropped; the gateway is hit
  keyless-free. OpenClaude therefore works without any API key.
- **Read-only / safe.** Binds to `127.0.0.1` only. Does not touch OpenClaude's config,
  so normal OpenClaude usage (with a real provider) keeps working independently.
- **Graceful.** If `models.dev` is unreachable, it falls back to the `-free` suffix rule
  and still serves the confirmed-free models.

## Build

```sh
go build -o openclaude-zen-free .
# or, fully static:
CGO_ENABLED=0 go build -o openclaude-zen-free .
```

Requires Go 1.24+. No external modules (`go.mod` has no `require`).

## Run

```sh
./openclaude-zen-free                 # listens on 127.0.0.1:8787
./openclaude-zen-free -listen 127.0.0.1:8799
./openclaude-zen-free -quiet          # silence access logs
```

## Point OpenClaude at it (verified)

Ready-to-use env file: `examples/openclaude.env`.

OpenClaude's built-in `custom` (OpenAI-compatible) route is used. It is `requiresAuth:false`,
performs model discovery via `GET {baseURL}/models`, and treats `127.0.0.1` as a local
provider so an **empty** API key passes validation. No config file edit and no `.env` are
needed — only environment variables.

### Build OpenClaude (one time)
```sh
git clone https://github.com/Gitlawb/openclaude
cd openclaude
bun install && bun run build   # produces dist/cli.mjs
```

### Run (proxy is a passive sidecar you start yourself)
```sh
# terminal A — start the proxy
./openclaude-zen-free

# terminal B — run OpenClaude against the proxy (isolated HOME, no config touched)
HOME=/tmp/ocz_home CLAUDE_CODE_USE_OPENAI=1 \
OPENAI_BASE_URL=http://127.0.0.1:8787/v1 \
OPENAI_MODEL=hy3-free \
OPENAI_API_KEY= \
node bin/openclaude -p "hello" --dangerously-skip-permissions --output-format text
```

Notes:
- `OPENAI_BASE_URL` keeps the `/v1` suffix (OpenClaude appends nothing extra), so discovery
  hits `http://127.0.0.1:8787/v1/models` and chat hits `.../v1/chat/completions`.
- `OPENAI_MODEL` is the raw model id (e.g. `hy3-free`) — no `provider/` prefix.
- The model list is fetched dynamically on each `openclaude` start (and via `/model`), and
  the proxy returns only free models. `OPENAI_MODEL` must be one of the free ids returned.
- To avoid touching any existing OpenClaude config, run with an isolated `HOME`, e.g.
  `HOME=/tmp/ocz_home ... node bin/openclaude ...`. Your normal OpenClaude settings are
  untouched because the proxy is registered purely via env vars.
- Do not set `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`, or model discovery is skipped.

This was verified end-to-end: OpenClaude listed the free models and completed a real
`hy3-free` chat through the proxy with no API key.

## Point OpenCode (`opencode`) at it (verified)

Ready-to-use config: `examples/opencode.json` (drop into your opencode config dir).

OpenCode's custom provider uses the `@ai-sdk/openai-compatible` package. Write a config
(here isolated via `XDG_CONFIG_HOME`) pointing `baseURL` at the proxy:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "zenfree": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "OpenCode Zen Free",
      "options": { "baseURL": "http://127.0.0.1:8787/v1", "apiKey": "sk-zen-free" },
      "models": {
        "hy3-free": {"name":"hy3-free"},
        "big-pickle": {"name":"big-pickle"},
        "deepseek-v4-flash-free": {"name":"deepseek-v4-flash-free"},
        "mimo-v2.5-free": {"name":"mimo-v2.5-free"},
        "nemotron-3-ultra-free": {"name":"nemotron-3-ultra-free"},
        "north-mini-code-free": {"name":"north-mini-code-free"}
      }
    }
  }
}
```

Then list models and run a one-shot chat:

```sh
XDG_CONFIG_HOME=/tmp/ocz_cfg opencode models zenfree          # lists the free models
XDG_CONFIG_HOME=/tmp/ocz_cfg opencode run "hello" -m zenfree/hy3-free
```

Verified: `opencode models zenfree` listed all six free models, and `opencode run` completed a
real `hy3-free` chat through the proxy with no real key (keyless upstream call).

## Point Codex (`code`) at it (verified)

Ready-to-use config: `examples/codex.toml` (copy to `$CODEX_HOME/config.toml`).

Codex reads `config.toml` from `$CODEX_HOME`. Register a custom `model_providers` entry:

```toml
model_provider = "zenfree"

[model_providers.zenfree]
name = "zenfree"
base_url = "http://127.0.0.1:8787/v1"
api_key = "sk-zen-free"
wire_api = "chat"
```

Then run non-interactively:

```sh
CODEX_HOME=/tmp/ocz_codex_home code exec -m zenfree/hy3-free "hello"
```

Verified: `code exec` reached the proxy and completed a real `hy3-free` chat, keyless.
The proxy strips the `zenfree/` provider prefix (Codex sends `zenfree/hy3-free`) so OpenCode
Zen receives the bare id. Any `provider/` prefix is handled transparently.

## Install

```sh
./install.sh        # builds and installs to ~/.local/bin/openclaude-zen-free
```

## Flags

| Flag          | Default                       | Purpose                          |
|---------------|-------------------------------|----------------------------------|
| `-listen`     | `127.0.0.1:8787`              | Listen address                   |
| `-upstream`   | `https://opencode.ai/zen/v1`  | OpenCode Zen gateway base        |
| `-models-dev` | `https://models.dev/api.json` | Cost-info source (non-fatal)     |
| `-quiet`      | `false`                       | Reduce access logging            |

## How it avoids detection / matches the client / protocol compatibility

Every request except `GET /v1/models` is forwarded **transparently** to the upstream
OpenCode Zen gateway — the proxy does not whitelist a fixed set of endpoints, so it works
with whatever the client sends (chat, streaming, tool calls, `responses`, `embeddings`,
file uploads, …). For each forwarded request the proxy:

- copies **all** incoming headers verbatim (so any current/p future protocol-version or
  client markers the real `opencode` CLI sends are preserved — the request stays on the
  latest Zen protocol), then
- strips only the credential (`Authorization`) and hop-by-hop headers, and
- overlays the OpenCode client identity (`User-Agent`, `x-opencode-client`, and per-request
  `x-opencode-session` / `-project` / `-request`), exactly mirroring the official
  `opencode` CLI when it uses Zen.

The request also goes keyless, exactly like `opencode` on Zen. Model ids are never
interpolated into the upstream URL; only the path is forwarded.

The only enforced rule is **free-only**: `GET /v1/models` is intercepted and returns only
the free models (fetched live from Zen each call, intersected with the free rule), and any
POST that carries a `model` field is rejected unless that model is free (the `provider/`
prefix some CLIs add, e.g. `zenfree/hy3-free`, is stripped before the check and before
forwarding). Everything else passes through untouched.

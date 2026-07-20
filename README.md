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

## How it avoids detection / matches the client

Requests forwarded upstream replicate the OpenCode CLI's request shape (User-Agent and
`x-opencode-*` headers) and go keyless-free, exactly like the official `opencode` client
when using Zen. Model ids are never interpolated into the upstream URL; only the path is
forwarded. Only `models` and `chat/completions` are handled — everything else returns 404.

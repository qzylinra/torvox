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

## Point OpenClaude at it

In your OpenClaude config, add an OpenAI-compatible provider whose `baseURL` is the proxy
and which lists a free model. Example (`config.json` / corresponding OpenClaude config):

```json
{
  "mcpServers": {},
  "customProviders": {
    "opencode-zen-free": {
      "baseURL": "http://127.0.0.1:8787/v1",
      "apiKey": "not-needed",
      "models": ["hy3-free"]
    }
  },
  "models": { "primary": "opencode-zen-free/hy3-free" }
}
```

`openclaude` will call `GET /v1/models` against the proxy (which returns only free models)
and can then chat with `hy3-free`. No `OPENCODE_API_KEY` or `.env` is required.

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

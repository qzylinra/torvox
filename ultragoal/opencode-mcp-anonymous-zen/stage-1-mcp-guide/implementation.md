# Implementation: Stage 1 — MCP Configuration Guide

## Quick Start

Create `opencode.json` in project root:

```json
{
  "mcp": {
    "my-server": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "."]
    }
  }
}
```

## Common Scenarios

### Add a Local MCP

```json
{
  "mcp": {
    "database-tools": {
      "type": "local",
      "command": ["node", "mcp-server.js"],
      "cwd": "./tools",
      "environment": { "DB_URL": "{env:DATABASE_URL}" },
      "timeout": 10000
    }
  }
}
```

### Connect to a Remote MCP

```json
{
  "mcp": {
    "sentry": {
      "type": "remote",
      "url": "https://mcp.sentry.dev/sse"
    }
  }
}
```
Run `opencode mcp auth sentry` to authenticate via OAuth.

### Override Team Defaults

Remote config (`.well-known/opencode`) provides org defaults. Override locally:

```json
{
  "mcp": {
    "team-logger": { "enabled": false }
  }
}
```

### Restrict Per Agent

Globally disable, per-agent enable:

```json
{
  "tools": { "database*": false },
  "agent": {
    "data-analyst": { "tools": { "database*": true } }
  }
}
```

Tool naming: `servername_toolname`.

### Use Secrets

```json
{ "environment": { "API_KEY": "{env:MY_API_KEY}" } }
```

Or from file: `"{file:~/.secrets/key}"`.

## Reference

### Config Sources (merged, later wins)

1. Remote `.well-known/opencode`
2. Global `~/.config/opencode/opencode.json`
3. `$OPENCODE_CONFIG`
4. Project `./opencode.json`
5. `.opencode/` agents/commands/plugins
6. `$OPENCODE_CONFIG_CONTENT`
7. macOS managed `/Library/Application Support/opencode/`
8. macOS MDM `.mobileconfig`

### Local MCP Options

| Option | Type | Description |
|--------|------|-------------|
| type | "local" | Required |
| command | string[] | Required |
| cwd | string | Working directory |
| environment | object | Env vars |
| enabled | boolean | Default true |
| timeout | number | Tool fetch ms (default 5000) |

### Remote MCP Options

| Option | Type | Description |
|--------|------|-------------|
| type | "remote" | Required |
| url | string | Required |
| headers | object | HTTP headers |
| oauth | object|false | OAuth config or disabled |
| enabled | boolean | Default true |
| timeout | number | Default 5000 |

### OAuth Commands

- `opencode mcp list` — List configured MCPs
- `opencode mcp auth <name>` — Authenticate
- `opencode mcp logout <name>` — Log out
- `opencode mcp debug <name>` — Debug info

## Pitfalls

- **Context bloat**: Each MCP registers tools. Enable only what's needed.
- **Naming**: MCP server names must not conflict. Use descriptive names.
- **OAuth tokens**: Stored in `~/.local/share/opencode/mcp-auth.json`.
- **Timeout**: Remote MCPs may need higher `timeout` for initial connection.

# Exploration: MCP Server Configuration in OpenCode

## Conclusion

OpenCode uses `opencode.json` (JSON/JSONC) with a top-level `mcp` key to configure MCP servers. Two types supported: `local` (stdio) and `remote` (HTTP). Config files merge across multiple locations with clear precedence. Variable substitution (`{env:VAR}`) and per-agent tool management via glob patterns are supported.

## Source

https://opencode.ai/docs/mcp-servers/ — Official MCP servers doc
https://opencode.ai/docs/config/ — Official config doc

## Config File Format

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "server-name": {
      "type": "local",       // or "remote"
      "command": ["npx", "-y", "my-mcp"],
      "enabled": true,
      "environment": {
        "MY_VAR": "value"
      },
      "cwd": "/path/to/workdir",
      "timeout": 5000
    }
  }
}
```

## Local MCP Servers

```json
{
  "mcp": {
    "my-local-mcp": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-everything"]
    }
  }
}
```

Options:
- `type` (required): `"local"`
- `command` (required): Array of command + args
- `cwd`: Working directory (relative paths resolve from workspace)
- `environment`: Object of env vars
- `enabled`: Boolean toggle
- `timeout`: Tool fetch timeout in ms (default 5000)

## Remote MCP Servers

```json
{
  "mcp": {
    "my-remote-mcp": {
      "type": "remote",
      "url": "https://my-mcp-server.com",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer {env:MY_API_KEY}"
      },
      "oauth": {
        "clientId": "{env:CLIENT_ID}",
        "clientSecret": "{env:CLIENT_SECRET}",
        "scope": "tools:read tools:execute"
      }
    }
  }
}
```

Options:
- `type` (required): `"remote"`
- `url` (required): MCP server URL
- `enabled`: Boolean toggle
- `headers`: HTTP headers
- `oauth`: OAuth config (or `false` to disable)
- `timeout`: Tool fetch timeout (default 5000)

## OAuth

- Automatic: OpenCode detects 401 and initiates OAuth flow
- Supports Dynamic Client Registration (RFC 7591)
- Manual auth via `opencode mcp auth <server-name>`
- Tokens stored in `~/.local/share/opencode/mcp-auth.json`
- Commands: `opencode mcp list`, `opencode mcp logout`, `opencode mcp debug`

## Config File Locations (Precedence: later overrides earlier)

1. Remote config (`.well-known/opencode`) — organizational defaults
2. Global config (`~/.config/opencode/opencode.json`) — user preferences
3. Custom config (`OPENCODE_CONFIG` env var)
4. Project config (`opencode.json` in project root)
5. `.opencode` directories — agents, commands, plugins
6. Inline config (`OPENCODE_CONFIG_CONTENT` env var)
7. Managed config files (`/Library/Application Support/opencode/` on macOS)
8. macOS managed preferences (`.mobileconfig` via MDM)

All configs are **merged**, not replaced. Later configs override earlier ones for conflicting keys only.

## Variable Substitution

- `{env:VARIABLE_NAME}` — environment variables (empty string if unset)
- `{file:path/to/file}` — file contents (relative to config file, or absolute)

```json
{
  "provider": {
    "anthropic": {
      "options": {
        "apiKey": "{file:~/.secrets/anthropic-key}"
      }
    }
  }
}
```

## Per-Agent & Global Tool Management

Globally disable, per-agent enable:
```json
{
  "mcp": {
    "my-mcp": { "type": "local", "command": ["bun", "x", "my-mcp-command"] }
  },
  "tools": { "my-mcp*": false },
  "agent": {
    "my-agent": {
      "tools": { "my-mcp*": true }
    }
  }
}
```

MCP tools are registered with server name as prefix: `servername_toolname`.

## Enabling/Disabling

- Set `"enabled": false` in the MCP config to disable
- Or use `"tools": { "my-mcp*": false }` globally/`true` per-agent
- Organization remote defaults can be overridden locally by matching server name + `"enabled": true`

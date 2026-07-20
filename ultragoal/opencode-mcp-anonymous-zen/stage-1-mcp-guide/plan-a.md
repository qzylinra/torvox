# Plan A: MCP Configuration Guide

## Conclusion

A reference-style guide organized from minimal example to depth, covering all 8 config precedence levels, both MCP types, OAuth, variable substitution, and per-agent tool gating.

## Structure

1. **Quick Start** — Minimal `opencode.json` with one local MCP server
2. **Config File Basics** — Location, format, schema, merging behavior
3. **Local MCP** — `type: "local"`, command array, environment, cwd, timeout, enabled
4. **Remote MCP** — `type: "remote"`, url, headers, oauth
5. **OAuth** — Auto-detection, dynamic registration, manual auth, CLI commands
6. **Config Precedence** — 8 levels in order, merged not replaced
7. **Variable Substitution** — `{env:VAR}`, `{file:path}`
8. **Tool Management** — Global disable, per-agent enable, glob patterns, naming convention
9. **Examples** — Sentry, Context7, Grep, Filesystem, GitHub, PostgreSQL, custom local

## Examples

- Sentry MCP (remote + OAuth): `opencode mcp auth sentry`
- Context7 (remote + optional API key header): `use context7`
- Grep by Vercel: `use the gh_grep tool`
- Filesystem: local npx server
- Per-agent: disable globally, enable per agent

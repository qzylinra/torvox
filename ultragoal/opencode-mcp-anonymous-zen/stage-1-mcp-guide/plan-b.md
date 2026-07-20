# Plan B: MCP Configuration Guide

## Conclusion

A problem-oriented guide organized by use case, prioritizing real-world scenarios over reference structure.

## Structure

1. **Use Case: "I want to add an MCP server to OpenCode"** — Minimal working example
2. **Use Case: "I have a local Python tool I want to expose as MCP"** — Local type, command array, cwd, environment
3. **Use Case: "I want to connect to a hosted MCP service"** — Remote type, url, headers, OAuth flow
4. **Use Case: "My team provides default MCPs, I want to enable some"** — Remote config override, enabled toggle
5. **Use Case: "I only want certain agents to use certain MCPs"** — Per-agent tool gating with globs
6. **Use Case: "My config has too many MCPs, performance is bad"** — Selective enabling, timeout config, context warning
7. **Pitfalls** — Context bloat from MCPs, OAuth token storage location, MCP server name prefix convention
8. **Reference** — Full options table for local and remote types

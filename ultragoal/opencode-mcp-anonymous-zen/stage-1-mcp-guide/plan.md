# Final Plan: Stage 1 — MCP Configuration Guide

## Structure

1. **Quick Start** — Minimal `opencode.json` with one local MCP server
2. **Common Scenarios** (use-case driven):
   - "Add a local MCP" — local type, command, cwd
   - "Connect to a hosted MCP" — remote type, url, headers, OAuth
   - "Override team defaults" — multi-level config precedence
   - "Restrict MCPs per agent" — per-agent tool gating
   - "Use secrets without hardcoding" — {env:VAR} substitution
   - "Troubleshoot MCP issues" — performance, context bloat
3. **Reference**:
   - Config file locations & precedence (8 levels)
   - Local MCP full options table
   - Remote MCP full options table (url, headers, oauth)
   - Variable substitution: {env:X}, {file:path}
   - Tool management: glob patterns, naming convention, per-agent
4. **Pitfalls** — Context bloat, MCP server naming, OAuth token storage

## Implementation

Write `ultragoal/opencode-mcp-anonymous-zen/stage-1-mcp-guide/implementation.md` with the complete guide content. Verify against exploration-mcp.md for accuracy.

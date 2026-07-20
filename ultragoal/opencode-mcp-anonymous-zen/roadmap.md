# Roadmap: OpenCode MCP and Zen Anonymous Mode

## Goal Summary

Provide complete, actionable guidance on:
1. Configuring MCP servers in OpenCode via `opencode.json`
2. Using OpenCode Zen free models without an API key (anonymous/public mode)

## Stages

### Stage 1: Compile MCP Configuration Guide

**Objective**: Produce a definitive reference for configuring both local and remote MCP servers in OpenCode.

**Acceptance Criteria**:
- Covers local MCP syntax (type, command, environment, cwd, timeout, enabled)
- Covers remote MCP syntax (type, url, headers, oauth)
- Documents config file locations and precedence (8 levels)
- Documents per-agent and global tool management with glob patterns
- Documents variable substitution ({env:X}, {file:path})
- Includes working examples

### Stage 2: Compile Zen Anonymous Mode Guide

**Objective**: Produce a definitive reference for using OpenCode Zen free models without API keys.

**Acceptance Criteria**:
- Explains anonymous/public mode concept
- Documents 4 methods to enable (TUI, env var, config, npx wizard)
- Lists all current free models with their model IDs
- Explains OpenCode Zen endpoints and model ID format
- Notes privacy implications of free models
- References pi-opencode-zen implementation pattern

### Stage 3: Final Integration and Verification

**Objective**: Combine both guides into a cohesive final answer, verify completeness against source docs, and resolve any gaps.

**Acceptance Criteria**:
- Combined answer covers all acceptance criteria from stages 1 and 2
- No contradictions between the two guides
- Cross-references between MCP and Zen config where applicable
- Verified against official docs at opencode.ai

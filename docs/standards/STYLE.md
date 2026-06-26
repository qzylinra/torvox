# Style Guide

## Shell Scripts

All shell scripts use Nushell (`.nu`). No bash or sh.

- Shebang: `#!/usr/bin/env -S nix develop --command nu`
- Check external command exit codes
- `snake_case` naming
- `$env.VAR = "val"` for environment variables
- `^command` for external commands
- Stderr: `e>|` for stderr-only redirect, `out+err>` for combined
- No `$args` at module level (breaks `source`-based syntax check)

## Nix

All environment management via Nix. No system shell builds.

- Always: `nix develop`, `nix develop --command cargo build`, `nix fmt`
- No abbreviated variable names
- ShellHook is the primary mechanism; checks and formatter defined in flake.nix

## GitHub Actions

- Action versions: default branch (`@main` or `@master`), not tags
- No step `name`
- Merge adjacent `run` steps into multi-line blocks
- `||` only for explicit error handling, never for error swallowing
- kebab-case job naming
- Always declare `permissions:`

## General

- No abbreviated variable names
- Inline intermediate variables when possible
- One document per topic, no duplication

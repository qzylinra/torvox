# Project Brief — Torvox

## Mission

Build a GPU-accelerated Android terminal emulator that rivals desktop-class terminal performance by leveraging Vulkan (via wgpu) for rendering, Ghostty's battle-tested VT parser for escape sequence handling, and a Kotlin + Compose UI.

## Core Goals

| Goal | Description |
|------|-------------|
| GPU-accelerated rendering | No CPU software fallback — wgpu/Vulkan only. Mesa Lavapipe for headless Linux, SwiftShader for Android emulator guest GPU. |
| Full VT5xx+ compliance | Vendored Ghostty parser handles the complete VT escape sequence specification including scrollback, SGR, DEC modes, Kitty keyboard protocol, and OSC sequences. |
| Low-latency input | Separate PTY reader and input writer threads. Kitty keyboard protocol for precise modifier encoding. IME pixel-stable layout (keyboard open/close does not shift content). |
| Android-first | Kotlin + Compose UI, JNA bridge for Kotlin→Rust calls, boltffi for Rust→Kotlin serialization. package name `com.termux` (intentional, for Termux add-on compatibility). |
| AI agent integration | MCP server over Unix socket exposes terminal state to AI coding agents via JSON-RPC 2.0. |

## Key Requirements

- `terminal-core` is `#![no_std]` with `#![forbid(unsafe_code)]` — zero unsafe in the data model crate
- `unsafe` isolated to `pty.rs` (fork/exec) and `gui-android` FFI only, each with safety comments
- Strict one-way crate dependency chain: `libghostty-vt ← terminal-core ← terminal-engine ← gpu-renderer ← android-gui ← android/app`
- All Rust tests pass, zero clippy warnings, zero fmt violations, zero new `unsafe` in core
- Golden images banned — use OCR (rapidocr) or pixel-coordinate assertions instead
- Font files banned — fonts come from Nix store, not bundled in git
- Nushell only — no bash/sh scripts
- No `anyhow` in library crates — use `thiserror 2`

## Out of Scope

- Java files (Kotlin only on Android side)
- `portable-pty`, `bincode`, `rust-android-gradle`
- CPU/Canvas rendering fallback (GPU-only)
- Desktop builds (Android-only deployment target)

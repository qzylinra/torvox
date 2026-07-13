# Product Context — Torvox

## Target Users

- **Android developers** who need a fast, reliable terminal on device
- **Termux users** who want GPU-accelerated rendering instead of CPU-based software rendering
- **SSH/Mosh users** who connect to remote servers from Android
- **AI-assisted developers** who use coding agents that can read terminal state via MCP

## User Experience Goals

| Goal | Implementation |
|------|---------------|
| IME stability | Keyboard open/close must NOT change terminal rows, columns, cell size, or font size. `windowSoftInputMode="adjustNothing"` on main activity. |
| Smooth scrolling | GPU-accelerated via wgpu — no jank, no tearing. Separate render thread with Vulkan synchronization. |
| Fast font rendering | cosmic-text for shaping, swash for rasterization, guillotiere atlas packing, wgpu atlas texture upload. R8Unorm atlas format. |
| Reliable clipboard | OSC 52 read/write for terminal-controlled clipboard operations. |
| Search | Full terminal content search via `getTerminalText()` (not per-line `scrollbackLine()`). |
| Multiple sessions | Tab-based session management with GPU surface lifecycle correctly handled on session switch. |

## Design Philosophy

- **GPU-accelerated everything**: The terminal grid is rendered entirely on GPU. No `Canvas.drawText` per cell. This is the key differentiator from CPU-based terminal emulators.
- **Deterministic builds**: Nix for reproducibility. `flake.nix` pins all toolchain dependencies. No system-installed SDKs.
- **Test closest to source**: Rust-side state verification (sub-ms, deterministic) preferred over Android-side screenshot analysis (slow, flaky). Pixel-level tests only for integration/E2E.
- **Structured workflow**: Memory-bank lesson files capture every hard-won lesson. No bug is fixed without documenting the root cause.

## Current Pain Points Addressed

| Pain Point | Solution |
|------------|----------|
| CPU-based rendering is slow on Android | wgpu/Vulkan with GPU-only render path |
| Keyboard destroys terminal layout | `adjustNothing` + `contentWindowInsets = WindowInsets(0.dp)` + `imePadding()` on content column |
| Surface lifecycle breaks render thread | Generation counter, pause/resume rendering on surface destroy/recreate |
| Ghostty dynamic linking on Android | build.rs with patchelf SONAME strip + bundle libc++_shared.so |

# System Patterns — Torvox

## Crate Dependency Chain (Strict One-Way)

```
libghostty-vt / libghostty-vt-sys    ← Ghostty VT parser (vendored Zig)
    ↑
terminal-core (no_std, serde + unicode-width)  ← Data model, Grid, Cell, Event
    ↑
terminal-engine (libghostty-vt + nix + flume)  ← PTY, VT parse, Session
    ↑
gpu-renderer (wgpu + cosmic-text + swash + guillotiere)  ← GPU render
    ↑
android-gui (boltffi + JNA)  ← Kotlin↔Rust bridge
    ↑
android/app (Kotlin + Compose)  ← Android UI
```

Each crate depends only on the crate directly below it. Violations break the build. Verified by `cargo metadata --no-deps --format-version 1`.

## Architecture Patterns

### 1. Pipeline Architecture

The render path is a fixed sequence of stages:

```
PTY → flume → GhosttyTerminal → DirtyMask → RenderThread
  → cosmic-text shape + swash glyph rasterize → guillotiere pack
  → wgpu atlas upload → Instance[]   → wgpu render_frame → TextureView
```

Each stage receives input from the previous stage and produces output for the next. No back-edges in the data flow.

### 2. Two-Way Bridge (boltffi + JNA)

| Direction | Mechanism | Data |
|-----------|-----------|------|
| Rust → Kotlin | boltffi binary wire format (position-sensitive, no length prefix/checksum) | Grid snapshots, terminal events |
| Kotlin → Rust | JNA reflection-based calls | Keyboard input, resize, session management |

**Discipline**: Bridge types must be sync'ed manually between `bridge.rs` and `TorvoxBridge.kt`. Field order and count must match exactly.

### 3. Thread Model (6-7 threads per session)

| Thread | Purpose |
|--------|---------|
| PTY Reader | Polls PTY with `poll()` (100ms timeout), feeds GhosttyTerminal |
| Input Writer | Writes keyboard input to PTY master (separate from reader) |
| Process Waiter | `waitpid()` on child process |
| RenderThread | CountDownLatch-woken loop: shape, rasterize, submit GPU frame |
| MCP Listener | Accepts Unix socket connections (per-server, not per-session) |
| MCP Worker | Per-connection handler |

### 4. State Management

- **Grid state**: `GhosttyTerminal` owns the terminal grid. `GridSnapshot` is produced on demand.
- **Dirty tracking**: `DirtyMask { partitions: Vec<u64> }` identifies changed rows.
- **Session state**: `Session` struct owns PTY, terminal, OSC handler, and output channel.
- **Android UI state**: ViewModel + Compose state hoisting. `imeBottomPadding` state for IME layout.

### 5. Error Handling

- No `anyhow` in library crates — use `thiserror 2` with `std` feature gate
- `terminal-core` uses `#![no_std]` — Error impls only with `std` feature enabled
- Render thread exits after 100 consecutive errors (~10s), must be restarted via generation counter
- `Option` for graceful fallbacks (e.g., headless Vulkan detection)

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| GPU-only (Vulkan via wgpu) | Single backend to maintain; Vulkan provides explicit GPU control needed for low-latency terminal rendering |
| Ghostty VT parser | Battle-tested, full VT5xx+ support, avoids reimplementing decades of escape sequence behavior |
| TextureView over SurfaceView | No `setZOrderOnTop` needed; integrates naturally with Compose |
| Dynamic linking for Ghostty | Zig install archive only provides `.o` files, not `.a` — static linking impossible |
| `adjustNothing` for IME | Prevents Android from resizing the activity when keyboard opens/closes, keeping terminal layout pixel-stable |
| `memory-bank/lessons/` | Each lesson captures a root-cause analysis: problem → root cause → fix → lesson. Prevents repeated debugging of the same issue. |

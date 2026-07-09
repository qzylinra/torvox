# Torvox Architecture Guide

## Overview

Torvox is a GPU-accelerated Android terminal emulator. The entire terminal engine (VT parsing, PTY management, grid data model, wgpu rendering) is written in Rust. The Android UI layer is Kotlin + Compose, connected via a boltffi/JNA FFI bridge.

Design goals:
- **GPU-only rendering** — wgpu (Vulkan everywhere), no GL or CPU software path
- **Memory safety** — `torvox-core` is `#![no_std]` + `#![forbid(unsafe_code)]`
- **Architecture discipline** — strict one-way crate dependency enforced at build time
- **Best-in-class VT parsing** — Ghostty's VT engine via Rust bindings (VT520 + Kitty extensions)

---

## Crate Dependency (Strict One-Way)

```
libghostty-vt / libghostty-vt-sys         ← Ghostty VT parser (vendored Zig → C API)
    ↑
torvox-core (no_std, serde + unicode-width)  ← Data model, Grid, Cell, Event
    ↑
torvox-terminal (libghostty-vt + nix + flume) ← PTY, VT parse, Session
    ↑
torvox-renderer (wgpu + cosmic-text + swash + guillotiere) ← GPU render
    ↑
torvox-gui-android (boltffi + JNA)           ← Rust↔Kotlin bridge
    ↑
android/app (Kotlin + Compose)               ← Android UI
```

Each layer knows only about the layers below it. Build-time verification via `nu scripts/check-rust.nu`.

### torvox-core (no_std)
- `Cell`, `Color`, `Attrs` — per-character cell data with serde/rkyv
- `Grid` — scrollable buffer with `GridSnapshot` trait for zero-copy rendering access
- `TerminalState` — VT mode flags, cursor state, tab stops
- `Selection` — Char/Word/Line/Block modes, URL expansion
- `Config` — TerminalConfig, ThemeConfig, RenderConfig, FontConfig
- `CsiHandler`, `OscHandler`, `EscHandler` — VT sequence dispatch
- `SgrAttribute` — SGR parsing and application

### torvox-terminal
- `GhosttyTerminal` — wraps `libghostty-vt` C API via Rust bindings
- `Session` — orchestrator: spawns PTY reader thread, feeds Ghostty, handles OSC events
- `Pty` / `PtyPair` — unix PTY fork/exec with `Pty` trait for testability
- `Keyboard` — input encoding (Kitty keyboard protocol, modifiers, special keys, mouse)
- `ShellEnv` — pre-exec environment setup

### torvox-renderer
- `GpuContext` — wgpu pipeline: shaders, swapchain, atlas texture, instance-based cell/selection/highlight/KGP rendering
- `FontPipeline` — cosmic-text shaping → swash rasterize → guillotiere atlas packing
- `GlyphCache` — LRU cache for rasterized glyphs

### torvox-gui-android
- `TorvoxBridge` — boltffi-exported FFI (~1500 lines, all FFI functions)
- `AndroidSurface` — native window management, render frame loop
- `JniBridge` — JNI for `ANativeWindow_fromSurface()`
- `MockSurface` — CPU software renderer for desktop testing

---

## Thread Model

6-7 threads per terminal session:

| Thread | Function |
|--------|----------|
| Main/UI | Kotlin Compose, bridge calls, event dispatch |
| PTY Reader | Reads PTY master FD, feeds GhosttyTerminal via flume channel |
| Input Writer | Writes user input to PTY master FD |
| Process Waiter | `waitpid()` on child process |
| Render Thread | wgpu frame rendering, driven by CountDownLatch |
| OS Event (optional) | Signal handling, clipboard polling |

```
┌─────────────┐     flume     ┌──────────────────┐
│ PTY Reader  │──────────────→│ GhosttyTerminal  │
│ (dedicated  │               │ process_output() │
│  thread)    │               └────────┬─────────┘
└─────────────┘                        │
                                       ↓
                               ┌──────────────────┐     wgpu    ┌──────────────┐
                               │  Render Thread   │────────────→│  SurfaceView │
                               │  (CountDownLatch)│  render      │  (Vulkan)    │
                               └──────────────────┘             └──────────────┘
```

---

## Render Pipeline

```
PTY Reader → flume → GhosttyTerminal
    → DirtyMask updated
    → RenderThread woken (CountDownLatch)
    → cosmic-text shape glyphs + swash rasterize
    → guillotiere pack into glyph atlas
    → wgpu atlas texture upload
    → CellInstance[] / KgpInstance[] vertex buffers
    → wgpu render_pass (instanced quad drawing)
    → swapchain present
    → SurfaceView (Android Vulkan surface)
```

The pipeline is entirely GPU-driven. The CPU side generates only instance data (position, glyph UV, color, attributes) and uploads it to GPU buffers. All text rendering is done via instanced quads with a glyph atlas texture — no `drawText` call at any point.

---

## FFI Bridge Layout

```
┌───────────────────────────────────────────────────────────┐
│  Kotlin (Compose UI)                                      │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ TorvoxBridge.kt (JNA interface)                     │  │
│  └─────────────────────┬───────────────────────────────┘  │
│                        │ JNA call                         │
│  ┌─────────────────────▼───────────────────────────────┐  │
│  │ NativeWindow.kt (Surface → ANativeWindow via JNI)   │  │
│  └─────────────────────┬───────────────────────────────┘  │
└────────────────────────┼──────────────────────────────────┘
                         │
┌────────────────────────┼──────────────────────────────────┐
│  Rust (torvox-gui-android)                                │
│  ┌─────────────────────▼───────────────────────────────┐  │
│  │ bridge.rs: TorvoxBridge (boltffi #[boltffi::export]) │  │
│  │   - spawn_terminal, render_frame, resize, input     │  │
│  │   - set_theme, set_font, selection, search          │  │
│  │   - clipboard, session save/restore, background     │  │
│  └─────────────────────┬───────────────────────────────┘  │
│  ┌─────────────────────▼───────────────────────────────┐  │
│  │ jni_bridge.rs: ANativeWindow_fromSurface (JNI)      │  │
│  └─────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ surface.rs: AndroidSurface (render loop, session)   │  │
│  └─────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

### Boltffi Data Flow

All data crossing the FFI boundary does so as boltffi POD types:
- `BridgeCell` — cell character, foreground color, background color, attributes
- `BridgeAttrs` — bold, italic, underline, strikethrough, blink, etc.
- `BridgeTheme` — ANSI colors 0-15, foreground, background, cursor
- `TerminalEvent` — output ready, bell, title changed, clipboard request, etc.

The render loop snapshot is serialized via rkyv (zero-copy deserialization) for the bridge.

---

## Data Model

### Cell

```rust
pub struct Cell {
    pub character: char,        // Unicode code point
    pub foreground: Color,      // foreground color
    pub background: Color,      // background color
    pub attributes: Attrs,      // bold, italic, underline, etc.
}
```

### Color

```rust
pub enum Color {
    Default,           // Use terminal default
    Index(u8),         // 0-15 ANSI, 16-231 cube, 232-255 grayscale
    TrueColor(u8, u8, u8),  // 24-bit RGB
}
```

### Grid

The `Grid` is a `Vec<Line>` where each `Line` is a `Box<[Cell]>`. Scrollback is stored as additional lines above the visible region. The `GridSnapshot` trait provides read-only access for the renderer.

### DirtyMask

`DirtyMask { partitions: Vec<u64> }` — bit-packed mask tracking which rows have been modified since last render. Each bit represents one row; each `u64` covers 64 rows.

---

## VT Parsing

Torvox delegates VT parsing to `libghostty-vt` (Ghostty's Zig VT engine, vendored via `libghostty-rs` Rust bindings). This provides:

- VT520/xterm emulation
- SGR attributes (including underline color, double underline, strikethrough, overline, fraktur)
- 256-color and 24-bit truecolor
- Alternate screen buffer
- Scrolling regions (DECSTBM)
- Kitty keyboard protocol (progressive enhancement)
- Kitty graphics protocol (KGP) via PNG
- Mouse tracking (normal, button, any-event, SGR, SGR-pixels)
- Bracketed paste mode
- DEC private modes (DECSET/DECRST)
- Unicode grapheme clustering (DEC mode 2027)
- OSC sequences (0-12, 52, 104, 110-112, 8/hyperlinks, 7/CWD, 9/777 notifications)

Torvox's own code (`csi.rs`, `sgr.rs`, `osc_handler.rs`) handles sequences that Ghostty does not process internally (e.g., some DEC private modes, OSC 7/9/52 interception for CWD, notifications, clipboard).

---

## Configuration

Configurations are `TorvoxCore` types that flow through the bridge:

| Config | Fields |
|--------|--------|
| `TerminalConfig` | Shell, args, env vars, working directory, rows, cols, scrollback lines |
| `RenderConfig` | Background opacity, cursor style, blink interval, selection color |
| `FontConfig` | Family, size, bold/italic variants, ligatures, feature tags |
| `Theme` | 16 ANSI colors + default foreground/background + cursor + selection + 0-15 |

16 built-in themes are compiled in (`config.rs`): BuiltinDark, BuiltinLight, Solarized, Dracula, Nord, OneDark, etc.

---

## Session Lifecycle

1. **Spawn**: Kotlin calls `spawn_terminal()` on the bridge → `Session::new()` → PTY fork/exec → reader thread starts
2. **Run**: Reader thread feeds PTY output → GhosttyTerminal → DirtyMask → RenderThread wakes on each frame
3. **Input**: Kotlin sends keystrokes via `write_input()` → Session writes to PTY master
4. **Resize**: Kotlin calls `resize()` → TIOCSWINSZ ioctl → Ghostty resize
5. **Save/Restore**: Session state serialized via rkyv for process survival
6. **Close**: PTY hangup → reader thread exits → cleanup

---

## Platform Support

| Target | Architecture | Status |
|--------|-------------|--------|
| Android (phone) | aarch64 | Primary target |
| Android (emulator) | x86_64 | Testing target |
| Desktop Linux | x86_64 | Development (MockSurface) |
| Embedded | thumbv6m-none-eabi | Listed, not actively used |

Android API 33+ (targetSdk 36). Vulkan 1.1+ required.

---

## Key Design Decisions

1. **GPU-only rendering** — No CPU fallback path. SwiftShader provides Vulkan on GPU-less emulators. This eliminates all Canvas/CPU rendering code but limits device compatibility.

2. **no_std core** — `torvox-core` does not depend on `std`. This is a design discipline that prevents accidental platform coupling in the data model.

3. **boltffi over JNI** — boltffi generates plain C FFI, avoiding JNI boilerplate for data serialization. JNI is used only where the Android NDK API requires it (ANativeWindow).

4. **Ghostty VT parser** — Rather than writing a custom VT parser (Termux's approach), Torvox reuses Ghostty's battle-tested engine, which supports VT520 + Kitty extensions.

5. **One-way crate dependency** — Enforced at build time. No crate can depend on a crate above it. This prevents circular dependencies and makes the architecture navigable.

6. **rkyv for snapshots** — Zero-copy deserialization for session save/restore across the FFI boundary, avoiding serde overhead.

7. **W^X workaround via torvox-exec** — On Android, the app data directory has W^X (no exec from writable memory). A multi-call binary in `nativeLibraryDir` (the only exec-allowed location) dispatches to the real shell by argv[0] name.

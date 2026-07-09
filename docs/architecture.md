# Torvox Architecture

## Overview

Torvox is a GPU-accelerated Android terminal emulator. The entire terminal engine — VT parsing, PTY management, grid data model, and wgpu-based GPU rendering — is written in Rust across seven crates. The Android UI layer is Kotlin + Compose, connected via a boltffi/JNA FFI bridge. VT parsing is delegated to Ghostty's VT520-class engine (vendored via `libghostty-rs`), providing battle-tested terminal emulation with Kitty protocol extensions.

Design goals: GPU-only rendering (no CPU fallback), memory safety in the core data model (`#![no_std]` + `#![forbid(unsafe_code)]`), strict one-way crate dependency enforcement at build time, and best-in-class VT parsing through Ghostty's engine.

---

## Crate Architecture

The workspace is a strict 7-crate hierarchy with one-way dependency direction. Build-time verification via `cargo metadata --no-deps --format-version 1 | nu scripts/check-rust.nu` catches violations.

```
┌─────────────────────────────────────────────────────────────┐
│                    android/app (Kotlin+Compose)              │
│  UI layer: TerminalScreen, SessionDrawer, ModifierBar,       │
│  SettingsScreen, TextSearchBar, PasteChipOverlay             │
└──────────────────────────┬──────────────────────────────────┘
                           │ JNA + boltffi
┌──────────────────────────▼──────────────────────────────────┐
│             torvox-gui-android (boltffi + JNA)               │
│  bridge.rs: TorvoxBridge (FFI export, ~3242 lines)           │
│  jni_bridge.rs: ANativeWindow_fromSurface (JNI)               │
│  surface.rs: AndroidSurface (render loop + session)           │
│  mock_surface.rs: CPU software renderer (desktop testing)     │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│             torvox-renderer (wgpu + cosmic-text)             │
│  gpu.rs: GpuContext (wgpu pipeline, atlas, instances)        │
│  font.rs: FontPipeline (cosmic-text shaping, swash raster)   │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│             torvox-terminal (VT parse + PTY + session)      │
│  ghostty_terminal.rs: GhosttyTerminal (libghostty-vt wrapper)│
│  session.rs: Session (orchestrator, reader/wait threads)    │
│  pty.rs: PtyPair (Unix PTY fork/exec)                        │
│  keyboard.rs: Input encoding (Kitty keyboard protocol)       │
│  osc_handler.rs: OSC sequence interception                  │
│  shell_env.rs: Pre-exec environment setup                   │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│          torvox-core (no_std, #![forbid(unsafe_code)])       │
│  cell.rs: Cell, Color, Attrs, DirtyMask                      │
│  grid.rs: Grid (scrollable buffer, GridSnapshot trait)       │
│  terminal.rs: TerminalState (cursor, modes, tab stops)       │
│  selection.rs: Char/Word/Line/Block selection modes         │
│  config.rs: TerminalConfig, ThemeConfig, FontConfig          │
│  event.rs: TerminalEvent (OutputReady, Bell, etc.)           │
│  snapshot.rs: SessionSnapshot (rkyv serialization)          │
│  csi.rs: CSI handler dispatch                               │
│  sgr.rs: SGR attribute parsing                              │
│  ansi.rs: ANSI escape sequence constants                    │
│  cursor.rs: CursorState, CursorStyle                        │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│       libghostty-vt / libghostty-vt-sys (vendored Zig)       │
│  Ghostty VT520 parser (Zig→C API) via libghostty-rs bindings │
│  Kitty keyboard + graphics protocols, grapheme clustering    │
└─────────────────────────────────────────────────────────────┘
```

Additional workspace members (not on the dependency chain):
- **torvox-exec**: Multi-call binary for Android W^X workaround — placed in `nativeLibraryDir` (the only exec-allowed location), dispatches to the real shell by `argv[0]` name
- **torvox-mcp**: MCP server (JSON-RPC over Unix socket) for AI agent inspection of terminal sessions
- **torvox-integration-tests**: Cross-crate integration tests
- **torvox-bench**: Performance benchmarks
- **fuzz**: 7 cargo-fuzz targets (VT, OSC, grid, keyboard, selection, attrs, wire format)

### torvox-core (no_std)

The foundational crate. The data model is `#![no_std]` + `#![forbid(unsafe_code)]` — zero `unsafe` verified by `cargo geiger`. Depends only on `serde`, `unicode-width`, and optionally `rkyv`. This design discipline prevents accidental platform coupling and enables potential embedded/bare-metal use.

Key types: `Cell` (character + colors + attributes), `Color` (RGBA), `Attrs` (13 boolean text attributes), `DirtyMask` (bit-packed row-change tracking), `Grid` (scrollable `Vec<Line>` buffer with `GridSnapshot` trait for read-only renderer access), `TerminalState` (cursor, DEC modes, tab stops, scrolling region), `Selection` (Char/Word/Line/Block), `SessionSnapshot` (rkyv-serializable for the bridge).

### torvox-terminal

Depends on `torvox-core`, `libghostty-vt` (C FFI), `nix` (unix PTY), `flume` (channels). The `Session` struct is the orchestrator: it spawns the PTY reader thread, feeds output through `GhosttyTerminal` (which wraps libghostty-vt's C API), and exposes events via channels. `PtyPair` handles the Unix PTY fork/exec — the only `unsafe` block allowed for fork. `Keyboard` encodes Android key events into VT sequences using the Kitty keyboard protocol.

### torvox-renderer

Depends on `torvox-core` and `torvox-terminal` (for `GridSnapshot` and `CellSnapshot`). The GPU pipeline uses `wgpu 29` (Vulkan on Android), `cosmic-text 0.19` (text shaping), `swash 0.2` (glyph rasterization), `guillotiere 0.7` (atlas packing), and `fontdb 0.23` (font discovery). `GpuContext` manages the wgpu instance/adapter/device/queue (global singleton via `OnceLock`), shaders, swapchain, atlas texture, and instance-based rendering. `FontPipeline` handles shaping → raster → pack → upload.

### torvox-gui-android

The only crate that produces a `cdylib` (for Android). Exports ~40 FFI functions via `#[boltffi::export]` in `bridge.rs`. Uses JNI exclusively for `ANativeWindow_fromSurface()`. `AndroidSurface` manages the wgpu surface lifecycle, render frame loop (woken by `CountDownLatch`), and coordinates with the `Session`.

---

## Thread Model

Each terminal session uses 6-7 threads:

```
┌─────────────────────────────────────────────────────────────┐
│                     Thread Layout (per session)              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐    flume     ┌──────────────────────┐     │
│  │ PTY Reader   │───channel──→│ GhosttyTerminal      │     │
│  │ (dedicated   │  Vec<u8>    │ process_output()     │     │
│  │  thread)     │             │ → updates Grid       │     │
│  └──────────────┘             │ → sets DirtyMask     │     │
│                               └──────┬───────────────┘     │
│  ┌──────────────┐                    │ Condvar wake        │
│  │ Input Writer │                    ▼                     │
│  │ (any thread) │──→ PTY Master ┌──────────────────────┐  │
│  └──────────────┘              │ Render Thread         │  │
│                                │ (CountDownLatch wake)  │  │
│  ┌──────────────┐              │ → cosmic-text shape   │  │
│  │ Process      │              │ → swash rasterize     │  │
│  │ Waiter       │── waitpid()  │ → guillotiere pack    │  │
│  │ (exits on    │              │ → atlas upload        │  │
│  │  child exit) │              │ → instance buffers    │  │
│  └──────────────┘              │ → wgpu render_pass    │  │
│                                │ → swapchain present   │  │
│  ┌──────────────┐              └──────┬───────────────┘  │
│  │ Main/UI      │                     │                   │
│  │ (Kotlin      │←── JNA callback ────┘                   │
│  │  Compose)    │                                         │
│  └──────────────┘                                         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

| Thread | Function | Lifespan |
|--------|----------|----------|
| **Main/UI** | Kotlin Compose rendering, bridge calls, event dispatch | Application lifetime |
| **PTY Reader** | Reads PTY master FD in 8KB chunks, sends over flume channel | Session lifetime |
| **Input Writer** | Writes user input (keystrokes, paste) to PTY master FD | Per-write, any caller thread |
| **Process Waiter** | `waitpid()` on child process, sets `exited` flag | Until child exits |
| **Render Thread** | wgpu frame rendering, woken by `CountDownLatch` | Session lifetime (dies after 300 consecutive errors ~30s) |
| **OS Event** (optional) | Signal handling, clipboard polling | As needed |

### Thread Communication

- **PTY → GhosttyTerminal**: `flume::bounded(128)` channel carrying `Vec<u8>` buffers. The reader thread sends raw PTY output; `process_output()` on the session drains it.
- **Wakeup**: `Condvar` (pty output available) for reader synchronization; `CountDownLatch` (frame ready) for render thread wakeup.
- **Exited flag**: `AtomicBool` shared between reader, waiter, and session owner. Reader thread polls the flag between reads; waiter thread sets it after `waitpid()` returns.
- **Clipboard, bell, notifications**: `Arc<Mutex<Option<T>>>` for cross-thread event delivery from OSC handler to bridge.

---

## Render Pipeline

The pipeline is entirely GPU-driven. The CPU side generates only instance data (position, glyph UV, color, attributes) and uploads it to GPU buffers. No `Canvas.drawText` is used at any point.

```
PTY Output (raw bytes)
    │
    ▼
PTY Reader Thread
    │ flume channel
    ▼
GhosttyTerminal::process_output()
    │ libghostty-vt C API
    ▼
Grid updated + DirtyMask set
    │ Condvar wake
    ▼
Render Thread (CountDownLatch)
    │
    ├── 1. Read GridSnapshot + DirtyMask
    │
    ├── 2. Cosmic-text shape changed lines
    │      → ShapedGlyphInfo: glyph_id, font_id, x, y, w
    │
    ├── 3. Swash rasterize new glyphs
    │      → GlyphInfo: alpha bitmap + placement
    │
    ├── 4. Guillotiere pack into atlas
    │      → (x, y) in atlas texture
    │
    ├── 5. Upload dirty atlas regions to GPU texture
    │
    ├── 6. Build CellInstance[] vertex buffer
    │      ┌──────────────────────────────┐
    │      │ CellInstance (repr(C))       │
    │      │ ├── quad_origin: [f32; 2]    │
    │      │ ├── quad_size: [f32; 2]      │
    │      │ ├── atlas_uv: [f32; 2]       │
    │      │ ├── fg_color: [f32; 4]       │
    │      │ ├── bg_color: [f32; 4]       │
    │      │ ├── deco_color: [f32; 4]     │
    │      │ ├── flags: u32               │
    │      │ └── padding: [f32; 3]        │
    │      └──────────────────────────────┘
    │
    ├── 7. Build KgpInstance[] vertex buffer
    │      (for Kitty Graphics Protocol images)
    │
    ├── 8. Build SelectionRange + SearchHighlight
    │
    ├── 9. wgpu render_pass
    │      ├── Background quad (instanced)
    │      ├── Glyph atlas quads (instanced)
    │      ├── Selection overlay
    │      ├── Cursor quad
    │      ├── Highlight quads (search)
    │      └── KGP image quads
    │
    ├── 10. Swapchain present
    │
    └── 11. SurfaceView (Android Vulkan surface)
```

### Pipeline characteristics

- **Atlas**: Single RGBA texture, dynamically grown. Glyphs are rasterized on demand via `LruCache<GlyphKey, GlyphInfo>` (capacity: 10,000). CJK characters get a penalty weight for eviction priority.
- **Shaping**: `cosmic-text` handles BiDi, ligatures, font fallback, and CJK. Each line is shaped independently; only dirty lines are reshaped each frame.
- **Instance rendering**: One `CellInstance` struct per visible character cell. Quad geometry is fixed (unit quad in vertex shader); instance data provides position, size, UV, and colors. This minimizes vertex buffer updates — only the instance buffer changes between frames.
- **Global GPU**: `GlobalGpu` is a process-level singleton (`OnceLock`) for wgpu instance/adapter/device/queue. This prevents SIGABRT on x86_64 Android emulators (SwiftShader) when Activity recreation creates a new `GpuContext` before the old Vulkan instance fully cleans up (`gpu.rs:70-75`).
- **DirtyMask**: Bit-packed `Vec<u64>` where each bit represents one row. Only rows with dirty bits set are reshaped and rebuilt into the instance buffer. The mask is cleared after each `render_frame()` call.

---

## Data Flow

### Output Path (PTY → Screen)

```
┌──────────┐   raw bytes   ┌───────────┐   flume   ┌──────────────┐   DirtyMask  ┌────────────┐
│  Shell   │──────────────→│ PTY Reader│──────────→│  GhosttyTerm │─────────────→│   Grid     │
│ (bash/zsh)│  PTY master  │  Thread   │  channel   │  .process_   │   updated    │ (data model)│
└──────────┘               └───────────┘            │  output()    │              └────────────┘
                                                     └──────────────┘                    │
                                                                                         │ Condvar
                                                                                         ▼
                                                     ┌──────────────┐   snapshot    ┌────────────┐
                                                     │  GPU Render  │←─────────────│  Session   │
                                                     │  Thread      │ GridSnapshot  │ (snapshot) │
                                                     └──────┬───────┘              └────────────┘
                                                            │ wgpu
                                                            ▼
                                                     ┌──────────────┐
                                                     │  SurfaceView │
                                                     │  (Vulkan)    │
                                                     └──────────────┘
```

### Input Path (Touch/Keyboard → PTY)

```
┌──────────────┐   Android     ┌────────────────┐   JNA/boltffi   ┌──────────────┐  write(2)  ┌──────────┐
│  User Touch  │──────────────→│ TerminalInput  │───────────────→│  TorvoxBridge│───────────→│  Shell   │
│  / Keyboard  │ KeyEvent      │ Encoder.kt     │ KeyEvent bytes │  .write_     │  PTY master │ (bash/zsh)│
└──────────────┘               └────────────────┘   + modifiers   │  input()     │            └──────────┘
                                                                   └──────────────┘
```

### Bridge Data Flow

```
Kotlin (JNA)                      Rust (boltffi)
┌──────────────────┐             ┌──────────────────────┐
│ TorvoxBridge.kt  │── JNA ───→  │ bridge.rs            │
│  (interface)     │  C ABI      │  #[boltffi::export]  │
│                  │←─── POD ────│                       │
│  BridgeCell      │  structs    │  BridgeCell           │
│  BridgeAttrs     │             │  BridgeAttrs          │
│  BridgeTheme     │             │  BridgeTheme          │
│  TerminalEvent   │             │  TerminalEvent        │
│  SessionSnapshot │── rkyv ───→│  SessionSnapshot      │
│  (archived bytes)│  zero-copy  │  (Archive trait)      │
└──────────────────┘             └──────────────────────┘
```

### Snapshot Serialization

Session save/restore uses **rkyv** (zero-copy deserialization) to serialize the grid, scrollback, cursor state, and terminal modes across the FFI boundary. The `SessionSnapshot` struct contains `visible_lines: Vec<Line>`, `scrollback_lines: Vec<Line>`, `rows`, `cols`, and `max_scrollback`. On restore, `apply_to_scrollback()` replays lines into a fresh `Grid` with scrollback limit enforcement.

---

## Android Integration

### FFI Stack

```
┌──────────────────────────────────────────────────────────────┐
│  Kotlin (Compose UI)                                         │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ TorvoxBridge.kt        ← JNA interface binding        │  │
│  │   terminal_spawn()                                     │  │
│  │   terminal_close()                                     │  │
│  │   render_frame()                                       │  │
│  │   write_input()                                        │  │
│  │   resize()                                             │  │
│  │   set_theme()                                          │  │
│  │   ... (~40 functions)                                  │  │
│  └──────────────────────────┬─────────────────────────────┘  │
│  ┌──────────────────────────▼─────────────────────────────┐  │
│  │ NativeWindow.kt          ← JNI for ANativeWindow      │  │
│  │   getNativeWindowPtr(surface): Long                    │  │
│  └──────────────────────────┬─────────────────────────────┘  │
└─────────────────────────────┼────────────────────────────────┘
                              │
┌─────────────────────────────┼────────────────────────────────┐
│  Rust (torvox-gui-android)  │                                 │
│  ┌──────────────────────────▼─────────────────────────────┐  │
│  │ bridge.rs               ← boltffi export location      │  │
│  │   TorvoxBridge struct with #[boltffi::export] fns      │  │
│  │   Mutex-guarded, catch_unwind for panic safety          │  │
│  └──────────────────────────┬─────────────────────────────┘  │
│  ┌──────────────────────────▼─────────────────────────────┐  │
│  │ jni_bridge.rs           ← JNI NDK functions            │  │
│  │   ANativeWindow_fromSurface()                           │  │
│  └──────────────────────────┬─────────────────────────────┘  │
│  ┌──────────────────────────▼─────────────────────────────┐  │
│  │ surface.rs              ← Render loop + session        │  │
│  │   AndroidSurface                                        │  │
│  │     - set_native_window()                               │  │
│  │     - native_window_ready()                             │  │
│  │     - render_frame()                                    │  │
│  │     - handle surface destruction/recreation             │  │
│  └─────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Surface Lifecycle

1. **SurfaceView** is created by Compose with `setZOrderOnTop(true)` to ensure the `ANativeWindow` is visible above the Compose layer
2. **NativeWindow.kt** calls `ANativeWindow_fromSurface()` via JNI to get the native window pointer
3. The pointer is passed to Rust via `set_native_window()`
4. `AndroidSurface` creates a wgpu `Surface` from the native window
5. On surface destruction (Activity pause, config change), the surface is released; the render thread enters a waiting state
6. On surface recreation, a new `ANativeWindow` is obtained and the wgpu surface is reconfigured
7. If the render thread encounters 300 consecutive errors (~30s), it exits permanently and requires a session restart

### Lifecycle Model

- **Foreground service**: `TerminalForegroundService` keeps the process alive when the app is backgrounded
- **Application ID**: `com.termux` — intentional design, enables Termux package compatibility
- **Session persistence**: Sessions survive Activity recreation via process-level singleton. Full session save/restore via rkyv enables survival across process death
- **Build**: APK is signed with AOSP testkey (downloaded, not self-signed); ProGuard requires `-dontoptimize` for JNA reflection

---

## Key Design Decisions

### 1. no_std Core

`torvox-core` is `#![no_std]` + `#![forbid(unsafe_code)]`. This prevents accidental platform coupling in the data model and opens the door to embedded/bare-metal use (e.g., `thumbv6m-none-eabi`). The `alloc` crate is used for `Vec`, `String`, and `VecDeque`. The `std` feature gate enables `serde/std` and other platform-dependent functionality only when needed. This is unique among Android terminal emulators — no competitor isolates the data model from `std`.

### 2. Strict One-Way Crate Dependencies

Enforced at build time by `nu scripts/check-rust.nu`. No crate can depend on a crate above it. This prevents circular dependencies, makes the architecture navigable for new contributors, and enables independent testing of each layer. The enforcement script parses `cargo metadata` output to verify the dependency DAG matches the allowed direction. This is the most disciplined architecture among comparable projects (termux-app is flat, Haven has no enforcement).

### 3. GPU-Only Rendering (Vulkan via wgpu)

No CPU fallback path. All text rendering uses instanced quads with a glyph atlas texture — no `Canvas.drawText` anywhere in the pipeline. This is the single biggest differentiator from all competitors (termux-app, Haven, ghostty-android all use CPU Canvas rendering). wgpu provides a cross-platform GPU abstraction that targets Vulkan on Android (via SwiftShader on emulators that lack physical GPUs).

The GPU approach enables: (a) high-throughput output without frame drops, (b) consistent 60fps during scrolling, (c) smooth animations for cursor blink and selection, and (d) potential for visual effects (background opacity, compositing). The cost is implementation complexity: shader management, atlas packing, surface lifecycle, and device compatibility.

### 4. Ghostty VT Parser (Vendored)

Rather than writing a custom VT parser (termux-app's approach) or depending on a third-party library (Haven's ConnectBot termlib), Torvox reuses Ghostty's battle-tested VT520 engine via `libghostty-rs` Rust bindings. This provides VT520-class emulation, Kitty keyboard protocol, Kitty graphics protocol, grapheme clustering (DEC mode 2027), scrollback management, and all standard DEC private modes — without writing or maintaining a single line of VT parsing code.

Ghostty is vendored via git dependency (`Cargo.toml:62-63`) and patched via `scripts/bootstrap-libghostty.nu`. The Zig compiler version for Ghostty must be `zig_0_15` (not the zig_0_16 used by `cargo-zigbuild`), managed via `shellHook` in `flake.nix`.

### 5. BoltFFI Bridge (Not JNI)

BoltFFI generates plain C FFI from Rust structs, avoiding JNI boilerplate for data serialization. All data crossing the FFI boundary does so as boltffi POD types (`BridgeCell`, `BridgeAttrs`, `BridgeTheme`, `TerminalEvent`). JNI is used only where the Android NDK API requires it (`ANativeWindow_fromSurface`). JNA on the Kotlin side provides the FFI call mechanism.

This is a deliberate tradeoff: boltffi + JNA is more complex than a single JNI library (ghostty-android's approach) but provides stronger type safety on the Rust side and avoids manual JNI stub generation for the ~40 exported functions.

### 6. rkyv for Zero-Copy Bridge Serialization

Session save/restore uses rkyv (zero-copy deserialization) rather than serde/bincode for the FFI bridge. rkyv archives can be accessed in-place without parsing overhead, which is critical for the render frame path where a `SessionSnapshot` is produced every frame. The `#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]` on grid types enables direct memory-mapped access on the Kotlin side.

### 7. Global wgpu Singleton

The wgpu instance, adapter, device, and queue are stored in a process-level `OnceLock<GlobalGpu>` (`gpu.rs:67-75`). This prevents SIGABRT on x86_64 Android emulators (SwiftShader) when the Activity is recreated and a new `GpuContext` is created before the old Vulkan instance fully cleans up. All sessions share the same GPU device; only per-session state (swapchain, surface, atlas, instance buffers) is session-local.

### 8. torvox-exec W^X Workaround

Android's W^X policy denies `exec()` from writable memory (the app data directory). Torvox solves this via `torvox-exec` — a multi-call binary placed in `nativeLibraryDir` (the only exec-allowed location on Android). It dispatches to the real shell by inspecting `argv[0]`. This is the same pattern used by Termux (`torvox-exec/src/main.rs`).

### 9. Kitty Keyboard Protocol

Torvox implements the Kitty keyboard protocol (progressive enhancement) for precise modifier and special key encoding. The protocol auto-negotiates with the terminal application, starting with basic CSI sequences and escalating to Kitty's extended format when the application signals support (`keyboard.rs`). This enables correct handling of modifiers (Ctrl+Alt+Shift combos), function keys, and application-defined keys that are impossible to encode with traditional VT sequences.

### 10. Dirty-Mask-Based Incremental Rendering

Rather than rebuilding the entire instance buffer every frame, only rows marked dirty in the `DirtyMask` (a `Vec<u64>` bitmask) are reshaped and rebuilt. The mask is set by `GhosttyTerminal` as output is processed and cleared after each `render_frame()` call. This reduces per-frame CPU work proportionally to output volume — an idle terminal does zero shaping work per frame.

---

## Session Lifecycle

```
                 ┌─────────────────────┐
                 │ SessionCreate        │
                 │  Kotlin calls        │
                 │  bridge.spawn()      │
                 └──────────┬───────────┘
                            │
                            ▼
                 ┌─────────────────────┐
                 │ Configure             │
                 │  TerminalConfig       │  ← rows, cols, shell, env
                 │  ThemeConfig          │  ← ANSI palette, fg/bg
                 │  FontConfig           │  ← family, size, features
                 └──────────┬───────────┘
                            │
                            ▼
                 ┌─────────────────────┐
                 │ Run                  │
                 │  PTY fork/exec       │
                 │  Reader thread start │  ← reads PTY master via flume
                 │  Waiter thread start │  ← waitpid() child
                 │  Render thread start │  ← CountDownLatch frame loop
                 │  Grid active         │  ← GhosttyTerminal processes output
                 └──────────┬───────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
     ┌────────────┐ ┌────────────┐ ┌────────────┐
     │ Input      │ │ Resize     │ │ Config     │
     │ write()    │ │ TIOCSWINSZ │ │ set_theme  │
     │ to PTY     │ │ + ghostty  │ │ set_font   │
     │ master     │ │ resize()   │ │ etc.       │
     └────────────┘ └────────────┘ └────────────┘
              │
              ▼ (PTY hangup / user close)
                 ┌─────────────────────┐
                 │ Destroy              │
                 │  Reader thread exit  │  ← EOF from PTY
                 │  Waiter thread exit  │  ← waitpid() returns
                 │  Render thread exit  │  ← cleanup
                 │  Grid freed          │
                 │  Handle returned     │
                 └─────────────────────┘
                            │
              ┌─────────────┼─────────────┐
              │                           │
              ▼                           ▼
     ┌────────────────┐        ┌────────────────┐
     │ Close          │        │ Save (optional) │
     │ bridge.close() │        │ rkyv serialize │
     │ cleanup PTY    │        │ to file path   │
     └────────────────┘        └───────┬────────┘
                                       │
                                       ▼
                              ┌────────────────┐
                              │ Restore        │
                              │ rkyv deserialize│
                              │ → new Session  │
                              │ resume playback│
                              └────────────────┘
```

### Detailed Lifecycle Steps

1. **Spawn**: Kotlin calls `bridge.terminal_spawn(config)` → `Session::new()` → `PtyPair::spawn()` does `fork()` → child `exec`s the shell with `ShellEnv` setup → parent stores PTY master FD, child PID → reader and waiter threads spawned
2. **Configure**: Theme (ANSI 0-15 + fg/bg), font (family/size/features), cursor style applied via bridge calls
3. **Run**:
   - Reader thread reads 8KB chunks from PTY master (non-blocking, 2ms sleep on `EWOULDBLOCK`), sends via flume channel
   - `process_output()` drains flume, feeds `libghostty-vt::feed()` with raw bytes, checks for OSC events (title, CWD, clipboard, notifications, shell integration), updates grid
   - Render thread waits on `CountDownLatch`, takes `GridSnapshot`, shapes/rasterizes/packs/renders
4. **Input**: Bridge receives encoded key events → `Session::write()` → `Pty::write_all()` to PTY master
5. **Resize**: Bridge receives new dimensions → `Session::resize()` → `Pty::resize()` (TIOCSWINSZ) + Ghostty terminal resize
6. **Save/Restore**: `SessionSnapshot::from_grid()` serializes grid + scrollback via rkyv → written to file. Restore reads archived bytes → `rkyv::access()` → `apply_to_scrollback()` → new `Session` started
7. **Close**: Reader thread detects EOF on PTY → sets `exited` flag → waiter thread returns from `waitpid()` → render thread stops frame loop → `bridge.terminal_close()` frees all resources

---

## Comparison with Alternatives

### termux-app

The most mature Android terminal emulator (10+ years). Pure Java with a custom VT parser (~2617 lines in `TerminalEmulator.java`). CPU Canvas rendering with batched `drawText` calls. 1000+ packages via apt/dpkg in a Termux environment. Plugin ecosystem with 6 official plugins (API, Boot, Float, Styling, Tasker, Widget). Background `Service`-based session model. No GPU acceleration, no Rust, no Kotlin.

Torvox differs by: GPU-accelerated rendering (vs CPU Canvas), Rust-based data model (vs Java object model), Ghostty VT engine (vs custom parser), boltffi bridge (vs JNI C glue), Nix-based build (vs Gradle-only). Torvox gains memory safety and GPU performance but loses termux-app's package ecosystem maturity and simpler debug-ability.

### Haven

A multi-protocol remote desktop client (SSH, VNC, RDP, Mosh, SMB) that happens to include a terminal. ~662 Kotlin files spanning ~30 Gradle modules. Rust used only for RDP via UniFFI-bound `ironrdp`. Terminal rendering outsourced to ConnectBot termlib (third-party, minimally maintained). CPU Canvas rendering. Unique features include Reticulum mesh networking, MCP agent endpoint (~130 tools), biometric lock, and cloud storage integration.

Torvox differs by: focus on local terminal only (no remote protocols), GPU rendering (vs CPU Canvas), Rust as the primary implementation language (vs Kotlin-dominant), strict layered architecture (vs multi-module monorepo). Torvox is more performant and architecturally disciplined but lacks Haven's breadth of connectivity features and UniFFI's ergonomic Rust↔Kotlin bridging.

### ghostty-android

The closest architectural cousin to Torvox. Both use Ghostty's VT engine libghostty-vt. ghostty-android is pure Java with JNI C wrappers (`pty_jni.c`, `terminal_jni.c`, `kitty_unicode.c`) around a prebuilt `libghostty-vt.a`. CPU Canvas rendering with grapheme cluster support. Debian under PRoot with backup/restore. Terminal text search. Kitty graphics protocol rendering (images inline). Single `app` Gradle module with flat Java structure.

Torvox differs by: GPU rendering via wgpu (vs CPU Canvas), Rust throughout (vs Java+JNI), boltffi bridge (vs hand-written JNI), stricter architecture (enforced one-way deps vs flat structure), no Debian PRoot (yet), no Kitty graphics rendering (struct defined but unconnected). Torvox gains GPU performance and memory safety but has a steeper integration stack and lacks several features that ghostty-android implements (search, Kitty graphics, grapheme rendering).

---

## Key Source Files

| File | Purpose | Lines |
|------|---------|-------|
| `torvox-core/src/cell.rs` | Cell, Color, Attrs, DirtyMask (no_std) | ~935 |
| `torvox-core/src/grid.rs` | Grid, Scrollback, GridSnapshot trait | ~1413 |
| `torvox-core/src/config.rs` | TerminalConfig, ThemeConfig, FontConfig | ~1263 |
| `torvox-core/src/selection.rs` | Selection modes (char/word/line/block) | ~963 |
| `torvox-core/src/event.rs` | TerminalEvent, DirtyRegion | ~232 |
| `torvox-core/src/snapshot.rs` | rkyv serialization for Android bridge | ~257 |
| `torvox-terminal/src/pty.rs` | PtyPair — only allowed fork unsafe | — |
| `torvox-terminal/src/session.rs` | Session orchestrator, clipboard, shell integration | ~603 |
| `torvox-terminal/src/ghostty_terminal.rs` | GhosttyTerminal (VT engine wrapper) | ~8640 |
| `torvox-terminal/src/keyboard.rs` | Input encoding (Kitty keyboard protocol) | ~1532 |
| `torvox-terminal/src/shell_env.rs` | ShellEnv (pre-exec environment setup) | — |
| `torvox-renderer/src/gpu.rs` | wgpu render pipeline, atlas, instances | ~4182 |
| `torvox-renderer/src/font.rs` | cosmic-text shaping, swash rasterization | ~2326 |
| `torvox-gui-android/src/bridge.rs` | boltffi data bridge — only export location | ~3242 |
| `torvox-gui-android/src/jni_bridge.rs` | JNI for ANativeWindow_fromSurface | — |
| `torvox-gui-android/src/surface.rs` | AndroidSurface, render loop | ~1390 |
| `torvox-exec/src/main.rs` | Multi-call binary for W^X workaround | ~52 |
| `torvox-mcp/src/main.rs` | MCP server (JSON-RPC over Unix socket) | — |

---

## Platform Support

| Target | Architecture | Graphics | Status |
|--------|-------------|----------|--------|
| Android (phone) | aarch64 | Vulkan (device GPU) | Primary target |
| Android (emulator) | x86_64 | SwiftShader (Vulkan) | Testing target |
| Desktop Linux | x86_64 | Vulkan (device/Mesa Lavapipe) | Development (MockSurface) |
| Embedded | thumbv6m-none-eabi | none | Listed, not actively used |

Android API 33+ (targetSdk 36). Vulkan 1.1+ required. On emulators without physical GPU, SwiftShader provides Vulkan support. On desktop Linux, Mesa Lavapipe provides software Vulkan when no physical GPU is available.

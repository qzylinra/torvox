# AGENTS.md

## Setup and Commands

```bash
nix develop                                         # Enter dev environment
cargo nextest --workspace                            # All Rust tests
cargo clippy -- --deny warnings                      # Lint
cargo fmt --check                                   # Format check
cd android && ./gradlew assembleDebug                # Android debug APK
cd android && ./gradlew spotlessCheck detekt         # Kotlin lint
```

Single crate test: `cargo nextest --package torvox-core`
Property tests (heavy): `QUICKCHECK_TESTS=10000 cargo nextest run --package torvox-core --test property_tests`
Fuzz: `nu scripts/check-rust.nu --fuzz`

---

## Standards (read before writing code)

- `docs/standards/STYLE.md` — Shell/Nix/GHA/General style rules
- `docs/standards/TESTING.md` — Test locations, commands, fuzz/property config
- `docs/standards/QUALITY-GATE.md` — Pre-commit checks, bridge change protocol, E2E

---

## Architecture

### Crate Direction (strict one-way, violations break the build)

```
libghostty-vt / libghostty-vt-sys         ← Ghostty VT parser (vendored)
    ↑
torvox-core (no_std, serde + unicode-width)  ← Data model, Grid, Cell, Event
    ↑
torvox-terminal (libghostty-vt + nix + flume) ← PTY, VT parse, Session
    ↑
torvox-renderer (wgpu + cosmic-text + swash + guillotiere) ← GPU render
    ↑
torvox-gui-android (boltffi + JNA)           ← Kotlin↔Rust bridge
    ↑
android/app (Kotlin + Compose)               ← Android UI
```

Verify: `cargo metadata --no-deps --format-version 1 | nu scripts/check-rust.nu --arch`

### Thread Model

6-7 threads per session. PTY reader and render threads always active; process waiter exits when child exits.

```
PTY Reader → GhosttyTerminal (dedicated thread, flume channel) → Grid
Input Writer → PTY Master
Process Waiter → waitpid
→ RenderThread (Condvar wake) → wgpu → SurfaceView
```

### Render Pipeline

GPU-only via wgpu Vulkan swapchain. No CPU software path.

```
PTY → flume → GhosttyTerminal → DirtyMask → RenderThread
→ cosmic-text shape + swash glyph rasterize → guillotiere pack
→ wgpu atlas upload → Instance[] → wgpu render_frame → SurfaceView
```

---

## When Writing Code

- Read `docs/standards/STYLE.md` before writing any file
- `torvox-core` is `#![no_std]`: no `std::`, no `alloc::` unless behind `#[cfg(feature = "std")]`
- `torvox-core` has zero `unsafe`: verify with `cargo geiger --package torvox-core`
- Sync `torvox-gui-android/src/bridge.rs` types when changing `torvox-core` types
- Run `TorvoxBridge.kt` JNA bindings check when modifying bridge types
- Lint after every file change: `cargo clippy -- --deny warnings`

---

## When Testing

- Read `docs/standards/TESTING.md` before writing tests
- Test public API only, not internal implementation
- One test = one behavior
- Run full suite before commit: `cargo nextest --workspace`

---

## Before Commit

Checklist (all must pass):

1. `cargo nextest --workspace` exits 0
2. `cargo clippy -- --deny warnings` exits 0
3. `cargo fmt --check` exits 0
4. `cargo geiger --package torvox-core` shows no new `unsafe`
5. Bridge type sync: if `torvox-core` types changed, `bridge.rs` + `TorvoxBridge.kt` updated

---

## When Blocked

- If tests fail after 3 attempts: stop and report the failing test with full output
- If a dependency is missing: check `flake.nix` first, then ask
- If you encounter merge conflicts: stop and show the conflicting files
- Never: delete files to resolve errors, force push, skip tests, or add `#[allow(...)]` to suppress real issues

---

## Boundaries

### Must

- Read this file before changing a crate
- Anchor new `unsafe` blocks with a safety comment (`// SAFETY: ...`)
- Keep PR scope under 10 files

### Ask First

- Adding new crate dependencies
- Changing public API surface on `torvox-core` (breaks rkyv wire format)
- Modifying `libghostty-vt` patch

### Never

- Java files, Termux dependency, portable-pty, bincode, rust-android-gradle
- `unsafe` in `torvox-core`
- `setup_scaffolding!()` in multiple crates
- `Canvas.drawText` per cell, raw bytes across FFI, `/proc/self/exe`
- `anyhow` in library crates — use `thiserror 2`
- boltffi Error `message` field — conflicts with Kotlin `Throwable.message`
- Code without spec, 10+ files in one change

---

## Known Pitfalls

| # | Pitfall | Lesson |
|---|---------|--------|
| 1 | `Shell::Custom(u8)` | u8 too small → `String`, lost Copy |
| 2 | `DirtyLine` enum | Changed to `DirtyMask { partitions: Vec<u64> }` |
| 3 | thiserror 2.x + no_std | Set optional, std feature enables |
| 4 | boltffi multi-crate export | Only one export location allowed |
| 5 | boltffi `message` field | Conflicts with Kotlin `Throwable.message` |
| 6 | cargo-ndk cdylib only | Use `CARGO_TARGET_*_LINKER` for torvox-exec |
| 7 | boltffi CLI no bridge gen | Use JNA manual binding (TorvoxBridge.kt) |
| 8 | libghostty-vt API | `scrollback_rows()` not `history_size()`; `resize(rows, cols)` two params |
| 9 | Ghostty Android linking | Dynamic (dylib) + build.rs SONAME strip; static fails (Zig install archive has only lib_vt.o) |
| 10 | Ghostty SONAME | `libghostty-vt.so.0` NEEDED in ELF; build.rs strips versioned SONAME — if skipped, Gradle filters versioned .so |
| 11 | Zig C++ namespace | Zig uses `std::__1`, NDK `libc++_shared.so` uses `std::__ndk1` — must bundle matching libc++ |
| 12 | SurfaceView z-order | `setZOrderOnTop(true)` required; otherwise ANativeWindow hidden behind Compose |
| 13 | Render thread death | After 300 consecutive errors (~30s), thread exits permanently; must restart on new surface |
| 14 | ProGuard R8 | `-dontoptimize` required for JNA reflection-based binding on release builds |
| 15 | ADB touch injection on phone emulator | `adb input tap/swipe` does NOT reach Compose `pointerInput` or TextureView `onTouchEvent` on the 1440x3120 API 35 phone emulator. Use real device, tablet emulator (1080x2090), or `am instrument` UI tests instead. |

---

## Key Files

| File | Purpose |
|------|---------|
| `torvox-core/src/cell.rs` | Cell, Attrs, Color, DirtyMask (no_std) |
| `torvox-core/src/grid.rs` | Grid, Scrollback |
| `torvox-core/src/config.rs` | ThemeConfig, ShellConfig, TerminalConfig |
| `torvox-core/src/selection.rs` | Selection modes (char/word/line/block) |
| `torvox-core/src/event.rs` | TerminalEvent, FocusEvent, CwdEvent |
| `torvox-core/src/snapshot.rs` | rkyv serialization for Android bridge |
| `torvox-terminal/src/pty.rs` | PtyPair — only allowed fork unsafe |
| `torvox-terminal/src/session.rs` | Session orchestrator, clipboard, shell integration |
| `torvox-terminal/src/ghostty_terminal.rs` | GhosttyTerminal (VT engine wrapper) |
| `torvox-terminal/src/keyboard.rs` | Input encoding (Kitty keyboard protocol) |
| `torvox-terminal/src/shell_env.rs` | ShellEnv (pre-exec environment setup) |
| `torvox-renderer/src/gpu.rs` | wgpu render pipeline, atlas, instance management |
| `torvox-renderer/src/font.rs` | cosmic-text shaping, swash glyph rasterization |
| `torvox-gui-android/src/bridge.rs` | boltffi data bridge — only export location |
| `torvox-gui-android/src/jni_bridge.rs` | JNI bridge for NDK functions (ANativeWindow) |
| `torvox-gui-android/src/surface.rs` | AndroidSurface, render pipeline |
| `torvox-mcp/src/main.rs` | MCP server (JSON-RPC over Unix socket) |
| `torvox-fuzz/fuzz/fuzz_targets/` | cargo-fuzz targets (VT, OSC, grid, keyboard, selection, attrs, wire) |

---

## scripts/ Directory

Only these 7 files allowed. No new files — merge into existing.

1. `bootstrap-libghostty.nu`
2. `build-android-libs.nu`
3. `build-apk.nu`
4. `check-rust.nu`
5. `setup-emulator.nu`
6. `test-android-gradle.nu`
7. `test-emulator.nu`

## .github/workflows

Only these 3 files, each with 1 job max. No new files.

1. `rust-checks.yml`
2. `release.yml`
3. `android-tests.yml`

Prefer modifying `scripts/` over workflows. Only modify workflows when scripts/ cannot solve the problem.

- `check-rust.nu` → `rust-checks.yml`
- `build-android-libs.nu` / `build-apk.nu` / `test-emulator.nu` → `release.yml`
- `test-android-gradle.nu` → `android-tests.yml`
- `bootstrap-libghostty.nu` / `setup-emulator.nu` → auxiliary tools

---

## docs/standards/ Reference

| File | When to Read |
|------|-------------|
| `docs/standards/STYLE.md` | Before writing any file |
| `docs/standards/TESTING.md` | Before writing tests |
| `docs/standards/QUALITY-GATE.md` | Before review or commit |

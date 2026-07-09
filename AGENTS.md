# AGENTS.md

## Project Context

Torvox is a GPU-accelerated Android terminal emulator using wgpu (Vulkan) for rendering, Ghostty VT parsing, and a Kotlin+Compose UI. Crate dependency order is strictly one-way.

## Setup and Commands

```bash
nix develop                                         # Enter dev environment
cargo test --workspace                              # All Rust tests
cargo clippy --all -- --deny warnings                # Lint (use --all)
cargo fmt --check                                   # Format check
cd android && ./gradlew assembleDebug                # Android debug APK
cd android && ./gradlew spotlessCheck detekt         # Kotlin lint
nu scripts/check-rust.nu                            # Full Rust CI script
nu scripts/test-android-gradle.nu                   # Full Android CI script
```

Single crate: `cargo test --package torvox-core`
Property tests: `cargo test run --package torvox-core --test property_tests`
Fuzz: `nu scripts/check-rust.nu` (runs unconditionally)

---

## Before Commit

Checklist (all must pass):

1. `cargo test --workspace` exits 0
2. `cargo clippy --all -- --deny warnings` exits 0
3. `cargo fmt --check` exits 0
4. `cargo geiger --package torvox-core` shows no new `unsafe`
5. Bridge type sync: if `torvox-core` types changed, `bridge.rs` + `TorvoxBridge.kt` updated

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

## Standards (read before writing code)

- `docs/standards/STYLE.md` — Shell/Nix/GHA/General style
- `docs/standards/TESTING.md` — Test locations, commands, fuzz/property config
- `docs/standards/QUALITY-GATE.md` — Pre-commit checks, bridge change, E2E

---

## Architecture

### Crate Direction (strict one-way, violations break the build)

```text
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

Verify: `cargo metadata --no-deps --format-version 1 | nu scripts/check-rust.nu`

### Thread Model

6-7 threads per session. PTY reader and render threads always active; process waiter exits when child exits.

```text
PTY Reader → GhosttyTerminal (dedicated thread, flume channel) → Grid
Input Writer → PTY Master
Process Waiter → waitpid
→ RenderThread (CountDownLatch wake) → wgpu → SurfaceView
```

### Render Pipeline

GPU-only via wgpu (Vulkan everywhere, including Android). No GL, no CPU software path. Emulator must provide SwiftShader.

```text
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
- Lint after every file change: `cargo clippy --all -- --deny warnings`
- No magic numbers: use named constants with descriptive names
- No abbreviations: `config` not `cfg`, `background` not `bg`, `application` not `app`, `terminal` not `term` (FFI-bound `bg`/`fg` in BridgeTheme are exempt)
- No `#[allow]` in production source code (test helpers excepted)
- No hardcoded `/data/.*/files` paths for app data — use `filesDir/home` for shell HOME
- No icons in Toast messages
- No `||` in Nushell scripts (invalid syntax)
- No bash/sh — Nushell only
- Rust: use `std::hint::black_box` not deprecated `criterion::black_box`
- Kotlin: use `SharingStarted.WhileSubscribed(TIMEOUT_MILLIS)` with named constant

---

## When Testing

- Read `docs/standards/TESTING.md` before writing tests
- Test public API only, not internal implementation
- One test = one behavior
- Run full suite before commit: `cargo test --workspace`

## Long Output Handling

- Commands that generate large output (dependency trees, full-stack traces) must save output to a temp file instead of dumping inline in panic messages.
- Use `std::env::temp_dir()` for the dump path and reference it in the error message.
- Retry operations only with a bounded maximum (e.g., emulator boot wait, max 5 min).
- `cargo-machete` must use `--skip-target-dir` to avoid IO errors on cached build artifacts. Do NOT use `--with-metadata` unless dependency renaming is present — it causes false positives with proc-macro deps like quickcheck.

---

## When Blocked

- If tests fail after 3 attempts: stop and report the failing test with full output
- If a dependency is missing: check `flake.nix` first, then ask
- If you encounter merge conflicts: stop and show the conflicting files
- Prefer fixing root causes: avoid deleting files, force pushing, skipping tests, or adding `#[allow(...)]` to suppress real issues
- Plan every non-trivial change with sub-agents: exploration, planning, implementation, and acceptance review.

## Known Pitfalls

| # | Pitfall | Lesson |
|---|---------|--------|
| 1 | `Shell::Custom(u8)` | u8 too small → `String`, lost Copy |
| 2 | `DirtyLine` enum | Changed to `DirtyMask { partitions: Vec<u64> }` |
| 3 | thiserror 2.x + no_std | Set optional, std feature enables |
| 4 | boltffi multi-crate export | Only one export location allowed |
| 5 | boltffi `message` field | Conflicts with Kotlin `Throwable.message` |
| 6 | cargo-zigbuild uses zig_0_16, ghostty uses zig_0_15 | `cargo zigbuild --target` handles cross-compilation via Android NDK. Override zig: `(cargo-zigbuild.override { zig = pkgs.zig; })`. Ensure `zig_0_15` is first in PATH via `shellHook` so ghostty finds the correct version. No `CARGO_TARGET_*_LINKER` needed. |
| 7 | boltffi CLI no bridge gen | Use JNA manual binding (TorvoxBridge.kt) |
| 8 | libghostty-vt API | `scrollback_rows()` not `history_size()`; `resize(rows, cols)` two params |
| 9 | Ghostty Android linking | Dynamic (dylib) + build.rs SONAME strip; static fails (Zig install archive has only lib_vt.o) |
| 10 | Ghostty SONAME | `libghostty-vt.so.0` NEEDED in ELF; build.rs strips versioned SONAME — if skipped, Gradle filters versioned .so |
| 11 | Zig C++ namespace | Zig uses `std::__1`, NDK `libc++_shared.so` uses `std::__ndk1` — must bundle matching libc++ |
| 12 | SurfaceView z-order | `setZOrderOnTop(true)` required; otherwise ANativeWindow hidden behind Compose |
| 13 | Render thread death | After 300 consecutive errors (~30s), thread exits permanently; must restart on new surface |
| 14 | ProGuard R8 | `-dontoptimize` required for JNA reflection-based binding on release builds |
| 15 | ADB touch injection on phone emulator | `adb input tap/swipe` does NOT reach Compose `pointerInput` or TextureView `onTouchEvent` on the 1440x3120 API 35 phone emulator. Use real device, tablet emulator (1080x2090), or `am instrument` UI tests instead. |
| 16 | `applicationId = "com.termux"` | Intentional design — do NOT change. Emulator has system `com.termux` with different signing key. `test-emulator.nu` runs `pm uninstall --user 0 com.termux` before Gradle to avoid `INSTALL_FAILED_UPDATE_INCOMPATIBLE`. |
| 17 | APK testkey | Must download from AOSP (`android.googlesource.com/platform/build`). Self-signing (`openssl req -x509`, `keytool -genkey`) is forbidden anywhere in the codebase. `.p12` excluded from git via `android/app/build/` in `.gitignore`. |
| 18 | rapidocr CLI not Python module | All OCR code must use `rapidocr` CLI command, NOT `from rapidocr import RapidOCR`. Model path is patched in `flake.nix` via `overridePythonAttrs { postPatch = '' substituteInPlace rapidocr/config.yaml --replace-fail "model_root_dir: null" "model_root_dir: /tmp/.rapidocr-models" ''; }`. Use `oldAttrs` not `old`, `postPatch` not `postFixup`. |
| 19 | Mesa Lavapipe for Vulkan | GPU renderer uses Vulkan via wgpu. Mesa's Lavapipe (`lvp_icd.x86_64.json`) provides software Vulkan when no physical GPU is available. Do NOT claim "no GPU" — Lavapipe IS the Vulkan implementation. Configured in `flake.nix` via `VK_ICD_FILENAMES`. Emulator must provide SwiftShader (not Lavapipe) for the guest GPU. |

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
| `fuzz/fuzz_targets/` | cargo-fuzz targets (VT, OSC, grid, keyboard, selection, attrs, wire) |

---

## Protected Files (Read-Only Unless Explicitly Requested)

- `.config/`, `.cargo/`, `.github/`, `assets/`, `scripts/` (directories, set read-only)
- `flake.nix`

These files and directories are set read-only. Wait for the user to ask before modifying them.

---

## scripts/ Directory

Only these 9 files allowed. No new files — merge into existing.

1. `bootstrap-libghostty.nu`
2. `build-android-libs.nu`
3. `build-apk.nu`
4. `check-rust.nu`
5. `download-rapidocr-models.nu`
6. `fetch-aosp-testkey.nu`
7. `setup-emulator.nu`
8. `test-android-gradle.nu`
9. `test-emulator.nu`

## .github/workflows

Only these 3 files, each with 1 job max. No new files.

1. `rust-checks.yml`
2. `release.yml`
3. `android-tests.yml`

Prefer `scripts/` over workflows. Only modify workflows when scripts cannot solve the problem.

- `check-rust.nu` → `rust-checks.yml`
- `build-android-libs.nu` / `build-apk.nu` / `test-emulator.nu` → `release.yml`
- `test-android-gradle.nu` → `android-tests.yml`
- `bootstrap-libghostty.nu` / `download-rapidocr-models.nu` / `setup-emulator.nu` → auxiliary tools

---

## docs/standards/ Reference

| File | When to Read |
|------|-------------|
| `docs/standards/STYLE.md` | Before writing any file |
| `docs/standards/TESTING.md` | Before writing tests |
| `docs/standards/QUALITY-GATE.md` | Before review or commit |

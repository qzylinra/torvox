# Technical Context — Torvox

## Technology Stack

| Layer | Technology | Version / Notes |
|-------|-----------|-----------------|
| Language (Rust) | Rust | 2024 edition, `rust-version = "1.97"` |
| Language (Kotlin) | Kotlin | Android app (Jetpack Compose) |
| Build system | Cargo + Gradle + Nix | `flake.nix` for toolchain reproducibility |
| GPU API | wgpu 29 | Vulkan backend only (no GL) |
| VT parser | libghostty-vt | Vendored Ghostty Zig parser; dynamic linking on Android |
| Text shaping | cosmic-text 0.19 | Unicode shaping, ligature support |
| Glyph rasterization | swash 0.2 | Coverage data → atlas |
| Atlas packing | guillotiere 0.7 | 2048×2048 atlas |
| Font discovery | fontdb 0.23 | System font enumeration |
| Serialization (bridge) | boltffi 0.27 + rkyv 0.8 | Rust→Kotlin binary wire format |
| FFI (Kotlin→Rust) | JNA | Reflection-based binding; requires ProGuard `-dontoptimize` |
| PTY | nix 0.31 | `fork()`, `setsid()`, `termios`, `poll()` |
| Async channels | flume 0.12 | Bounded channels for PTY→renderer data flow |
| Concurrency testing | shuttle | Nightly-only concurrency verification |
| Property testing | proptest 1.11, quickcheck 1.1 | Structured fuzzing |
| Mutation testing | cargo-mutants | `.cargo/mutants.toml` |
| OCR verification | rapidocr (CLI) | Font rendering verification; NOT Python module |
| Android emulator testing | Maestro, UiAutomator, Espresso | 6 Android test types |

## Development Setup

### Prerequisites

- Nix with flakes enabled (`nix develop` enters the dev shell)
- Android SDK/NDK (managed by Nix)
- Cursor 2.0+ (for .cursor commands if using cursor-memory-bank workflow)

### CI Pipeline

```
scripts/check-rust.nu        → GitHub Actions rust-checks.yml
scripts/test-android-gradle.nu → GitHub Actions android-tests.yml
scripts/build-apk.nu + test-emulator.nu → GitHub Actions release.yml
```

### Key Commands

```bash
cargo test --workspace              # All Rust tests
cargo clippy --all -- --deny warnings  # Lint
cargo fmt --check                   # Format check
cd android && ./gradlew assembleDebug  # Android APK
cd android && ./gradlew spotlessCheck detekt  # Kotlin lint
```

## Constraints

### Rust

- `torvox-core`: `#![no_std]`, `#![forbid(unsafe_code)]` — no `std::`, no `alloc::` without `std` feature, zero unsafe blocks
- `unsafe` confined to `torvox-terminal/src/pty.rs` (fork/exec) and `torvox-gui-android` FFI, each with `// SAFETY:` comments
- No `anyhow` in library crates — use `thiserror 2`
- No abbreviated variable names, no magic numbers, no `#[allow]` in production code

### Kotlin

- Use `SharingStarted.WhileSubscribed(TIMEOUT_MILLIS)` with named constant
- No icons in Toast messages
- No hardcoded `/data/.*/files` paths — use `filesDir`

### Android

- `applicationId = "com.termux"` — intentional, do not change
- TextureView (not SurfaceView)
- `windowSoftInputMode="adjustNothing"` on `MainActivity`
- ProGuard R8 `-dontoptimize` required for JNA

### Infrastructure

- Nushell only — no bash/sh
- Must use AOSP testkey for APK signing (not self-signed)
- Emulator test: `pm uninstall --user 0 com.termux` before Gradle to avoid `INSTALL_FAILED_UPDATE_INCOMPATIBLE`

## Workspace Layout

```
torvox/
├── torvox-core/          # Data model (no_std)
├── torvox-terminal/      # PTY, VT parsing, session
├── torvox-renderer/      # GPU rendering pipeline
├── torvox-gui-android/   # Rust↔Kotlin bridge
├── torvox-mcp/           # MCP server
├── torvox-exec/          # SSH/Mosh executable
├── torvox-integration-tests/
├── torvox-bench/         # Criterion benchmarks
├── fuzz/                 # cargo-fuzz targets
├── android/              # Android app (Kotlin + Compose)
├── docs/                 # Architecture, standards, SRS, ADRs
├── memory-bank/          # Lessons learned, project context
└── scripts/              # CI scripts (Nushell only, 9 files max)
```

# Torvox Test Results

## Rust Tests (2026-07-09)

| Test Suite | Result | Details |
|-----------|--------|---------|
| `cargo clippy --all -- --deny warnings` | ✅ PASS | No warnings |
| `cargo fmt --check` | ✅ PASS | All formatted |
| `cargo test --package torvox-core` | ✅ 15/15 | All doc tests pass |
| `cargo test --package torvox-terminal` | ✅ 39/39 | VT parsing, SGR, cursor, selection, layout, DEC modes |

## Android Build & Tests

| Check | Result | Details |
|-------|--------|---------|
| `./gradlew :app:compileDebugKotlin` | ✅ BUILD SUCCESSFUL | Main source compiles |
| `./gradlew :app:testDebugUnitTest` | ✅ 786/807 PASS | 21 pre-existing failures (font enumeration, keyboard layout) |
| `./gradlew :app:assembleDebug` | ✅ BUILD SUCCESSFUL | APK generated |
| Emulator install | ✅ SUCCESS | Installed on emulator-5554 (API 35) |
| App launch | ✅ SUCCESS | `com.termux` launches on emulator |

## Key Fixes Applied

### 1. Font Rendering

- IME keyboard font stretch: Projection decoupled from swapchain
- CJK fallback: Outline detection + cosmic-text AttrsList pass-through
- Font picker crash: `catch_unwind` + state restoration

### 2. Cursor

- Blink reset on all user input (commitText, sendKeyEvent, tap, long press)
- Force render reliability improved (volatile flag)

### 3. Text Selection

- Long press drag extension support
- Handle drag with edge auto-scroll
- Paste button on blank area long press

### 4. Scrollback

- `scrollback_lines` now correctly threaded from settings → Session → GhosttyTerminal
- Previously hardcoded to `DEFAULT_SCROLLBACK_LINES` (50000)

### 5. Other Fixes

- UrlDetector: Restored custom regex (fixed unit tests)
- `forceRender()`: Added volatile flag for reliable render wake

## Known Remaining Issues (Pre-existing)

- Android test sources have compilation errors (`assertIsDisplayed`, `performLongClick` unresolved)
- Emulator resolution is 1080x2400 (not 480x854 as requested)
- Font enumeration tests fail in CI environment (fonts not installed)

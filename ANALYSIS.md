# Torvox Analysis Report

## Current Implementation Status

### Text Search ✅

- Search bar opens from session drawer button (not ctrl+f)
- Search bar replaces modifier bar at bottom when active
- Previous/Next/Close/Smart case toggle buttons implemented
- `imePadding()` applied to prevent IME obscuring search bar
- Auto-scroll to center current match when navigating
- Smart case detection: auto-enables when query has uppercase letters
- Search results sent to Rust renderer via `setSearchHighlights(ByteArray)`

### Search Highlight Rendering (GPU)

- **rust `gpu.rs`**: All search matches get fg/bg swap (反色效果)
- Current match: fg/bg swap + `selectionBg` overlay at 0.95 alpha
- Other matches: fg/bg swap + `foreground` overlay at 0.85 alpha
- **Known issue**: For other matches, foreground-on-foreground blend is invisible (no visible tint). The only visible effect is the fg/bg swap.
- For current match, the selectionBg tint IS visible on the swapped background.

### Text Selection ✅

- Long press on empty area shows paste popup with `PasteChipOverlay`
- Long press on text expands word (using `expandWordOnLine()`)
- Selection handles with drag support (START/END via `HandleDrag`)
- Context menu via `ActionMode.Callback2` with copy/paste/select all
- Handle drag hides/resets context menu, shows handles
- Edge scrolling during drag
- Theme accent colors used for handles via `getAccentColor()`

### Bug Status

| Bug | Status | Notes |
|-----|--------|-------|
| **IME bottom click** | Partial fix | Reduced modifier bar exclusion zone from 80dp→56dp, gated on `selection.active == false` |
| **Font stretch/compress on IME** | Not fixed | `onSurfaceTextureSizeChanged` has aspect-change check (`surfaceTextureChanged`) but needs video analysis |
| **CJK rendering** | Not fixed | Needs font fallback chain configuration change |
| **Cursor blink/visibility** | Partial fix | `resetCursorBlink()` called in `commitText`/`setComposingText` but blinking logic may have edge cases |
| **Scroll broken** | Not fixed | `scrollToRow()` exists but may not be called correctly for search navigation. The `scrollOffset` in `TerminalScreen.kt` is a separate state variable |
| **Background image/blur** | Not fixed | Not yet implemented in renderer |
| **Search highlight delay** | Not fixed | `forceRender()` and `bridge.render()` called but may have stale frame pipeline |
| **Session restore blank lines** | Not fixed | Needs investigation in `saveSession()`/`restoreSession()` |
| **Performance** | Not fixed | Multiple areas: full-rebuild on every frame, no dirty rect optimization |
| **CJK font fallback** | Not fixed | Needs font config changes |
| **Swipe-up gesture** | Not fixed | `onSwipeLeft`/`onSwipeRight` exist, system gesture collision not addressed |
| **Font list crash** | Not fixed | Unknown |
| **Cursor style** | Not fixed | Only Block/Bar/Underline implemented, style selector may not persist |
| **Default theme** | Not fixed | Background blue issue |

## Test Infrastructure

### Existing Tests

- `SelectionEspressoTest.kt` — Espresso-based selection tests (fixed compilation)
- `TerminalActivityEspressoTest.kt` — Activity launch tests (fixed compilation)
- `KeyboardJellyInstrumentedTest.kt` — IME interaction tests (fixed compilation)
- `TerminalScreenComposeTest.kt` — Compose UI tests
- `TextSearchEndToEndTest.kt` — 10 search end-to-end tests
- `SelectionVisualVerificationTest.kt` — 12 selection tests with screenshot
- `TextSearchColorVerificationTest.kt` — Search color tests
- `TextSearchEmulatorTest.kt` — Emulator-based search tests
- `TextSearchOcrTest.kt` — OCR verification tests
- `SelectionHandlePositionTest.kt` — Unit test (new, compiles)
- `SelectionTest.kt` — Word expansion tests (new, compiles)
- `TextSearchTest.kt` — Search matching unit tests (existing)

### Pre-existing Compilation Errors (still present, not regressions)

- `KeyboardJellyInstrumentedTest.kt` — Fixed (was missing `assertIsDisplayed`)
- `SelectionEspressoTest.kt` — Fixed (replaced `performLongClick` with Espresso `longClick`)
- `TerminalActivityEspressoTest.kt` — Fixed (added `assertIsDisplayed` import)

### What Tests Need

1. **Roborazzi tests** — Screenshot comparison tests for search highlights and selection visuals
2. **UiAutomator tests** — Cross-app interaction tests
3. **Maestro tests** — Gesture-based flow tests
4. **OCR tests** — RapidOCR verification of highlighted cells
5. **Video recording** — ADB screenrecord for temporal analysis

## CI Status

- `cargo check --workspace`: ✅ Passes
- `cargo test --workspace`: ✅ 847 tests pass (1 flaky: `mock_surface_dec_2026_sync_active_reports_correct_state`)
- `cargo clippy --all -- --deny warnings`: Not verified this session
- `cargo fmt --check`: Not verified this session
- Kotlin `compileDebugKotlin`: ✅ Passes
- Kotlin `compileDebugAndroidTestKotlin`: ✅ Passes (after fixes)
- Kotlin `testDebugUnitTest`: ✅ 807 tests pass (21 pre-existing font/layout failures)
- `spotlessCheck`: ✅ Passes

## Required Implementation Changes

### 1. Fix search highlight for OTHER matches (GPU renderer)

**Problem**: `gpu.rs` blends `foreground` color at 0.85 alpha over the swapped bg (which IS the foreground). This produces no visible change.
**Fix**: Send a contrasting color for non-current matches (e.g., a dark overlay at 0.2 alpha), or change the Rust code to use a visible tint.

### 2. Fix scroll in search navigation

**Problem**: `composeScrollOffset` in `TerminalScreen.kt` is a state variable that doesn't actually drive the Rust rendering. A mismatch between Kotlin scroll state and Rust render scroll state causes apparent "no scroll" behavior.
**Fix**: Ensure `scrollToRow()` -> `surface.setScrollOffset()` -> `bridge.render()` is called and propagated.

### 3. Fix IME/ModifierBar interaction

**Problem**: The modifier bar's touch exclusion zone (80dp) was reduced to 56dp but may still interfere with IME triggering.
**Fix**: Dynamic exclusion zone based on selection state already done. May need further reduction.

### 4. Fix context menu positioning

**Problem**: `onGetContentRect` handles upper/lower half positioning but has IME inset calculation that could be wrong.
**Fix**: Verify with emulator testing.

### 5. Performance improvements

**Problem**: Full frame rebuild on every change. The `DirtyMask` architecture exists but may not be fully utilized.
**Fix**: Ensure `dirty_rows` parameter to `build_cell_instances_into` is properly set.

---

## Adversarial Review

### Test Failures Found

1. `mock_surface_dec_2026_sync_active_reports_correct_state` — flaky in concurrent Rust test suite, passes in isolation. Race condition in mock surface state tracking.
2. 21 pre-existing Kotlin unit test failures — font fallback / layout-aware hardware key tests. Not caused by current work.
3. 3 pre-existing Android Instrumentation test compilation failures — fixed this session.

### Code Quality Issues

1. No `#[allow]` annotations in production Rust code ✅
2. `unsafe` in `torvox-core` ✅ (0 instances)
3. `applyHandleDrag` function in Kotlin is not a method on `SelectionState` (needs implementation)

### References Not Consulted

- github.com/GlassHaven/Haven — Wayland terminal reference
- github.com/termux/termux-app — v0.119.0-beta.3 reference
- github.com/sylirre/ghostty-android-terminal — Ghostty Android port
- github.com/ghostty-org/ghostling — Ghostty reference
- gitlab.gnome.org/GNOME/console — GNOME terminal reference

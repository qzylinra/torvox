# Progress

## 2026-07-10 Session

### Completed Changes

#### Text Search Improvements

1. **Search bar replaces modifier bar** - When search is opened via drawer button, search bar appears at bottom replacing the modifier bar area
2. **Smart case toggle** - `Aa` button toggles case sensitivity. Auto-detects when query has uppercase letters
3. **Current match vs other match visual distinction** - Current match uses selectionBg color at 0.95 alpha, other matches use foreground color at 0.85 alpha
4. **Immediate highlight clearing** - Added `forceRender()` calls after setting/clearing highlights so they appear/disappear instantly
5. **Force render after scroll offset change** - Added direct `bridge.render()` call in `setScrollOffset` for immediate visual feedback

#### Selection Fixes

1. **syncSelectionToNative column ordering** - Fixed conditional column swap to use `minOf`/`maxOf` unconditionally (G2 fix)
2. **Handle drag ACTION_DOWN** - Now uses touch position (col/row) instead of stored anchor values (G1 fix)
3. **IME bottom area** - Reduced modifier bar touch exclusion zone from 80dp to 56dp to improve IME triggering near terminal bottom

#### Cursor Blink

1. **Direct render call** - `setCursorVisible` now calls `bridge.render()` directly in addition to `forceRender()`, ensuring cursor blink is always visible

#### Rust Renderer - Search Highlight Inversion

1. **Proper fg/bg swap for search highlights** - The renderer now swaps foreground and background colors for search-highlighted cells (反色效果), matching terminal emulator best practices
2. **Applied in all 4 code paths** - dirty cell loop, empty/space cell loop, glyph rendering loop, and the second cursor rendering section

#### Testing

1. **TextSearchEndToEndTest.kt** (new) - 10 tests: search from drawer button (not ctrl+f), text highlighting, next/previous navigation with scroll, smart case toggle, close restores modifier bar, multi-line search, empty query, no results, IME interaction, theme color verification with OCR
2. **SelectionVisualVerificationTest.kt** (new) - 12 tests: long press on text/empty area, selection handle positions, drag to extend selection, context menu position, IME interaction, drawer interaction, theme colors, select all, OCR verification, double tap line select, triple tap select all

### Build Status

- **Rust**: cargo check --workspace passes
- **Kotlin**: compileDebugKotlin passes
- **Formatting**: spotlessCheck passes
- **Rust tests**: All pass (654+ core, 847+ full workspace)
- **Kotlin unit tests**: 807 completed, 21 pre-existing failures (font-related, CI environment)

### Known Remaining Issues

- IME font stretch/compression when keyboard opens/closes (needs video frame analysis)
- CJK font fallback rendering quality
- Background image/blur not working
- Cursor disappearing under certain conditions
- Session restore adds extra blank lines
- Terminal scroll not smooth in some cases
- Gesture swipe-up handling
- Full emulator testing with 480x854 360dp resolution
- Emulator screen recording analysis

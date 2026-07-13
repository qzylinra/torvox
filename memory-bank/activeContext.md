# Active Context — Torvox

## Current Focus: IME Pixel-Stable Terminal Layout

**Status**: Implementation complete, need on-device verification

### What Was Done

Fixed the soft keyboard jelly effect that caused terminal rows/columns to change when the keyboard opens/closes. The root cause chain:

1. Android `View.setImeWindowInsets()` modifies `mPaddingBottom` → calls `requestLayout()`
2. `requestLayout()` propagates to Compose → `onMeasure` returns smaller height → fewer rows
3. Even if `onMeasure` ignores insets, `setPadding` still calls `requestLayout()`

**Fix**:
- `AndroidManifest.xml`: `windowSoftInputMode="adjustNothing"` on MainActivity
- `TerminalSession.kt`: `imeInsets` → `imeBottomPadding` state, `contentWindowInsets = WindowInsets(0.dp)`, `imePadding()` on content column, removed `imeNestedScroll()`
- `Session.kt`: Simplified bridge to `updateLayoutStable(rows, cols, cellWidthPx, cellHeightPx)`

### Verification Status

| Gate | Status |
|------|--------|
| Rust tests (847 pass) | ✅ |
| Rust clippy | ✅ |
| Rust fmt | ✅ |
| Kotlin spotlessCheck | ✅ |
| Kotlin detekt | ✅ |
| Android lint | ✅ |
| `assembleDebug` | ✅ |
| IME unit tests (3/3) | ✅ |
| On-device verification | ❌ ADB device offline |
| `test-emulator.nu` | ❌ Blocked by device |

### Next Steps

1. (Blocked) On-device verification — need ADB device or emulator
2. After verification: mark goal complete and archive

## Recent Activity

- IME pixel-stable layout: `android/app/src/main/AndroidManifest.xml`, `TerminalSession.kt`, `Session.kt`, `ImeInsetsTest.kt`
- Native lib rebuilt, APK assembled successfully
- This memory-bank is being updated following cursor-memory-bank structure (v0.8 pattern)

## Open Questions

- Should IME pixel-stable be considered complete based on unit tests + CI alone, or require on-device verification?
- When to attempt emulator reconnection?

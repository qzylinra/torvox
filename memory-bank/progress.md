# Progress — Torvox

## Session Overview

| Date | Task | Status | Summary |
|------|------|--------|---------|
| 2026-07-13 | IME pixel-stable layout | In progress | Implementation done. CI gates pass. On-device verification blocked by offline ADB. |
| 2026-07-13 | Memory-bank restructure | In progress | Adding projectbrief, productContext, systemPatterns, techContext, activeContext, progress, tasks following cursor-memory-bank v0.8 pattern. |

## Implementation Status

### IME Pixel-Stable Layout

| Component | Status | Notes |
|-----------|--------|-------|
| `AndroidManifest.xml` | ✅ | `adjustNothing` on MainActivity |
| `TerminalSession.kt` | ✅ | `imeBottomPadding`, `WindowInsets(0.dp)`, `imePadding()` |
| `Session.kt` | ✅ | `updateLayoutStable` bridge |
| `ImeInsetsTest.kt` | ✅ | 3 tests passing |
| Kotlin lint/check | ✅ | spotless, detekt pass |
| APK build | ✅ | `assembleDebug` successful |
| On-device test | ❌ | Blocked: ADB offline |

### Memory-Bank Documentation

| File | Status | Notes |
|------|--------|-------|
| `projectbrief.md` | ✅ | New |
| `productContext.md` | ✅ | New |
| `systemPatterns.md` | ✅ | New |
| `techContext.md` | ✅ | New |
| `activeContext.md` | ✅ | New |
| `progress.md` | ✅ | This file |
| `tasks.md` | ✅ | New |
| `index.md` | Updated | Merged with cursor-memory-bank structure |
| `lessons/` (7 files) | Preserved | Existing lessons kept + new IME lesson added |

## Observations

- The `WindowInsets(0.dp)` parameter only clears Compose's window inset handling, not Android's View-level padding. The real fix is `adjustNothing`.
- Ghostty's `scrollbackLine()` has lazy access semantics — use `getTerminalText()` for full content iteration.
- JNA doesn't support `Array<ByteArray>` — use `Pointer` + manual `Memory` allocation.
- Mesa Lavapipe is faster to build than SwiftShader (pre-built on Nix cache vs 30-min source build).

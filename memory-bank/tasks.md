# Tasks — Torvox

## Active Task

| ID | Task | Complexity | Status |
|----|------|------------|--------|
| T-003 | IME pixel-stable terminal layout | Level 2 | In progress |

### T-003 Checklist

- [x] Research root cause of IME layout instability
- [x] Set `windowSoftInputMode="adjustNothing"` on MainActivity
- [x] Replace `imeInsets` with `imeBottomPadding` state
- [x] Remove `imeNestedScroll()`, add `imePadding()` to content column
- [x] Simplify bridge to `updateLayoutStable(rows, cols, cellWidthPx, cellHeightPx)`
- [x] All Rust tests pass (847)
- [x] Clippy clean
- [x] Kotlin lint clean (spotlessCheck, detekt)
- [x] Android lint pass
- [x] APK build successful
- [x] IME unit tests pass (3/3)
- [ ] On-device verification
- [ ] Archive task with reflection

## Pending Tasks

| ID | Task | Complexity | Notes |
|----|------|------------|-------|
| T-004 | Memory-bank restructure | Level 2 | Adding core files following cursor-memory-bank v0.8 pattern. |
| T-005 | Write IME pixel-stable lesson | Level 1 | Document root cause, fix, and lesson in `lessons/07-ime-pixel-stable.md` |

## Completed Tasks

| ID | Task | Date | Complexity |
|----|------|------|------------|
| T-001 | Initial project setup | Earlier | Level 4 |
| T-002 | Bridge type sync discipline | Earlier | Level 2 |

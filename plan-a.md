# Stage 2 Plan — Selection Bug Fixes

## Overview

Fix 6 selection bugs identified in Stage 1 audit. All changes are in the Android layer (Kotlin). No Rust/native bridge changes needed.

## Files Modified

- `android/app/src/main/java/io/torvox/ui/TerminalSurface.kt`
- `android/app/src/main/java/io/torvox/TerminalViewModel.kt`
- `android/app/src/main/java/io/torvox/runtime/TorvoxRuntime.kt`

---

## G1 (P0) — Handle drag on ACTION_DOWN passes stale column

**File:** `TerminalSurface.kt:1353-1359`

**Root cause:** `startCol`/`endCol` are read from `minOf(currentSelection.start.col, currentSelection.end.col)` / `maxOf(currentSelection.start.col, currentSelection.end.col)` (lines 1345-1346), which are the original anchor columns, not derived from the touch position. This makes the handle drag snap back to the anchor column rather than following the finger.

**Fix:** Replace `startCol`/`endCol` in the `ACTION_DOWN` handler with `(event.x / cellWidth).toInt()`.

### Edit 1.1 — Start handle

```diff
oldString: viewModel?.updateSelectionStart(touchRow.coerceIn(loRow, hiRow), startCol)
newString: viewModel?.updateSelectionStart(touchRow.coerceIn(loRow, hiRow), (event.x / cellWidth).toInt())
```

### Edit 1.2 — End handle

```diff
oldString: viewModel?.updateSelection(touchRow.coerceIn(loRow, hiRow), endCol)
newString: viewModel?.updateSelection(touchRow.coerceIn(loRow, hiRow), (event.x / cellWidth).toInt())
```

---

## G2 (P0) — syncSelectionToNative column ordering bug

**File:** `TerminalViewModel.kt:940-948`

**Root cause:** `syncSelectionToNative()` conditionally swaps columns based on row ordering (`if (start.row <= end.row)`). This is wrong — on multi-row selections, `loCol`/`hiCol` should always be `minOf(start.col, end.col)` / `maxOf(start.col, end.col)` regardless of which row is first. Row ordering only determines `loRow`/`hiRow` (already correct).

**Fix:** Replace the conditional column swap with unconditional `minOf`/`maxOf`.

### Edit 2.1

```kotlin
oldString:             val loCol: Int
            val hiCol: Int
            if (start.row <= end.row) {
                loCol = start.col
                hiCol = end.col
            } else {
                loCol = end.col
                hiCol = start.col
            }
newString:             val loCol = minOf(start.col, end.col)
            val hiCol = maxOf(start.col, end.col)
```

---

## G3 (P1) — coerceIn prevents handle crossover

**File:** `TerminalSurface.kt:1355,1359`

**Root cause:** `touchRow.coerceIn(loRow, hiRow)` on `ACTION_DOWN` clamps the touch row to the current selection bounds, making it impossible to drag a handle past the opposite anchor to reverse selection direction. The `applyHandleDrag` logic already handles crossover correctly (via `HandleDragResult` flip), so the `coerceIn` is both unnecessary and harmful.

**Fix:** Remove `.coerceIn(loRow, hiRow)` from both `updateSelectionStart` and `updateSelection` calls in the `ACTION_DOWN` handler.

### Edit 3.1 — Start handle

```diff
oldString: viewModel?.updateSelectionStart(touchRow.coerceIn(loRow, hiRow), (event.x / cellWidth).toInt())
newString: viewModel?.updateSelectionStart(touchRow, (event.x / cellWidth).toInt())
```

### Edit 3.2 — End handle

```diff
oldString: viewModel?.updateSelection(touchRow.coerceIn(loRow, hiRow), (event.x / cellWidth).toInt())
newString: viewModel?.updateSelection(touchRow, (event.x / cellWidth).toInt())
```

---

## G4 (P1) — accentColor never updated after theme change

**File:** `TorvoxRuntime.kt`

**Root cause:** `accentColor` (line 73) is initialized to a hardcoded `0xFF2196F3` (Material Blue) and never updated. When the user changes theme via Settings, `applySettings()` calls `buildConfig()` which calls `makeBridgeTheme()` which has access to `ansi5` — but the accent color cache is never refreshed.

**Fix:** Set `accentColor = bridgeTheme.ansi5` in `buildConfig()` after the bridge theme is built.

### Edit 4.1

In `buildConfig()`, after `val bridgeTheme = makeBridgeTheme(resolvedTheme)` (line 144), add:

```kotlin
accentColor = bridgeTheme.ansi5
```

This ensures `accentColor` is refreshed every time the config is built (which happens on startup AND on every `applySettings()` call).

---

## G5 (P1) — Handle Y positions ignore scrollOffset

**File:** `TerminalSurface.kt:292-296, 319-323`

**Root cause:** `startCursorY` and `endCursorY` are computed from grid row positions (`(startRow + 1) * cellHeight`) without subtracting `scrollOffset * cellHeight`. When the user scrolls back, handles jump below the visible selection rather than tracking the visual position.

**Fix:** Subtract `scrollOffset * cellHeight` from both Y calculations.

### Edit 5.1 — Start handle Y

```diff
oldString:             val startCursorY =
                ((startRow + 1) * cellHeight)
                    .toInt()
                    .coerceAtMost(surfaceHeightPixels - handleH)
newString:             val startCursorY =
                ((startRow + 1) * cellHeight - scrollOffset * cellHeight)
                    .toInt()
                    .coerceAtMost(surfaceHeightPixels - handleH)
```

### Edit 5.2 — End handle Y

```diff
oldString:             val endCursorY =
                ((endRow + 1) * cellHeight)
                    .toInt()
                    .coerceAtMost(surfaceHeightPixels - handleH)
newString:             val endCursorY =
                ((endRow + 1) * cellHeight - scrollOffset * cellHeight)
                    .toInt()
                    .coerceAtMost(surfaceHeightPixels - handleH)
```

---

## G6 (P1) — Context menu Y ignores scrollOffset

**File:** `TerminalSurface.kt:460-461`

**Root cause:** `selectionTopPx` and `selectionBottomPx` use `selectionStartRow * cellHeight` and `(selectionEndRow + 1) * cellHeight` without accounting for `scrollOffset`. When scrolled back, the context menu appears below the visual selection.

**Fix:** Subtract `scrollOffset * cellHeight` from both.

### Edit 6.1

```diff
oldString:             val selectionTopPx = (loc[1] + selectionStartRow * cellHeight).toInt()
            val selectionBottomPx = (loc[1] + (selectionEndRow + 1) * cellHeight).toInt()
newString:             val selectionTopPx = (loc[1] + (selectionStartRow - scrollOffset) * cellHeight).toInt()
            val selectionBottomPx = (loc[1] + (selectionEndRow + 1 - scrollOffset) * cellHeight).toInt()
```

---

## Implementation Order

The fixes are independent with no ordering dependencies. Recommended order for clarity:

1. **G2** (syncSelectionToNative) — pure logic fix, no effect on tests of other components
2. **G1** + **G3** (TerminalSurface ACTION_DOWN) — same location, apply together
3. **G5** + **G6** (scrollOffset in handle Y + menu Y) — similar pattern, both in TerminalSurface
4. **G4** (accentColor refresh) — separate file, isolated change

## Verification

### Unit Tests

No new unit tests are strictly required for these fixes, but existing tests must continue to pass:

- `SelectionHandleDragStateTest` — tests `applyHandleDrag` behavior (unaffected by these edits, but verifies the crossover logic that G3 depends on)
- `TerminalViewModelSelectionTest` — tests `applyHandleDrag` and `SelectionAnchor` logic (unaffected)

### Manual Verification

| Bug | Test Scenario | Expected |
|-----|--------------|----------|
| G1 | Long-press text, then drag the start handle | Handle snaps to finger X position, not anchor column |
| G2 | Select text across 3+ rows, copy, paste in another app | All selected text is copied, not missing first/last line columns |
| G3 | Drag start handle past the end handle (or vice versa) | Selection reverses direction; handles flip anchors |
| G4 | Change theme in Settings | Handle accent color and selection highlight update to new theme's magenta/ansi5 |
| G5 | Select text, scroll back, drag handles | Handles stay on visual selection, not at grid-row position |
| G6 | Select text, scroll back, tap selection | Context menu appears at visual selection position |

### CI Commands

```bash
./gradlew testDebugUnitTest
./gradlew spotlessCheck detekt
```

## Tests That Need Updating

No existing tests break from these changes. No test exercises `syncSelectionToNative` directly (private method), the ACTION_DOWN handle drag code (UI event-driven), `scrollOffset` computation, or `accentColor` refresh.

If desired, new tests could be added:

1. **`syncSelectionToNative` column logic** — test `TerminalViewModel` by setting selection with reversed multi-row anchors and verifying `runtime.setSelection` receives correct `loCol`/`hiCol`
2. **Handle Y scroll offset** — test `showSelectionHandles` with non-zero `scrollOffset` and verify popup Y position
3. **Context menu Y scroll offset** — test context menu positioning with non-zero `scrollOffset`

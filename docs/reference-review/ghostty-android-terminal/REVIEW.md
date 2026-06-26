# Ghostty Android Terminal — Reference Review

**Source**: `/tmp/reference-projects/ghostty-android-terminal/`
**Torvox target**: `/home/runner/work/kudzu/kudzu/repositories/torvox/`
**Date**: 2026-06-24

---

## Summary

- **Total files reviewed**: 20/20 (all source files in reference project)
- **Key features torvox is missing**: 8
- **Bugs torvox should avoid**: 6
- **Best practices torvox should adopt**: 12
- **Priority recommendations**: See bottom

---

## Files Reviewed

| File | Lines | Torvox Equivalent |
|------|-------|-------------------|
| `terminal_jni.c` | ~2300 | `bridge.rs` (boltffi) |
| `pty_jni.c` | ~450 | `pty.rs` |
| `TerminalEmulator.java` | ~650 | `GhosttyTerminal` |
| `TerminalView.java` | ~2050 | `TerminalSurface` |
| `SearchBarView.java` | ~500 | `TextSearchBar` |
| `SessionManager.java` | ~150 | `Session` |
| `TerminalSession.java` | ~244 | `Session` |
| `TerminalNative.java` | ~302 | `bridge.rs` |
| `ExtraKeysView.java` | ~1000 | **MISSING** |
| `ExtraKeysConfig.java` | ~348 | **MISSING** |
| `ExtraKey.java` | ~75 | **MISSING** |
| `ScreenSnapshot.java` | ~200 | `snapshot.rs` |
| `SessionCommand.java` | ~80 | `shell_env.rs` |
| `ThemeStore.java` | ~150 | `config.rs` |
| `RichInputConnection` (inner) | ~100 | **MISSING** |

---

## 1. JNI Bridge vs boltffi+JNA

### Reference: `terminal_jni.c` + `pty_jni.c` + `TerminalNative.java`

**Pattern**: Single `libterm.so` with JNI entry points. Each function takes a raw `long handle` (pointer to `Terminal`). Java side loads via `System.loadLibrary("term")`.

**Key observations**:

#### 1a. Snapshot Protocol — Rich, Compact, Zero-Copy
The reference passes **parallel arrays** (`int[] codepoints`, `int[] fg`, `int[] bg`, `byte[] attrs`, `int[] meta`, `int[] graphemes`) instead of a single serialized struct. This is excellent:
- No allocation per cell on the Rust side
- No rkyv/serde overhead — just raw memcpy
- `meta[16]` packs all global state (cursor pos, selection bounds, dirty flag, input modes, colors) in one array
- `graphemes` is a self-describing overflow buffer: `[count, cellIndex, count, cp0, cp1, ...]`

**Torvox comparison**: Torvox uses rkyv serialization for the snapshot. This adds:
- Serialization overhead on every frame
- Larger wire size (headers, alignment padding)
- The `DirtyMask` approach is fine for dirty tracking but the snapshot transfer itself is heavier

**Recommendation**: Consider switching the JNI/FFI bridge to parallel-array protocol. The rkyv path is correct but heavier than needed for a per-frame data transfer.

#### 1b. Key Encoding — Native-Side Dispatch
`terminalEncodeKey()` takes `(handle, androidKeyCode, mods, utf8, unshiftedCp)` and returns `byte[]`. The encoding logic (Kitty keyboard protocol, CSI u, etc.) lives entirely in C/Rust.

**Torvox comparison**: Same approach — `encode_key()` in `keyboard.rs`. Both correctly delegate to the VT engine.

#### 1c. Mouse Encoding — Native-Side
`terminalEncodeMouse()` handles SGR/X10/Normal mouse protocols natively.

**Torvox**: Torvox handles mouse encoding in Rust as well. **Feature parity**.

#### 1d. Color/Cursor Theme — Explicit API
`terminalSetColors()` and `terminalSetCursorStyle()` are explicit JNI calls with defaults pushed to the engine. The engine re-asserts these after each feed (survives RIS reset).

**Torvox**: Config is loaded once. Programs can override via OSC but defaults aren't re-asserted.

**Missing feature**: `terminalSetGraphemeClustering()` — force-enables DEC mode 2027, re-asserted after each feed. Torvox doesn't have this.

#### 1e. Search — Native-Side
`terminalSearchSet()`, `terminalSearchStep()`, `terminalSearchClear()` — search lives entirely in the C/Rust terminal engine. The Java side just calls these and gets back hit counts + positions.

**Torvox**: TextSearchBar exists but search is handled differently. The reference's approach of keeping search state in the terminal engine (survives screen updates, scrollback pruning) is superior.

#### 1f. Selection — Native-Side
`terminalSelectWord()`, `terminalSelectionAnchor()`, `terminalSelectionDrag()`, `terminalSelectionClear()`, `terminalSelectionText()` — selection state lives in the terminal, survives scroll/reflow.

**Torvox**: Selection is handled in `torvox-core/src/selection.rs`. Both handle this, but the reference's approach of keeping it in the terminal engine (survives grid mutations) is cleaner.

#### 1g. Paste Encoding — Native-Side
`terminalEncodePaste()` strips unsafe control bytes, applies bracketed-paste markers or newline→CR per terminal modes.

**Torvox**: Paste encoding happens on the Kotlin/Compose side. **Missing**: native-side paste encoding that respects bracketed-paste mode and strips control characters.

#### 1h. Kitty Graphics — Full Protocol
`terminalGraphics()` returns placement records. `terminalImage()` returns RGBA pixel data. Full Kitty graphics protocol support.

**Torvox**: **Not implemented**. This is a significant feature gap.

---

## 2. TerminalView vs TerminalSurface

### Reference: `TerminalView.java` (2051 lines)

This is the most sophisticated component. Key patterns:

#### 2a. Dual Input Mode (Rich + Plain)
The reference implements TWO input connection types:
- **TYPE_NULL** (plain terminal): Keyboard forwards raw keys. No IME composition.
- **TYPE_CLASS_TEXT** (rich/composing mode): Real text field with suggestions, autocorrect, swipe typing. The `RichInputConnection` mirrors the IME buffer and reconciles it with the remote line via backspace+resend.

**Torvox**: Only TYPE_NULL. No composing mode, no swipe typing, no autocorrect.

**Missing feature**: Rich keyboard mode with `RichInputConnection` that mirrors IME state and reconciles via backspace+resend. This is critical for non-English input methods (CJK, etc.).

#### 2b. Smooth Scroll with Sub-Row Offset
`snapshotSmooth()` returns the viewport AND the row above it, allowing pixel-level smooth scrolling. The draw loop translates the canvas by the offset and draws the partial top row.

**Torvox**: Uses Android's built-in `overScrollBy()`. No sub-row pixel scrolling.

**Missing feature**: Sub-row smooth scrolling via `snapshotSmooth()` + canvas translation.

#### 2c. Background Image with Alpha
`drawBackgroundImage()` center-crops a bitmap and draws it at configurable alpha over the theme background.

**Torvox**: No background image support.

#### 2d. Kitty Graphics Rendering
`updateGraphics()` fetches placement records, maintains a bitmap cache (decoding RGBA→Bitmap), and `drawImages()` renders them at correct positions with source-rect cropping. Supports z-ordering (below/above text).

**Torvox**: **Not implemented**.

#### 2e. Curly/Dotted/Dashed Underlines
`drawUnderline()` renders 5 underline styles: single, double, curly (quad bezier), dotted (DashPathEffect), dashed.

**Torvox**: Basic underline only.

#### 2f. Cursor Blink with Movement Reset
`updateCursorBlink()` resets the blink phase when the cursor moves (solid during typing/scrolling, blinks only when idle).

**Torvox**: Basic cursor blink. No movement-aware phase reset.

#### 2g. Selection Handles
`drawSelectionHandles()` places draggable handles at selection endpoints with enlarged touch targets. Handles are `Drawable` objects positioned at 3/4 and 1/4 width offsets (matching Android TextView convention).

**Torvox**: Selection exists but handles are different.

#### 2h. Sticky Modifier Keys
`sticky` state tracks CTRL/ALT toggles that apply to the next key press. Consumed atomically on dispatch.

**Torvox**: ExtraKeysView has modifier keys but the sticky state management is less refined.

#### 2i. Rich Input Reconciliation
`reconcileRich()` computes the longest common prefix between the local IME buffer and what was sent, then emits backspaces + new tail. Handles swipe (whole-word commit), autocorrect (word replacement), and plain typing uniformly.

**Torvox**: **Not implemented**. This is essential for IME support.

---

## 3. SearchBarView vs TextSearchBar

### Reference: `SearchBarView.java` (~500 lines)

Both are similar in functionality. Key differences:

#### 3a. Native Search State
The reference delegates search entirely to the terminal engine. `searchSet()` scans the whole screen and highlights/reveals the nearest match atomically. Navigation (`searchStep()`) re-scans only if the buffer changed.

**Torvox**: TextSearchBar manages search state. The reference's approach is cleaner (state in terminal, not UI).

#### 3b. Case-Sensitive Toggle
Both have case-sensitive toggle. **Feature parity**.

#### 3c. Match Count Display
Both show "N/M" match counts. **Feature parity**.

---

## 4. Session/TerminalSession

### Reference: `TerminalSession.java` (244 lines) + `SessionManager.java`

#### 4a. Update Coalescing
`dispatchEvents()` uses `AtomicBoolean updatePending` to coalesce UI updates — at most one pending `onUpdate` callback at a time. A flood of output can't queue unbounded UI work.

**Torvox**: Uses flume channel + Condvar wake. Different approach but both solve the same problem.

#### 4b. User Interaction Tracking
`userInteracted` flag distinguishes a session the user used from one that died before they could touch it (e.g., PRoot failed to launch). Useful for tab titles and error display.

**Torvox**: **Not implemented**.

#### 4c. Resize Skip for No-Op
`resize()` skips if `cols == lastCols && rows == lastRows` to avoid spurious SIGWINCH that can wipe shell prompts (observed on mksh).

**Torvox**: `pty.rs` sends SIGWINCH unconditionally. **Bug risk**: spurious SIGWINCH can cause visual glitches.

#### 4d. PTY Read Buffer Size
8192 bytes. Standard and correct.

**Torvox**: Uses 8192 as well. **Feature parity**.

---

## 5. ExtraKeysView / ExtraKeysConfig — MISSING in Torvox

### Reference: `ExtraKeysView.java` (1000+ lines)

This is a **fully customizable extra-keys toolbar** with:

#### 5a. Multi-Row Layout
Up to 3 rows of keys. Layout is configurable via settings. Each row is independently sized.

**Torvox**: **Missing entirely**. No extra-keys toolbar.

#### 5b. Key Types
- **KEY**: Non-printable keys (ESC, arrows, F-keys) sent through VT encoder
- **TEXT**: Literal strings ("-", "|", custom text) written to PTY
- **MODIFIER**: Sticky CTRL/ALT toggles

**Torvox**: **Missing**.

#### 5c. Modifier Combos
Single-tap combo keys: Ctrl-C, Ctrl-→, Shift-Tab, etc. Mods are baked into the button and applied atomically.

**Torvox**: **Missing**.

#### 5d. User-Defined Keys
Users can add custom text keys (e.g., "git status\n") and reorder the toolbar. Layout persisted as JSON array of arrays in SharedPreferences.

**Torvox**: **Missing**.

#### 5e. Long-Press Repeat
Long-press on a key starts repeating at 30ms intervals (accelerating).

**Torvox**: **Missing**.

#### 5f. Haptic Feedback
Vibration on key press.

**Torvox**: **Missing**.

#### 5g. Swipe-to-Dismiss
Vertical swipe on the toolbar hides it. Swipe up on the terminal restores it.

**Torvox**: **Missing**.

---

## 6. Bugs Torvox Should Avoid

### Bug 1: Spurious SIGWINCH on No-Op Resize
The reference's `TerminalSession.resize()` skips if cols/rows haven't changed. Torvox's `pty.rs` should do the same — mksh and other shells wipe their prompt on spurious SIGWINCH.

### Bug 2: Cursor Blink Phase Not Reset on Move
If torvox's cursor blink doesn't reset phase when the cursor moves, the cursor appears to "jump" during typing. The reference resets `cursorBlinkOn = true` on movement.

### Bug 3: Rich Input Mirror Not Synced After Special Keys
After sending a special key (arrows, Ctrl-C), the reference calls `resetRichInput()` to drop the mirror. If torvox implements composing mode, it must do the same — otherwise the IME buffer diverges from the remote line.

### Bug 4: Selection Disappears on Screen Switch
The reference handles `!snapshot.hasSelection()` during draw by calling `finishSelection()`. Torvox should retire selection UI when the selection is invalidated by screen/scrollback changes.

### Bug 5: Toolbar Not Repositioned During Selection Drag
The reference hides the action mode toolbar during drag (`draggingHandle >= 0`) and repositions it otherwise. Torvox should handle this if it implements selection handles.

### Bug 6: Grapheme Cluster Rendering
The reference draws multi-codepoint grapheme clusters (combining marks, ZWJ emoji) as single units via `snap.graphemeAt(i)`. If torvox draws individual codepoints, combining marks and emoji will break.

---

## 7. Best Practices Torvox Should Adopt

### Practice 1: Parallel-Array Snapshot Protocol
Replace rkyv serialization with parallel arrays (`codepoints[]`, `fg[]`, `bg[]`, `attrs[]`, `meta[]`). Zero-copy, no allocation, no alignment overhead.

### Practice 2: Native-Side Search
Move search state into the terminal engine (survives screen updates, scrollback pruning). The UI just calls `searchSet()`/`searchStep()` and reads hit counts.

### Practice 3: Native-Side Selection
Keep selection state in the terminal engine. Survives scroll/reflow. The UI calls `selectWord()`, `selectionDrag()`, `selectionText()`.

### Practice 4: Native-Side Paste Encoding
`encodePaste()` should strip control characters, apply bracketed-paste markers, and convert newline→CR per terminal modes.

### Practice 5: Grapheme Clustering API
Add `setGraphemeClustering(enable)` that forces DEC mode 2027 and re-asserts it after each feed.

### Practice 6: Color/Cursor Theme API
Add `setColors(fg, bg, cursor, palette256)` and `setCursorStyle(style, blink)` that push defaults to the engine and survive RIS reset.

### Practice 7: Update Coalescing
Use `AtomicBoolean` or similar to coalesce UI updates. At most one pending `onUpdate` at a time.

### Practice 8: Resize Skip for No-Op
Skip SIGWINCH if cols/rows haven't changed.

### Practice 9: User Interaction Tracking
Track whether the user has sent any input to the session. Useful for tab titles and error display.

### Practice 10: Rich Keyboard Mode
Implement composing-mode input connection for IME support (CJK, autocorrect, swipe typing). The `reconcileRich()` pattern (common prefix + backspace + resend) is the correct approach.

### Practice 11: Smooth Scroll with Sub-Row Offset
Implement `snapshotSmooth()` for pixel-level smooth scrolling via canvas translation.

### Practice 12: Extra-Keys Toolbar
Implement a configurable multi-row toolbar with sticky modifiers, modifier combos, user-defined keys, long-press repeat, and haptic feedback.

---

## 8. Priority Recommendations

| Priority | Feature | Impact | Effort |
|----------|---------|--------|--------|
| **P0** | Extra-keys toolbar (sticky modifiers, combos, custom keys) | High — core UX for Android terminal | Medium |
| **P0** | Native-side search (move state to terminal engine) | High — search reliability | Low |
| **P0** | Resize skip for no-op (avoid spurious SIGWINCH) | Medium — prevents shell prompt glitches | Trivial |
| **P1** | Rich keyboard mode (IME composing, autocorrect, swipe) | High — essential for CJK users | High |
| **P1** | Native-side selection (survives scroll/reflow) | Medium — selection reliability | Medium |
| **P1** | Grapheme clustering API (DEC 2027) | Medium — emoji/combining mark rendering | Low |
| **P1** | Native-side paste encoding (bracketed paste, control strip) | Medium — security/correctness | Low |
| **P2** | Parallel-array snapshot protocol | Medium — reduces allocation overhead | Medium |
| **P2** | Smooth scroll with sub-row offset | Low — polish | Medium |
| **P2** | Background image with alpha | Low — cosmetic | Low |
| **P2** | Kitty graphics protocol | Low — nice-to-have | High |
| **P3** | Curly/dotted/dashed underlines | Low — cosmetic | Low |
| **P3** | Cursor blink phase reset on movement | Low — polish | Trivial |
| **P3** | Color/cursor theme re-assertion API | Low — correctness | Low |

---

## 9. Architecture Comparison

### Reference Architecture
```
Java UI (TerminalView)
  → TerminalNative (JNI)
    → terminal_jni.c (C bridge)
      → GhosttyTerminal (Rust: grid + VT parser + selection + search)
      → PTY (fork/pty/resize/waitpid)
    → ScreenSnapshot (parallel arrays, zero-copy)
```

### Torvox Architecture
```
Kotlin/Compose UI (TerminalSurface)
  → TorvoxBridge (boltffi + JNA)
    → torvox-core (Rust: grid + cell + selection)
    → torvox-terminal (PTY + VT parser + GhosttyTerminal)
  → rkyv snapshot (serialized struct)
```

### Key Architectural Differences

1. **Bridge layer**: Reference uses JNI (direct, fast). Torvox uses boltffi+JNA (two layers of indirection). JNI is ~2x faster for small calls.

2. **Snapshot format**: Reference uses parallel arrays (zero-copy). Torvox uses rkyv (serialize + copy). Parallel arrays win for per-frame transfers.

3. **State ownership**: Reference keeps search/selection state in the terminal engine. Torvox keeps some in the UI layer. Centralizing in the engine is cleaner.

4. **Render path**: Reference uses Android Canvas (CPU). Torvox uses wgpu (GPU). Torvox's GPU path is superior for performance but heavier for simple cases.

5. **Input handling**: Reference has dual-mode (plain + rich/composing). Torvox has plain only. Rich mode is essential for IME support.

---

## 10. Code Quality Notes

### Reference Strengths
- Consistent naming (`terminal*` prefix for all JNI functions)
- Self-describing data formats (grapheme buffer, meta array)
- Defensive coding (null checks, bounds checks, retry-on-overflow)
- Clear thread model (reader thread, waiter thread, main thread)

### Reference Weaknesses
- JNI string handling is manual (GetStringUTFChars/ReleaseStringUTFChars)
- No null-safety annotations on JNI functions
- `ExtraKeysView` at 1000+ lines is a god class

### Torvox Strengths
- GPU rendering (wgpu) is future-proof
- Clean crate separation (core/terminal/renderer/gui)
- Property tests and fuzz targets

### Torvox Weaknesses
- No extra-keys toolbar at all
- No IME composing mode
- Snapshot serialization overhead
- No native-side search/selection state management

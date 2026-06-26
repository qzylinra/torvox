# Haven vs Torvox: Terminal Implementation Comparison

Generated: 2026-06-25

## Summary

| Category | Haven (Kotlin) | Torvox (Rust) | Verdict |
|----------|---------------|---------------|---------|
| VT Parser | Custom hand-rolled | libghostty-vt (vendored Ghostty) | Torvox: production-grade, more complete |
| Grid/Cell | Custom Grid + Cell | libghostty-vt Terminal + CellSnapshot | Torvox: delegates to Ghostty C API |
| Selection | Custom SelectionManager | torvox-core selection.rs | Both exist; compare below |
| Keyboard Input | Custom InputEngine | torvox-terminal keyboard.rs | Feature parity; Torvox slightly broader |
| OSC Handling | OscHandler | osc_handler.rs | Near-identical (Torvox credits Haven) |
| Mouse Support | MouseHandler | keyboard.rs (encode_mouse) | Torvox: encode-only; no decode |
| Themes | CustomThemeLoader | Theme (config.rs) | Torvox: 16 built-in themes + custom parser |
| Config | TerminalConfig | TerminalConfig + RenderConfig | Torvox: richer config model |
| Shell Integration | ShellIntegration (OSC 133) | Session (OSC 133) | Torvox: equivalent |
| C0/C1 Controls | Decoded in parser | control.rs enums | Torvox: explicit enums |

---

## Detailed Feature Comparison

### 1. VT Parser / Terminal Engine

**Haven** (`TerminalController.kt`, `EscapeSequenceParser.kt`, `AnsiParser.kt`):
- Hand-rolled VT500+ parser in Kotlin
- Parses CSI, OSC, DCS, ESC, SS3 sequences
- State machine with ~15 states
- Custom sequence dispatch

**Torvox** (`ghostty_terminal.rs`):
- Delegates to `libghostty-vt::Terminal` (vendored Ghostty VT parser)
- GhosttyTerminal runs on a dedicated thread with flume channel serialization
- Commands: Write, FlushAck, SetTheme, Resize, TakeSnapshot, DumpGrid, SearchInScrollback, ModeGet, etc.
- `vt_write()` converts `\n` to `\r\n` and appends `\x1b\\\x1b[0m` after each write
- PTY write-back callback for terminal responses (DECRPM, DSR, DA)

**Verdict**: Torvox has a stronger foundation. Haven's hand-rolled parser is impressive but Ghostty's parser is battle-tested. Torvox correctly delegates complex VT parsing.

---

### 2. Grid and Cell Model

**Haven** (`Grid.kt`, `Cell.kt`, `Buffer.kt`):
- `Cell`: codepoint (Int), fg/bg (Color), attributes (bold, italic, underline, strikethrough, blink, reverse, hidden, overline)
- `Grid`: rows × cols of Cells, scrollback buffer
- `Buffer`: primary + alternate screen buffers
- `DirtyLine`: tracks which lines changed (enum: Full, Partial(Vec<Int>), None)
- Cell width tracked for CJK

**Torvox** (`ghostty_terminal.rs` → `CellSnapshot`, `GridSnapshot`):
- `CellSnapshot`: codepoint (u32), fg/bg ([f32;4] linear), bold, dim, italic, underline, reverse, strikethrough, blink, hidden, overline, uri (Option<String>), semantic (SemanticContent), width (u8)
- `GridSnapshot`: rows, cols, cursor_row, cursor_col, cursor_visible, cells Vec
- `SemanticContent`: Output, Input, Prompt (for shell integration)
- URI support (OSC 8 hyperlinks) at cell level
- 256-color palette resolution at snapshot time (palette_index_to_float)
- `DumpedGrid`: visible + scrollback for full export

**Key differences**:
- Torvox stores fg/bg as `[f32; 4]` (linear float) vs Haven's `Color` (RGB bytes)
- Torvox adds `semantic` content (shell integration markers)
- Torvox adds `uri` for hyperlinks (OSC 8)
- Torvox adds `dim` attribute (though Ghostty C API doesn't expose it — hardcoded false)
- Torvox adds `width` for CJK detection at snapshot time
- Haven has explicit `Buffer` for primary/alternate screen; Torvox delegates to Ghostty's screen management

---

### 3. Selection

**Haven** (`SelectionManager.kt`, `SelectionMode.kt`):
- Modes: CHAR, WORD, LINE, BLOCK
- Selection state: anchor, head, active flag
- Word boundary detection (whitespace, punctuation, brackets)
- Block selection (rectangular)
- Visual line/word/char expansion
- Copy to clipboard integration

**Torvox** (`torvox-core/src/selection.rs`):
- Selection struct with start/end positions
- SelectionMode enum (Char, Word, Line, Block)
- Word boundary detection using unicode categories
- Block selection support

**Verdict**: Feature parity. Haven's SelectionManager is more visual/UI-oriented. Torvox's selection is data-only (no UI layer in core).

---

### 4. Keyboard Input / Encoding

**Haven** (`InputEngine.kt`, `KeyMapping.kt`):
- `InputEngine`: encodes key events to escape sequences
- Kitty keyboard protocol support
- Legacy (xterm) encoding
- Modifier tracking (Shift, Alt, Ctrl, Meta/Super)
- Cursor key application mode
- Keypad application mode
- Bracketed paste start/end
- Mouse SGR encoding (press, release, motion)
- F1-F20, Home, End, PageUp/Down, Insert, Delete
- Backspace mode (DEL 0x7F vs BS 0x08)

**Torvox** (`torvox-terminal/src/keyboard.rs`):
- `InputEngine`: identical architecture to Haven
- Kitty keyboard protocol (`encode_kitty`)
- Legacy xterm encoding (`encode_legacy`)
- Same modifier model (Shift, Alt, Ctrl, Meta via bitflags)
- Cursor key application mode (SS3 for arrows)
- Keypad application mode (flag tracked)
- Bracketed paste (encode_paste_start/end)
- Mouse SGR encoding (encode_mouse_press/release/motion)
- F1-F20, Home, End, PageUp/Down, Insert, Delete
- Backspace mode (configurable byte)
- QuickCheck property tests

**Verdict**: Near-identical. Torvox is directly inspired by Haven (credited in osc_handler.rs). Torvox has QuickCheck fuzz tests; Haven does not.

---

### 5. Mouse Support

**Haven** (`MouseHandler.kt`):
- Full mouse event decoding
- SGR mouse mode (1006)
- Normal mouse mode (1000)
- Button-event tracking (1002)
- Any-event tracking (1003)
- Mouse position → cell coordinate mapping
- Scroll wheel encoding
- Double-click word selection
- Triple-click line selection
- Touch-to-mouse emulation (Android)

**Torvox** (`torvox-terminal/src/keyboard.rs`):
- Mouse encoding only (encode_mouse_press/release/motion)
- SGR format (\x1b[<btn;x;yM/m)
- No mouse decoding (input only, not reading mouse from PTY)
- Mouse tracking mode detection via `is_mouse_tracking_active()` (modes 1000, 1002, 1003)

**Verdict**: Haven is significantly more complete for mouse handling. Torvox only encodes mouse events for sending to PTY; no decoding/receiving of mouse events from PTY.

---

### 6. OSC (Operating System Command) Handling

**Haven** (`OscHandler.kt`):
- OSC 52: Clipboard (base64 decode)
- OSC 7: CWD (current working directory)
- OSC 8: Hyperlinks (open/close)
- OSC 9: Notifications (iTerm2-style)
- OSC 0/1: Title setting (pass-through)
- Partial sequence handling across buffer boundaries
- Payload size limit

**Torvox** (`torvox-terminal/src/osc_handler.rs`):
- OSC 52: Clipboard (base64 decode) ✓
- OSC 7: CWD ✓ (pass-through to Ghostty)
- OSC 8: Hyperlinks ✓
- OSC 9: Notifications (iTerm2) ✓
- OSC 777: Notifications (rxvt-unicode-style) ✓ (Haven doesn't have this)
- Partial sequence handling ✓
- Payload size limit (MAX_PAYLOAD_BYTES = 1MB) ✓
- State machine: Ground → Esc → OscBracket → OscNumber → Payload → StEsc/PtStEsc
- Reusable buffers (no per-call allocations)

**Torvox credits Haven**: Line 2 of osc_handler.rs: "Inspired by Haven's OscHandler."

**Verdict**: Torvox is a superset. Adds OSC 777 and is written in `no_std`-friendly Rust with explicit state machine.

---

### 7. Theme System

**Haven** (`ThemeManager.kt`, `Theme.kt`, `CustomThemeLoader.kt`):
- Theme struct: name, colors (bg, fg, cursor, selection, 16 ANSI)
- `ThemeManager`: load/save themes, built-in themes
- `CustomThemeLoader`: parse .conf theme files
- Color formats: #RGB, #RRGGBB, rgb(r,g,b)
- Theme persistence (SharedPreferences)

**Torvox** (`torvox-core/src/config.rs`):
- Theme struct: name, bg, fg, cursor, 16 ANSI colors (all [u8;3])
- 16 built-in themes: Dracula, Catppuccin Mocha/Latte, Nord, Tokyo Night, Rose Pine, Gruvbox Dark/Light, Everforest, One Dark/Light, Monokai, Ayu Dark/Light, Kanagawa Wave, Night Owl
- Custom theme parser: `Theme::parse_custom()` — supports #RGB, #RRGGBB, rgb(r,g,b), named colors (red, green, blue, etc.), ansi0-ansi15 keys
- Quotes and comments supported
- `all_built_in()` returns all 16 themes
- Serde serialization/deserialization

**Verdict**: Torvox has more built-in themes (16 vs typical 5-8). Custom parser is equivalent. Haven has selection color in theme; Torvox doesn't expose selection color in Theme struct.

---

### 8. Terminal Configuration

**Haven** (`TerminalConfig.kt`):
- rows, cols, scrollback
- shell path
- font size
- backspace mode (DEL/BS)
- right alt mode (AltGr/Meta)
- clipboard mode

**Torvox** (`torvox-core/src/config.rs`):
- `TerminalConfig`: rows, cols, scrollback_lines, shell (SystemDefault/Custom), font_size_tenths, backspace_mode, right_alt_mode
- `RenderConfig`: FontConfig (family, size, line_spacing), Theme, CursorStyle
- `FontConfig`: family (default "JetBrains Mono Nerd Font"), size, line_spacing
- `CursorStyle`: Block, Bar, Underline (via cursor.rs)
- Shell: SystemDefault or Custom(String)
- Full serde roundtrip

**Verdict**: Torvox has richer config. `RenderConfig` is separate from `TerminalConfig`, allowing render-specific settings. Haven has clipboard mode; Torvox doesn't.

---

### 9. Session Management

**Haven** (`SessionManager.kt`):
- PTY spawn with shell path, rows, cols
- Environment variables setup
- Reader thread (reads PTY output)
- Writer (writes to PTY)
- Process exit detection
- Resize handling
- Cleanup on exit (SIGHUP → SIGCONT → SIGKILL)

**Torvox** (`torvox-terminal/src/session.rs`):
- PTY spawn via PtyPair::spawn(shell, rows, cols, env)
- Reader thread (8192 byte buffer, non-blocking read with 2ms sleep on WouldBlock)
- Wait thread (waitpid for child exit)
- Writer via pty.write_all()
- Resize via pty.resize() + terminal.resize()
- OSC handler integration (Clipboard, CWD, Hyperlink, Notification events)
- ShellIntegration (OSC 133: PromptStart, PromptEnd, CommandStart, CommandExecuted)
- BEL detection (AtomicBool)
- Output notification via Condvar
- Drop: SIGHUP → 50ms → SIGKILL, then join threads
- Pixel size tracking (`set_pixel_size`)

**Verdict**: Torvox is more sophisticated. ShellIntegration (OSC 133) is not in Haven's SessionManager. OSC event dispatching is integrated. Thread model is cleaner (dedicated wait thread).

---

### 10. Unicode / Character Width

**Haven** (`UnicodeUtils.kt`):
- Character width detection (0, 1, 2)
- CJK detection
- Emoji detection
- Combining character detection
- String width calculation

**Torvox** (`torvox-core/src/unicode.rs`):
- `UnicodeWidth` enum: Zero, Single, Double
- `width(char)` → UnicodeWidth
- `width_value(char)` → u8
- `string_width(str)` → u32
- `is_wide(char)` → bool
- Uses `unicode-width` crate
- `no_std` compatible (uses only unicode-width)

**Verdict**: Feature parity. Torvox is `no_std` and uses a well-tested crate. Haven's implementation is custom.

---

### 11. Control Codes

**Haven** (parsed inline in `EscapeSequenceParser.kt`):
- C0 codes decoded as part of state machine
- No separate enum

**Torvox** (`torvox-core/src/control.rs`):
- `C0` enum: Nul, Enq, Bel, Bksp, Tab, Lf, Vt, Ff, Cr, So, Si, Xon, Xoff, Esc, Other(u8)
- `C1` enum: Hts, Ri, Dcs, Osc, Sos, St, Csi, Nel, Ind
- Explicit `from_byte()` methods
- Serde + rkyv serialization

**Verdict**: Torvox is more structured. Explicit enums with serialization support for the no_std core.

---

### 12. Alternate Screen Buffer

**Haven** (`Buffer.kt`):
- `Buffer` class wrapping primary + alternate Grid
- Switch methods
- Scrollback management

**Torvox** (`ghostty_terminal.rs`):
- Delegates to `libghostty_vt::Terminal` which has built-in primary/alternate screen
- `AltScreen` command queries active screen
- Ghostty handles buffer switching internally

**Verdict**: Torvox delegates to Ghostty. Haven implements from scratch.

---

### 13. Cursor Management

**Haven** (`Cursor.kt`):
- Cursor position (row, col)
- Cursor style (BLOCK, BAR, UNDERLINE)
- Cursor visibility
- Cursor blink tracking
- Origin mode (DEC origin mode)
- Autowrap mode

**Torvox** (`ghostty_terminal.rs`):
- `cursor_x()`, `cursor_y()`, `cursor_visible()` via flume commands
- `is_cursor_enabled()` (mode 25)
- `is_origin_mode()` (mode 6, DEC)
- `is_autowrap_enabled()` (mode 7, DEC)
- `is_alt_screen_active()`
- `is_bracketed_paste_active()` (mode 2004)
- `is_mouse_tracking_active()` (modes 1000, 1002, 1003)
- `mode_get(num, kind)` — generic mode query
- `title()` — terminal title
- `cwd()` — current working directory
- Cursor style: in `RenderConfig.cursor_style` (Block, Bar, Underline)

**Verdict**: Torvox has more mode queries exposed as public API. Cursor style is in config, not dynamically queried from VT.

---

### 14. DEC Rectangle Operations

**Haven**: No explicit DEC rectangle operations.

**Torvox** (`ghostty_terminal.rs` lines 529-574):
- `dec_fill_rect(char_code, top, left, bottom, right)` — DECFRA
- `dec_erase_rect(top, left, bottom, right)` — DECERA
- `dec_change_attr_rect(sgr_seq, top, left, bottom, right)` — DECCARA
- Decomposes into primitive VT sequences since Ghostty doesn't handle CSI $ intermediates natively

**Verdict**: Torvox-only. Haven doesn't implement DEC rectangle operations.

---

### 15. Hyperlink Support

**Haven** (`OscHandler.kt`):
- OSC 8 open/close events
- URI tracking per cell

**Torvox** (`ghostty_terminal.rs` + `osc_handler.rs`):
- OSC 8 handled by osc_handler.rs → HyperlinkEvent
- CellSnapshot has `uri: Option<String>`
- `uri_at(row, col)` on GridSnapshot
- Ghostty terminal handles OSC 8 internally for cell-level URI association

**Verdict**: Torvox is more complete. URI is stored per-cell in snapshot and queryable.

---

### 16. Shell Integration

**Haven**: Not implemented in the terminal files reviewed.

**Torvox** (`session.rs` lines 18-40, 445-476):
- `ShellIntegration` enum: None, PromptStart, PromptEnd, CommandStart, CommandExecuted
- Extracts OSC 133 markers from output stream
- `poll_shell_integration()` returns and resets marker
- Used for semantic content (Input/Prompt/Output) in CellSnapshot

**Verdict**: Torvox-only. Haven doesn't have shell integration markers.

---

### 17. Scrollback Search

**Haven**: Not implemented in reviewed files.

**Torvox** (`ghostty_terminal.rs` lines 301-316, 904-917):
- `search_in_scrollback(query)` → Option<(row, col)>
- Iterates all total rows (visible + scrollback)
- Line text extraction per row

**Verdict**: Torvox-only.

---

### 18. Grid Dump / Export

**Haven**: Not a first-class feature.

**Torvox** (`ghostty_terminal.rs` lines 350-353, 588-705):
- `dump_grid()` → DumpedGrid
- Contains: rows, cols, visible (Vec<CellSnapshot>), scrollback (Vec<Vec<CellSnapshot>>)
- Full export of every cell with all attributes

**Verdict**: Torvox-only. Useful for debugging and serialization.

---

### 19. Title / Window Title

**Haven** (`OscHandler.kt`):
- OSC 0/1 pass-through (title setting)

**Torvox** (`ghostty_terminal.rs` lines 104, 347-349, 485-489):
- `title()` → String — queries terminal title from Ghostty
- OSC 0/1 handled by Ghostty internally

**Verdict**: Both support title. Torvox can query it; Haven can only pass it through.

---

### 20. Testing

**Haven**:
- Unit tests for key encoding, OSC parsing, selection
- Integration tests for session lifecycle
- No property-based testing visible

**Torvox**:
- Comprehensive unit tests for all modules
- QuickCheck property tests for keyboard encoding (`keyboard.rs`)
- `assert_invariants()` helper for snapshot validation
- Color approximation tests (`colors_approx_eq`)
- Effect flag testing (EffectFlag enum for bold/italic/underline/reverse)
- OSC handler tests covering partial sequences, payload limits, reuse
- Session tests with timeout-based drain helpers

**Verdict**: Torvox has significantly more thorough testing, including property-based tests.

---

## Bugs and Issues Found

### Haven Issues

1. **`OscHandler.kt`** — OSC 7 CWD is passed through AND an event is created (line 362-364 in torvox osc_handler test shows passthrough). Haven's test shows passthrough, which means Ghostty sees it too — potential double-handling.

2. **`InputEngine.kt`** — Kitty protocol release events return empty Vec. This is correct per spec but means release events are silently dropped. Torvox does the same.

3. **No thread safety** — Haven's Kotlin coroutines handle concurrency, but some mutable state (cursor position, selection) could race between UI and parser threads.

### Torvox Issues

1. **`ghostty_terminal.rs` line 623, 676, 846** — `data.dim = false` hardcoded because "Ghostty C API does not expose dim/faint". This means SGR 2 (dim) is parsed but not reflected in snapshots. Haven's Cell tracks dim.

2. **`ghostty_terminal.rs` line 578** — `populate_uri` is a no-op. The comment says it's for URI population but the body is empty. This means `uri_at()` on snapshots will always return None. OSC 8 hyperlinks are parsed by osc_handler but not propagated to cell-level URIs in snapshots. Haven tracks URI per-cell.

3. **`ghostty_terminal.rs` line 370** — `vt_write()` appends `\x1b\\\x1b[0m` after every write. This resets SGR attributes after every output chunk, which could interfere with programs that expect attributes to persist across writes.

4. **`session.rs` line 332** — BEL detection only checks for `0x07` in data, but doesn't check if it's inside an OSC sequence. Haven's OscHandler properly parses BEL as an OSC terminator.

5. **`keyboard.rs`** — Mouse button decoding (higher buttons like button 64+) uses `encoded |= 0x40` for press. This is correct SGR encoding but button numbers > 5 are uncommon on mobile.

---

## Architecture Comparison

### Haven Architecture
```
Android UI (Compose) 
  → TerminalController 
    → EscapeSequenceParser (state machine)
    → Grid (primary + alternate Buffer)
    → SelectionManager
    → InputEngine (keyboard encoding)
    → MouseHandler (touch → mouse events)
    → OscHandler (OSC interception)
    → ThemeManager
```

### Torvox Architecture
```
Android UI (Compose)
  → torvox-gui-android (JNI bridge)
    → torvox-renderer (wgpu GPU rendering)
      → torvox-terminal
        → Session (PTY + threads)
          → GhosttyTerminal (dedicated thread, flume channels)
            → libghostty-vt (Ghostty VT parser)
          → OscHandler (OSC interception)
          → PtyPair (fork/pty)
          → ShellEnv (environment setup)
        → InputEngine (keyboard encoding)
      → torvox-core
        → Grid, Cell, Selection (no_std)
        → Config (themes, terminal config)
        → Unicode (width detection)
        → Control (C0/C1 enums)
```

### Key Architectural Differences

| Aspect | Haven | Torvox |
|--------|-------|--------|
| Language | Kotlin (JVM) | Rust (no_std core) |
| VT Parser | Hand-rolled | Vendored Ghostty C API |
| Threading | Coroutines + channels | Dedicated threads + flume |
| Memory Safety | JVM GC | Rust ownership (no unsafe in core) |
| GPU Rendering | Canvas 2D | wgpu (Vulkan/Metal/DX12) |
| FFI | N/A (pure Kotlin) | libghostty-vt (C FFI) |
| Serialization | Parcelable | rkyv (zero-copy) |

---

## Recommendations

### For Torvox

1. **Fix `populate_uri`** — Populate URI from Ghostty's cell data so `uri_at()` works. Currently a no-op.

2. **Expose dim attribute** — Even if Ghostty C API doesn't provide it, track it from SGR 2 in the VT write stream.

3. **Add mouse decoding** — If Android touch events need to be translated from PTY mouse responses, add mouse decoding. Currently only encoding exists.

4. **Remove SGR reset in `vt_write`** — The `\x1b[0m` appended after every write may cause issues with long-running programs. Consider only resetting on explicit request.

5. **Selection color in Theme** — Haven includes selection color in its theme. Torvox's Theme struct doesn't have selection color (it's hardcoded in the renderer).

### For Haven

1. **Shell integration (OSC 133)** — Torvox's shell integration is a significant UX improvement. Haven should add it.

2. **DEC rectangle operations** — Torvox's DECFRA/DECERA/DECCARA support improves compatibility with apps like htop and tmux.

3. **Property-based testing** — Torvox's QuickCheck tests catch edge cases. Haven should adopt property testing.

4. **Grid dump/export** — Useful for debugging and testing. Torvox's `dump_grid()` is valuable.

5. **Scrollback search** — Torvox's search-in-scrollback is a useful feature Haven lacks.

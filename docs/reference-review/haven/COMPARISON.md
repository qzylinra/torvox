# Haven vs Torvox: Feature Comparison

**Date**: 2026-06-24
**Purpose**: Comprehensive feature-by-feature comparison between Haven reference project and Torvox implementation

---

## 1. Executive Summary

### What Torvox Already Does Better

- **VT Engine**: Torvox uses Ghostty's battle-tested VT parser (libghostty-vt) with zero-copy architecture, whereas Haven builds on Android's native `TermSession` with more overhead
- **GPU Rendering**: Torvox has a full wgpu Vulkan rendering pipeline with cosmic-text shaping and swash glyph rasterization; Haven uses Android Canvas/Compose 2D rendering
- **Thread Model**: Torvox has explicit 6-7 thread architecture with Condvar synchronization; Haven relies on Android's Handler/Looper pattern
- **No-Std Core**: `torvox-core` is `#![no_std]` compatible, enabling use in non-Android environments
- **Shell Integration**: Torvox has dedicated OSC 133 extraction with 4 markers (PromptStart/PromptEnd/CommandStart/CommandExecuted); Haven lacks this
- **Kitty Keyboard Protocol**: Full implementation in torvox with proper modifier encoding; Haven's keyboard handling is more basic

### What's Missing or Gaps

| Gap | Priority | Status |
|-----|----------|--------|
| USB Device Proxy (ADB/USB pass-through) | CRITICAL | Missing |
| SFTP/SCP File Transfer | HIGH | Missing |
| SSH Connection Management | HIGH | Missing |
| Sustained Bell/Notification UI | MEDIUM | Missing (event only) |
| Text Search UI (interactive find) | MEDIUM | Partial (scrollback search exists) |
| Selection Toolbar (copy/share/paste) | HIGH | Missing |
| Text Selection Modes (char/word/line) | HIGH | Partial (grid-level selection only) |

---

## 2. Feature-by-Feature Comparison Table

| # | Feature | Haven | Torvox | Status |
|---|---------|-------|--------|--------|
| 1 | VT Parser | Android TermSession | libghostty-vt | **TORVOX BETTER** |
| 2 | OSC Handling | OscHandler.kt | osc_handler.rs | **TORVOX BETTER** (stateful parser) |
| 3 | OSC 52 Clipboard | Yes | Yes | EQUAL |
| 4 | OSC 7 CWD | Yes | Yes | EQUAL |
| 5 | OSC 8 Hyperlinks | Yes | Yes | EQUAL |
| 6 | OSC 9/777 Notifications | Yes | Yes | EQUAL |
| 7 | OSC 133 Shell Integration | No | Yes | **TORVOX BETTER** |
| 8 | Mouse Mode Tracking | Yes (MouseModeTracker.kt) | Yes (mode_get checks) | EQUAL |
| 9 | Kitty Keyboard Protocol | No | Yes (keyboard.rs) | **TORVOX BETTER** |
| 10 | Legacy Keyboard Encoding | Yes | Yes | EQUAL |
| 11 | SGR Attributes | Yes | Yes (CellSnapshot) | EQUAL |
| 12 | 256-Color Palette | Yes | Yes (palette_index_to_float) | EQUAL |
| 13 | Grid Snapshot | Yes (TerminalViewModel) | Yes (GridSnapshot) | EQUAL |
| 14 | Scrollback History | Yes | Yes | EQUAL |
| 15 | Text Search (scrollback) | Yes (ViewModel) | Yes (search_in_scrollback) | EQUAL |
| 16 | Terminal Resize | Yes | Yes | EQUAL |
| 17 | Clipboard (OSC 52) | Yes (ViewModel) | Yes (Session.poll_clipboard) | EQUAL |
| 18 | BEL Detection | Yes | Yes (poll_bel) | EQUAL |
| 19 | Notification System | Yes (ViewModel) | Yes (poll_notification) | EQUAL |
| 20 | USB Device Proxy | Yes (UsbBroker) | **MISSING** | MISSING |
| 21 | SFTP File Transfer | Yes (SftpManager) | **MISSING** | MISSING |
| 22 | SSH Connection | Yes (SshManager) | **MISSING** | MISSING |
| 23 | Selection Toolbar | Yes (SelectionToolbar.kt) | **MISSING** | MISSING |
| 24 | Text Selection (word/line) | Yes (SelectionToolbar.kt) | Partial (torvox-core selection.rs) | PARTIAL |
| 25 | GPU Rendering | No (Canvas 2D) | Yes (wgpu Vulkan) | **TORVOX BETTER** |
| 26 | Font Rendering | No (Paint.getTextWidths) | Yes (cosmic-text + swash) | **TORVOX BETTER** |
| 27 | Theme Support | Yes (ViewModel) | Yes (set_theme) | EQUAL |
| 28 | Focus Events | Yes (ViewModel) | Yes (focus_event) | EQUAL |
| 29 | Bracketed Paste | Yes | Yes (encode_paste_start/end) | EQUAL |
| 30 | Alt Screen Buffer | Yes | Yes (alt_screen) | EQUAL |
| 31 | Origin Mode | Yes | Yes | EQUAL |
| 32 | Cursor Visibility | Yes | Yes | EQUAL |
| 33 | Semantic Content (prompt/input/output) | No | Yes (SemanticContent) | **TORVOX BETTER** |
| 34 | rkyv Serialization | No | Yes (snapshot.rs) | **TORVOX BETTER** |
| 35 | Rectangular Operations (DECFRA/DECERA) | No | Yes (dec_fill_rect) | **TORVOX BETTER** |

---

## 3. Detailed Analysis

### 3.1 OSC Handling

**Haven (OscHandler.kt, lines 1-411)**
- Handles OSC 52 (clipboard), OSC 7 (CWD), OSC 8 (hyperlinks), OSC 9 (notifications)
- Uses `OscHandler` class with `OscType` enum dispatch
- Processes byte-by-byte with state tracking
- Produces `OscEvent` sealed class (Clipboard, Cwd, Hyperlink, Notification)

**Torvox (osc_handler.rs, lines 1-483)**
- Handles same OSC codes (52, 7, 8, 9, 777) + OSC 777 (rxvt notifications)
- Uses byte-level state machine with 8 states
- Produces `OscEvent` enum (Clipboard, Cwd, Hyperlink, Notification)
- Handles partial sequences across buffer boundaries (line 433-444)
- Has payload size limit (1MB max) to prevent DoS
- Reuses internal buffers to avoid allocations

**Verdict**: Torvox has a more robust implementation with better edge case handling (partial sequences, payload limits, buffer reuse).

### 3.2 Terminal Session Management

**Haven (TerminalViewModel.kt, lines 1-2422)**
- Uses Android ViewModel pattern with LiveData/StateFlow
- Manages `TermSession` from Android terminal library
- Handles clipboard via `ClipboardManager`
- Manages bell notifications via `Toast` or notification channel
- Thread management via Android's `Handler`/`Looper`
- Keyboard input via `InputConnection` method calls
- Mouse tracking via `TermSession` callbacks

**Torvox (session.rs, lines 1-603)**
- Uses explicit thread spawning (reader thread, wait thread)
- Manages `GhosttyTerminal` (dedicated terminal thread)
- Uses flume channels for inter-thread communication
- Uses Condvar for output notification
- Handles OSC events in `process_output()` (lines 328-385)
- Clean shutdown with SIGHUP → SIGCONT → SIGKILL sequence (lines 478-497)

**Verdict**: Torvox has a more explicit and predictable thread model; Haven benefits from Android's lifecycle management.

### 3.3 Ghostty Terminal Wrapper

**Torvox (ghostty_terminal.rs, lines 1-1516+)**
- Thread-safe wrapper using `Sender<Command>` (no unsafe Send/Sync)
- Dedicated terminal thread processes commands via flume channel
- Grid snapshot with cell-level detail (fg, bg, bold, italic, etc.)
- Semantic content detection (Prompt/Input/Output)
- Scrollback search capability
- DumpGrid for full terminal state export
- Rectangle operations (DECFRA, DECERA, DECCARA)
- URI/hyperlink tracking per cell

**Haven (HavenTerminal.kt, lines 1-102)**
- Wraps Android `TermSession`
- Provides cursor position, title, CWD queries
- Mouse tracking mode checks

**Verdict**: Torvox's `GhosttyTerminal` is significantly more capable with thread-safe snapshot architecture and semantic content tracking.

### 3.4 Keyboard Input

**Haven (HavenKeyboardMode.kt, lines 1-142)**
- Tracks keyboard mode (normal/application)
- Uses `TermSession` for keyboard encoding
- Basic modifier support

**Torvox (keyboard.rs, lines 1-1500+)**
- Full `InputEngine` with Kitty keyboard protocol support
- Legacy xterm encoding fallback
- Modifier tracking (Shift, Alt, Ctrl, Meta)
- Cursor key application mode
- Keypad application mode
- Configurable backspace byte (0x7F or 0x08)
- Mouse SGR encoding (press/release/motion)
- 100+ unit tests covering all key combinations

**Verdict**: Torvox has a significantly more complete keyboard input system with Kitty protocol support and extensive test coverage.

### 3.5 Mouse Mode Tracking

**Haven (MouseModeTracker.kt, lines 1-142)**
- Tracks mouse mode state (normal, button, motion, SGR, URXVT)
- Provides coordinate conversion
- Returns encoded mouse event data

**Torvox (ghostty_terminal.rs, lines 461-463)**
- Uses `mode_get(1000, 0) || mode_get(1002, 0) || mode_get(1003, 0)` for mouse tracking detection
- Delegates to Ghostty VT for actual mode handling

**Verdict**: Haven's explicit `MouseModeTracker` is more structured; Torvox's approach is simpler but relies on Ghostty's built-in mode handling.

### 3.6 Selection and Text Interaction

**Haven (SelectionToolbar.kt, lines 1-606)**
- Full selection toolbar with Copy, Share, Paste, Select All
- Text selection modes (character, word, line, block)
- Selection anchoring and range management
- URL detection and opening
- Search within selection
- Clipboard integration

**Torvox (torvox-core/src/selection.rs)**
- Selection modes (char/word/line/block) defined
- Selection state management
- No UI toolbar implementation

**Verdict**: Haven has a complete selection UI; Torvox has the data model but lacks the UI layer.

### 3.7 USB Device Proxy

**Haven (UsbBroker.kt, UsbProxy.kt, etc.)**
- USB device discovery and ADB pass-through
- Proxy communication over USB
- Device permission management
- File transfer capability

**Torvox**: No USB functionality.

**Verdict**: This is a Haven-specific feature for Android USB devices. **MISSING** in Torvox.

### 3.8 SFTP/SCP File Transfer

**Haven (SftpManager.kt, etc.)**
- SFTP connection management
- File upload/download
- Directory browsing
- Progress tracking

**Torvox**: No SFTP functionality.

**Verdict**: **MISSING** in Torvox.

### 3.9 SSH Connection Management

**Haven (SshManager.kt, etc.)**
- SSH connection with key/password auth
- Connection pooling
- Session management
- Port forwarding

**Torvox**: No SSH functionality (SSH features handled externally).

**Verdict**: **MISSING** in Torvox.

---

## 4. Implementation Roadmap for Missing Features

### Phase 1: Core Terminal (Complete)
- [x] VT Parser (libghostty-vt)
- [x] OSC Handling (52, 7, 8, 9, 777)
- [x] OSC 133 Shell Integration
- [x] Kitty Keyboard Protocol
- [x] Mouse SGR Encoding
- [x] Thread-safe Terminal Snapshot
- [x] Semantic Content Detection
- [x] GPU Rendering (wgpu)

### Phase 2: Text Interaction (In Progress)
- [ ] Selection Toolbar UI (Compose)
- [ ] Copy/Share/Paste actions
- [ ] URL detection in selection
- [ ] Search in scrollback UI

### Phase 3: Network Features (Future)
- [ ] SSH Connection Manager
- [ ] SFTP File Transfer
- [ ] USB Device Proxy (if needed)

---

## 5. Recommendations

### What to Implement from Haven

1. **Selection Toolbar UI** (HIGH)
   - Haven's `SelectionToolbar.kt` provides a good reference
   - Torvox has the data model (`selection.rs`) but needs the Compose UI layer
   - Priority: HIGH — user-facing feature

2. **Sustained Bell Notification** (MEDIUM)
   - Haven shows a visual indicator for BEL
   - Torvox has `poll_bel()` but no UI feedback
   - Simple Compose overlay would suffice

3. **URL Detection** (LOW)
   - Haven's selection toolbar detects URLs
   - Torvox has OSC 8 hyperlink support but no click handling
   - Could use existing `uri_at()` from `GridSnapshot`

### What to Skip from Haven

1. **USB Device Proxy** — Haven-specific, not needed for Torvox's scope
2. **SFTP/SCP** — Could be added as a separate feature module later
3. **SSH Connection Manager** — Out of scope for terminal emulator core

### What Torvox Does Better (Don't Change)

1. **OSC Handler** — More robust with partial sequence handling
2. **Keyboard Input** — Kitty protocol support is superior
3. **Thread Model** — Explicit architecture is more predictable
4. **GPU Rendering** — wgpu + cosmic-text is state-of-art
5. **Shell Integration** — OSC 133 markers are unique to Torvox
6. **Semantic Content** — Prompt/Input/Output detection is unique
7. **rkyv Serialization** — Enables efficient cross-thread data transfer

---

## Appendix: Source File References

| Haven File | Lines | Torvox Equivalent |
|------------|-------|-------------------|
| OscHandler.kt | 411 | torvox-terminal/src/osc_handler.rs |
| TerminalViewModel.kt | 2422 | torvox-terminal/src/session.rs |
| TerminalScreen.kt | 1613 | torvox-renderer/src/gpu.rs |
| SelectionToolbar.kt | 606 | torvox-core/src/selection.rs (partial) |
| HavenTerminal.kt | 102 | torvox-terminal/src/ghostty_terminal.rs |
| MouseModeTracker.kt | 142 | torvox-terminal/src/keyboard.rs |
| HavenKeyboardMode.kt | 142 | torvox-terminal/src/keyboard.rs |
| TerminalComposable.kt | - | torvox-renderer/src/gpu.rs |
| USB features | - | Not implemented |
| SFTP features | - | Not implemented |
| SSH features | - | Not implemented |

---

**Conclusion**: Torvox is architecturally superior in core terminal emulation. The main gaps are UI-level features (selection toolbar) and network features (SSH/SFTP/USB). The comparison shows Torvox's Rust-based design with Ghostty VT parser provides better performance and correctness than Haven's Kotlin/Android-based implementation.

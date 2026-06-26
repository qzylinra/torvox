# Termux-App Deep Source Review

**Reviewed project:** Termux (https://github.com/termux/termux-app)
**Reviewed against:** Torvox (https://github.com/anomalyco/torvox)
**Date:** 2026-06-24

---

## Summary

| Metric | Value |
|--------|-------|
| Total files reviewed | 28/28 key source files |
| Files in project | 206 total Java/C/Shell |
| Key features torvox is missing | 12 |
| Bugs torvox should avoid | 8 |
| Best practices torvox should adopt | 15 |

### Key Features Torvox is Missing

1. **Clipboard integration (OSC 52 + native)** — Termux has `onCopyTextToClipboard`/`onPasteTextFromClipboard` wired through `TerminalOutput` → `TerminalSessionClient` → Android `ClipboardManager`. Torvox has no clipboard support.
2. **Bell/notification handling** — Termux supports vibrate, beep, and ignore modes via `BellHandler` and `SoundPool`. Torvox has no bell support.
3. **Session persistence** — Termux stores current session handle in `SharedPreferences` and restores on restart. Torvox does not persist session state.
4. **Extra keys view** — Termux has a fully configurable extra keys bar with CTRL/ALT/SHIFT/FN special buttons, arrow keys, tab, escape, etc. Torvox has a modifier bar but no extra keys system.
5. **Terminal toolbar ViewPager** — Termux has a swipeable toolbar with extra keys page + text input page. Torvox has no text input fallback.
6. **Volume key as Ctrl/Fn** — Termux maps volume down→Ctrl, volume up→Fn for soft keyboard users. Torvox has no volume key mapping.
7. **Wake/WiFi lock** — Termux acquires `PowerManager.WakeLock` and `WifiManager.WifiLock` to keep sessions alive. Torvox has no such mechanism.
8. **Storage symlinks** — Termux creates `~/storage/shared`, `~/storage/downloads`, etc. via `Os.symlink()`. Torvox has no storage integration.
9. **Package management / bootstrap installer** — Termux extracts bootstrap zip from native library, creates prefix with symlinks. Torvox has `BootstrapInstaller` but lacks the robust error recovery and retry dialog.
10. **URL detection and opening** — Termux extracts URLs from transcript text and allows tap-to-open or long-press-to-copy. Torvox has `UrlDetector` but less integration.
11. **Context menu** — Termux has a full context menu with "Select URL", "Share transcript", "Share selected text", "Reset terminal", "Kill process", "Report issue", etc. Torvox has none.
12. **Keyboard shortcuts** — Termux has configurable `Ctrl+Alt+N/P/C/R/1-9` shortcuts for session management. Torvox has basic keyboard handling.

### Bugs Torvox Should Avoid

1. **`System.exit(1)` in `wrapFileDescriptor`** (`TerminalSession.java:332`) — Termux calls `System.exit(1)` if reflection to wrap a file descriptor fails. This is an anti-pattern; it should fail gracefully instead of killing the process.
2. **Thread safety in `ByteQueue`** — Uses `synchronized` + `wait()`/`notify()` which is correct but can cause priority inversion. Torvox should use lock-free SPSC queues (which it already does with `flume`).
3. **Handler leak** — `MainThreadHandler` in `TerminalSession.java:337` is an inner class holding a reference to the outer class. Android lint flags this as a potential memory leak. Torvox should avoid this pattern.
4. **Reflection for `FileDescriptor`** — `TerminalSession.java:317-333` uses reflection to set the `descriptor` field on `FileDescriptor`. This is fragile across Android versions and breaks on some OEM ROMs. Torvox's Rust-native PTY avoids this entirely.
5. **`clearenv()` + `putenv()` in JNI** — `termux.c:98-99` clears the environment and rebuilds it, which is unsafe in multi-threaded programs. Torvox's `ShellEnv` approach is cleaner.
6. **No bounds checking on `mArgs` array** — `TerminalEmulator.java:184` uses a fixed-size `int[32]` array with `mArgIndex` that could overflow if a malicious sequence sends >32 parameters. The code does clamp `mArgIndex` in some places but not all.
7. **`short` for `mSpaceUsed`** — `TerminalRow.java:46` uses `short` (max 32767) for tracking character usage, which can overflow for long lines with many combining characters. Torvox should use `u32` or `usize`.
8. **No timeout on `ByteQueue.read()`** — The blocking read in `ByteQueue` has no timeout, so a stuck process could block the reader thread indefinitely. Torvox's `flume` channels have better timeout support.

### Best Practices Torvox Should Adopt

1. **Separate terminal emulator from view** — Termux cleanly separates `TerminalEmulator` (data model + VT parsing) from `TerminalView` (rendering) from `TerminalSession` (process management). Torvox follows this pattern well already.
2. **Circular buffer for scrollback** — Termux uses a circular buffer (`TerminalBuffer`) with `externalToInternalRow()` mapping. Torvox uses `VecDeque` which is similar but less optimized for random access.
3. **TextStyle encoding in `long`** — Termux packs fg color (24-bit), bg color (24-bit), and effect flags (11 bits) into a single `long`. This is very cache-friendly. Torvox uses separate `Color` + `Attrs` structs which are larger.
4. **`WcWidth` for Unicode width** — Termux bundles a comprehensive `WcWidth` implementation. Torvox uses `unicode-width` crate which is equivalent.
5. **`TerminalSessionClient` interface pattern** — Termux defines a clean callback interface between session and client. Torvox should ensure its equivalent is equally clean.
6. **Foreground service for session persistence** — Termux runs as a foreground service with a notification, keeping sessions alive. Torvox has `TerminalForegroundService` which should be verified to match.
7. **Configuration reload via broadcast** — Termux listens for `ACTION_RELOAD_STYLE` broadcasts to reload properties without restart. Torvox should support similar hot-reload.
8. **Session rename** — Termux allows long-press on session in drawer to rename. This is a good UX pattern Torvox should adopt.
9. **Max session limit** — Termux limits to 8 sessions (`MAX_SESSIONS`). This prevents resource exhaustion. Torvox should have similar limits.
10. **Crash report notification** — Termux sends crash notifications with transcript and device info. Torvox should have similar diagnostics.
11. **Font loading from file** — Termux loads custom fonts from `$HOME/.termux/font.ttf`. Torvox supports this via `FontManager`.
12. **Color scheme from file** — Termux loads colors from `$HOME/.termux/colors.properties`. Torvox has theme support but should verify similar file-based customization.
13. **`TermuxInstaller` bootstrap pattern** — The staging directory + atomic rename pattern for bootstrap installation is robust. Torvox's installer should be equally robust.
14. **`onTextChanged` visibility check** — Termux checks `mActivity.isVisible()` before updating the view, avoiding unnecessary work when in background. Torvox should do the same.
15. **Context menu for URL selection** — Termux's "Select URL" dialog with tap-to-copy and long-press-to-open is excellent UX. Torvox should implement similar.

---

## Detailed File Analysis

### Core Terminal Emulator (`terminal-emulator/`)

#### `TerminalSession.java` (373 lines)
**Purpose:** Manages a single terminal session: PTY creation, I/O threads, process lifecycle.

**Key patterns:**
- Three dedicated threads: `TermSessionInputReader` (PTY→emulator), `TermSessionOutputWriter` (user→PTY), `TermSessionWaiter` (waitpid)
- `ByteQueue` (64KB read, 4KB write) for producer-consumer I/O with `synchronized` wait/notify
- `MainThreadHandler` processes input on Android main thread
- `updateSize()` either initializes emulator or resizes via `JNI.setPtyWindowSize() + emulator.resize()`
- `writeCodePoint()` handles UTF-8 encoding with escape prepending
- `finishIfRunning()` sends `SIGKILL` via `Os.kill()`
- `getCwd()` reads `/proc/{pid}/cwd` symlink

**Anti-patterns to avoid:**
- `System.exit(1)` on reflection failure (line 332)
- Inner `Handler` class causing potential memory leak
- Reflection-based `FileDescriptor` wrapping

**Torvox comparison:** Torvox uses Rust-native PTY via `nix` crate, avoiding all JNI/reflection issues. Torvox's `Session` struct is cleaner.

---

#### `TerminalEmulator.java` (~3500 lines)
**Purpose:** Full VT100/xterm emulator. The largest and most complex file.

**Key patterns:**
- State machine with 24+ escape states (`ESC_NONE`, `ESC_CSI`, `ESC_OSC`, etc.)
- `DECSET_BIT_*` flags for terminal modes (cursor keys, mouse tracking, bracketed paste, etc.)
- Main/alternate screen buffer switching
- OSC 4 (color palette), OSC 52 (clipboard), OSC 7 (working directory), OSC 112 (reset cursor color)
- DCS for device control strings (DECRQSS, termcap/terminfo responses)
- Rectangular area operations: DECCRA, DECSERA, DECFRA, DECERA, DECCARA, DECRARA
- Mouse event sending in both normal and SGR protocols
- Tab stop management
- Line drawing character sets (G0/G1)

**Key features torvox should verify it has:**
- Bracketed paste mode (DECSET 2004)
- SGR mouse protocol (DECSET 1006)
- Cursor blink control
- Origin mode (DECOM)
- Auto-wrap (DECAWM)
- Insert mode
- Reverse video (DECSCUSR)
- Tab stops

**Torvox comparison:** Torvox uses `libghostty-vt` (Ghostty's VT parser) which is more modern and comprehensive than Termux's hand-rolled parser. This is an advantage for Torvox.

---

#### `TerminalBuffer.java` (497 lines)
**Purpose:** Circular buffer of `TerminalRow` for screen + scrollback.

**Key patterns:**
- `externalToInternalRow()` maps logical screen coordinates to circular buffer indices
- `getSelectedText()` handles line-wrapped text selection with join modes
- `getWordAtLocation()` for double-tap word selection
- `scrollDownOneLine()` with margin support
- `blockCopy()` and `blockSet()` for rectangular operations
- `resize()` with cursor preservation

**Torvox comparison:** Torvox's `Grid` uses `VecDeque<Line>` for scrollback and `Vec<Line>` for screen, which is similar but the `externalToInternalRow` pattern is more explicit in Termux.

---

#### `TerminalRow.java` (283 lines)
**Purpose:** Single row of terminal cells with text + style arrays.

**Key patterns:**
- `char[]` for text (Java chars, not Unicode code points) + `long[]` for styles
- `mSpaceUsed` (short) tracks actual character usage
- `mLineWrap` flag per row
- `mHasNonOneWidthOrSurrogateChars` for fast path optimization
- `findStartOfColumn()` maps column index to char array index (handles wide chars + combining)
- `setChar()` handles wide character replacement, combining character accumulation
- `MAX_COMBINING_CHARACTERS_PER_COLUMN = 15` limit
- `SPARE_CAPACITY_FACTOR = 1.5f` for text array growth

**Anti-patterns:**
- `short mSpaceUsed` can overflow for very long lines
- Complex `setChar()` logic with multiple edge cases

**Torvox comparison:** Torvox's `Cell` struct stores one char + attrs per cell, which is simpler but less memory-efficient for combining characters. Termux's approach of storing raw chars with separate styles is more compact.

---

#### `KeyHandler.java` (373 lines)
**Purpose:** Maps Android key codes + modifiers to terminal escape sequences.

**Key patterns:**
- Comprehensive key map: arrows, function keys F1-F12, numpad, home/end, page up/down, insert, delete
- `transformForModifiers()` generates CSI sequences with modifier parameters (e.g., `\033[1;2A` for Shift+Up)
- Termcap name → keycode mapping for DECRQSS responses
- Application cursor mode vs normal mode
- Application keypad mode vs numeric keypad mode
- NumLock state handling for numpad keys

**Torvox comparison:** Torvox's `keyboard.rs` implements Kitty keyboard protocol which is more modern than Termux's legacy approach. However, Termux's termcap mapping for DECRQSS is something Torvox should verify it handles.

---

#### `TextStyle.java` (90 lines)
**Purpose:** Encodes cell attributes into a 64-bit long.

**Bit layout:**
- Bits 0-10: Effect flags (bold, italic, underline, blink, inverse, invisible, strikethrough, protected, dim, truecolor fg, truecolor bg)
- Bits 16-39: Background color (9-bit index or 24-bit truecolor)
- Bits 40-63: Foreground color (9-bit index or 24-bit truecolor)

**Torvox comparison:** Torvox uses separate `Color` (4 bytes) + `Attrs` (14 bools ≈ 14 bytes) per cell = ~18 bytes. Termux's `long` approach = 8 bytes. Termux is more memory-efficient.

---

#### `TerminalColors.java` (96 lines)
**Purpose:** Current terminal color palette with parsing and brightness calculation.

**Key patterns:**
- 259 colors (256 indexed + foreground/background/cursor)
- `parse()` handles `#RGB`, `#RRGGBB`, `#RRRGGGBBB`, `#RRRRGGGGBBBB`, `rgb:` formats
- `getPerceivedBrightnessOfColor()` for adaptive cursor color

**Torvox comparison:** Torvox has similar color support in `ThemeConfig`.

---

#### `JNI.java` (41 lines)
**Purpose:** Native method declarations for PTY management.

**Methods:**
- `createSubprocess()` — fork + exec with PTY
- `setPtyWindowSize()` — `ioctl(TIOCSWINSZ)`
- `waitFor()` — `waitpid()`
- `close()` — `close()`

**Torvox comparison:** Torvox uses Rust `nix` crate for all PTY operations, which is safer and more portable.

---

#### `termux.c` (218 lines)
**Purpose:** JNI implementation for PTY creation.

**Key patterns:**
- Opens `/dev/ptmx` with `O_RDWR | O_CLOEXEC`
- `grantpt()` + `unlockpt()` + `ptsname_r()` for PTY setup
- Sets `IUTF8` flag, disables `IXON`/`IXOFF`
- `fork()` + `setsid()` + `dup2()` for child process
- Closes all fds > 2 via `/proc/self/fd` iteration
- `clearenv()` + `putenv()` for environment setup
- `chdir()` + `execvp()` for process execution

**Anti-patterns:**
- `clearenv()` is not thread-safe
- No error handling for `dup2()` failures
- `ReleaseStringUTFChars` called with wrong string on line 165 (`cmd` instead of `cwd`)

**Torvox comparison:** Torvox's Rust implementation avoids all these C pitfalls.

---

### Terminal View (`terminal-view/`)

#### `TerminalView.java` (~1500 lines)
**Purpose:** Android View that displays and interacts with a TerminalSession.

**Key patterns:**
- Custom `InputConnection` for IME support with `commitText()`, `deleteSurroundingText()`, `finishComposingText()`
- `GestureAndScaleRecognizer` for tap, scroll, fling, scale, long press
- Mouse event reporting for touch and mouse input
- Text selection with cursor controllers
- Cursor blinker with configurable rate
- AutoFill support (API 26+)
- Scrollbar integration
- `onScreenUpdated()` with scroll counter management
- Scale factor for pinch-to-zoom font resizing

**Key features:**
- `onCreateInputConnection()` with `TYPE_NULL` for raw keyboard input
- `sendTextToTerminal()` handles surrogate pairs and Ctrl key combinations
- Fling scrolling with velocity-based animation
- Accessibility support via `setContentDescription()`

**Torvox comparison:** Torvox uses Compose + `TextureView`/`SurfaceView` instead of custom `View`. The IME integration pattern is something Torvox should verify it handles correctly.

---

#### `TerminalRenderer.java` (249 lines)
**Purpose:** Renders terminal content to Android Canvas.

**Key patterns:**
- Pre-computed ASCII character widths for fast measurement
- Run-length encoding of style changes for efficient drawing
- Cursor rendering with block/underline/bar styles
- Selection highlighting
- Bold→bright color mapping for indexed colors
- Dim color calculation (2/3 intensity)
- Font width mismatch detection and scaling
- `drawTextRun()` with fake bold, underline, italic, strikethrough

**Torvox comparison:** Torvox uses wgpu GPU rendering which is fundamentally different and more modern. Termux's Canvas-based approach is simpler but slower.

---

### App Module (`app/`)

#### `TermuxActivity.java` (~1013 lines)
**Purpose:** Main activity hosting the terminal.

**Key patterns:**
- `ServiceConnection` for binding to `TermuxService`
- `DrawerLayout` for session list sidebar
- `ViewPager` for terminal toolbar (extra keys + text input)
- Context menu with URL selection, share, reset, kill, report
- Broadcast receiver for style reload
- Window insets handling for navigation bar
- Full screen mode support

**Torvox comparison:** Torvox uses Compose Navigation with `MainActivity` → `TerminalScreen`. The drawer pattern is similar to Torvox's `SessionDrawer`.

---

#### `TermuxService.java` (~959 lines)
**Purpose:** Foreground service managing sessions and background tasks.

**Key patterns:**
- `LocalBinder` for in-process binding
- Foreground notification with session count
- Wake/WiFi lock management
- `ACTION_SERVICE_EXECUTE` for plugin command execution
- `killAllTermuxExecutionCommands()` with plugin result handling
- `TermuxShellManager` for session/task/pending command lists
- `START_NOT_STICKY` return policy

**Torvox comparison:** Torvox has `TerminalForegroundService` which should serve the same purpose.

---

#### `TermuxTerminalSessionActivityClient.java` (528 lines)
**Purpose:** Activity-level callbacks from terminal sessions.

**Key patterns:**
- `MAX_SESSIONS = 8` limit
- `SoundPool` for bell sound with lazy loading
- Bell behavior: vibrate/beep/ignore
- Session finish handling with auto-close on exit 0 or 130
- Font/color loading from `$HOME/.termux/`
- Background color update from cursor color
- Session rename with `TextInputDialogUtils`

**Torvox comparison:** Torvox's equivalent should handle bell, session limits, and font/color loading similarly.

---

#### `TermuxTerminalViewClient.java` (802 lines)
**Purpose:** View-level callbacks and keyboard handling.

**Key patterns:**
- Volume key → Ctrl/Fn mapping (when no hardware keyboard)
- Fn key layer: WASD→arrows, PN→page up/down, T→tab, 1-9→F1-F9
- Configurable `Ctrl+Alt` shortcuts for session management
- Soft keyboard state management (enable/disable/show/hide)
- Cursor blinker state management
- URL selection dialog with tap-to-copy, long-press-to-open
- Share transcript/selected text
- Report issue with debug info

**Torvox comparison:** Torvox has `ModifierBar` and `TerminalInputEncoder` but lacks the Fn key layer and configurable shortcuts.

---

#### `TermuxInstaller.java` (386 lines)
**Purpose:** Bootstrap package installation.

**Key patterns:**
- Staging directory + atomic rename pattern
- Zip extraction from native library
- SYMLINKS.txt parsing for symlink creation
- Permission setting for executables
- Error dialog with retry option
- Primary user check
- Storage symlink setup (`~/storage/shared`, etc.)

**Torvox comparison:** Torvox has `BootstrapInstaller` + `BootstrapDownloader` + `BootstrapOrchestrator` which is more modular. Termux's approach of embedding the zip in a native library is simpler but less flexible.

---

#### `TerminalToolbarViewPager.java` (117 lines)
**Purpose:** Swipeable toolbar with extra keys + text input.

**Key patterns:**
- Page 0: `ExtraKeysView` with configurable button text case
- Page 1: `EditText` for text input with `OnEditorActionListener`
- Focus management between pages
- `FullScreenWorkAround` for extra keys positioning

**Torvox comparison:** Torvox has no text input fallback toolbar.

---

### Shared Libraries

#### `ByteQueue.java` (108 lines)
**Purpose:** SPSC circular byte buffer for terminal I/O.

**Key patterns:**
- `synchronized` blocks with `wait()`/`notify()`
- Separate read/write methods with blocking option
- `close()` to signal shutdown

**Torvox comparison:** Torvox uses `flume` channels which are lock-free and more efficient.

---

#### `WcWidth.java` (~566 lines)
**Purpose:** Unicode character width calculation (Unicode 15).

**Key patterns:**
- Zero-width combining character tables
- Double-width CJK/emoji tables
- `width()` method returns 0, 1, or 2
- `zeroWidthCharsCount()` for combining character counting

**Torvox comparison:** Torvox uses `unicode-width` crate which is equivalent.

---

## Architecture Comparison

| Aspect | Termux | Torvox |
|--------|--------|--------|
| Language | Java + C (JNI) | Rust + Kotlin |
| VT Parser | Hand-rolled `TerminalEmulator` | `libghostty-vt` (Ghostty) |
| Rendering | Android Canvas (`TerminalRenderer`) | wgpu GPU (`torvox-renderer`) |
| PTY | C JNI (`termux.c`) | Rust `nix` crate |
| I/O Channels | `ByteQueue` (synchronized) | `flume` (lock-free) |
| Text Encoding | `char[]` per row | `Cell` per position |
| Style Encoding | Packed `long` (8 bytes) | `Color` + `Attrs` (18 bytes) |
| Scrollback | Circular buffer in `TerminalBuffer` | `VecDeque<Line>` in `Grid` |
| UI Framework | Android Views + DrawerLayout | Jetpack Compose |
| Session Service | Foreground `Service` | Foreground `Service` |
| Configuration | `SharedPreferences` + properties files | DataStore + TOML |

---

## Priority Recommendations

### P0 (Critical)
1. **Add clipboard support** — OSC 52 + native clipboard integration. Essential for terminal usability.
2. **Add bell/notification handling** — At minimum vibrate or beep on BEL character.
3. **Verify session persistence** — Ensure `TerminalForegroundService` keeps sessions alive across app restarts.

### P1 (High)
4. **Add extra keys view** — Configurable buttons for CTRL/ALT/SHIFT/FN/arrows/tab/escape. Critical for soft keyboard users.
5. **Add volume key mapping** — Volume down→Ctrl, volume up→Fn when no hardware keyboard.
6. **Add wake/WiFi lock** — Optional but important for long-running tasks.
7. **Add storage symlinks** — `~/storage/shared` etc. for file access.

### P2 (Medium)
8. **Add context menu** — URL selection, share, reset, kill, report.
9. **Add session rename** — Long-press in drawer.
10. **Add configurable keyboard shortcuts** — Ctrl+Alt+N/P/C/R for session management.
11. **Add font/color file loading** — `$HOME/.termux/font.ttf` and `$HOME/.termux/colors.properties`.
12. **Add max session limit** — Prevent resource exhaustion.

### P3 (Low)
13. **Add crash report notification** — With transcript and device info.
14. **Add text input toolbar** — Fallback for when extra keys are insufficient.
15. **Add URL detection and opening** — Tap to open, long-press to copy.

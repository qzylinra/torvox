# Torvox Reference-Project Comparison

> Authoritative side-by-side comparison of **torvox** against four reference terminals:
> **Haven** (Kotlin/termlib), **termux-app** (`github-releases/v0.119.0-beta.3`),
> **ghostty-android-terminal** (libghostty-vt demo), and **ghostling** (minimal
> libghostty-vt C demo). Produced for the deep-overhaul effort. No `rs`/`kt` code is
> modified by this document — it is a design artifact. All findings are anchored with
> `file:line` citations from **both** torvox and the references.

Conventions used below:

- `torvox:…` = `/home/runner/work/kudzu/kudzu/repositories/torvox/…`
- `haven:…` = `/tmp/opencode/reference-repos/Haven/…`
- `termux:…` = `/tmp/opencode/reference-repos/termux-app/…`
- `ghostty-android:…` = `/tmp/opencode/reference-repos/ghostty-android-terminal/…`
- `ghostling:…` = `/tmp/opencode/reference-repos/ghostling/…`

---

## 1. Architecture Overview

torvox is a **Rust-native** stack (Ghostty VT parser, wgpu renderer, Kotlin/Compose UI)
whereas the three JVM references are conventional Android apps. The structural
differences matter most for the IME and lifecycle sections.

| Axis | torvox | Haven | termux-app | ghostty-android |
|---|---|---|---|---|
| Core emulator | libghostty-vt (Rust, vendored) `torvox-terminal/src/ghostty_terminal.rs` | ConnectBot `termlib` (C/Java, submodule) via `HavenTerminal.kt` | `TerminalEmulator.java` (hand-rolled Java) | libghostty-vt (prebuilt `.a`) `terminal_jni.c:47-96` |
| Thread model | 6–7 threads: PTY reader, input writer, process waiter, render thread (AGENTS.md "Thread Model") | reader thread in `LocalSession.kt:19-194`; emulation on main thread | dedicated named threads: `TermSessionInputReader[pid=…]` `TerminalSession.java:133,150,166` | single `TermCtx` per terminal, serialized under one monitor `TerminalEmulator.java:6-9` |
| Crate/package structure | strictly one-way: `libghostty-vt` → `torvox-core` → `torvox-terminal` → `torvox-renderer` → `torvox-gui-android` → `android/app` (AGENTS.md "Architecture") | `core/terminal-haven`, `feature/terminal`, `core/wayland` | `terminal-emulator` (Java+JNI), `terminal-view`, `app` | `app/src/main/{cpp,java}` flat |
| Session ownership | long-lived bridge in `torvox-gui-android`; `TorvoxRuntime.kt` owns `CoroutineScope(SupervisorJob()+IO)` (exploration-kotlin:213) | `SshTerminalEmulatorOwner` is a `@Singleton` surviving ViewModel teardown `SshTerminalEmulatorOwner.kt:30-57` | `TermuxService` foreground service `TermuxActivity.java:260` | process-wide `SessionManager` singleton `docs/architecture.md:420-425` |
| Data model | `#![no_std]` `Grid`/`Cell`/`DirtyMask` + rkyv wire `torvox-core/src/lib.rs:45` | three hand-rolled state machines (`OscHandler`, `MouseModeTracker`, `ScrollbackRing`) | `TerminalBuffer` of `TerminalRow`s | `RenderState` flattened to `int[]` in `terminalSnapshot` `terminal_jni.c:507-827` |
| Rendering | wgpu/Vulkan GPU `torvox-renderer/src/gpu.rs` | Canvas `TerminalView` composable (`HavenTerminal.kt`) | Canvas `TerminalRenderer.java:19` | Canvas `TerminalView.onDraw` `TerminalView.java:187-216` |

**Takeaway:** torvox's separation (Rust core, no emulation on the main thread, named-role
threads) is *more* principled than Haven/termux. Its weaknesses are not architectural —
they are in the **Kotlin IME/keyboard layer** (section 2) and a handful of robustness gaps
(section 8). The renderer (section 4) and data model are ahead of every reference.

---

## 2. IME & Keyboard Input (torvox's weakest area)

This is the largest section because the exploration reports converge here: torvox's
keyboard encoding is *both* behind termux/Haven on the Kotlin side *and* subtly wrong
against the libghostty-vt contract it just adopted.

### 2.1 Soft-keyboard IME modes

| Concern | torvox | Reference (best practice) |
|---|---|---|
| Secure/private mode | `KeyboardMode.Secure` uses `TYPE_NULL` `KeyboardMode.kt:29-34`. This **disables CJK/voice composition entirely** (a regression vs Haven). | Haven `HavenKeyboardMode.kt:15-21` documents Secure as `TYPE_TEXT_VARIATION_VISIBLE_PASSWORD` + `TYPE_TEXT_FLAG_NO_SUGGESTIONS` — strongest "don't rewrite input" hint Gboard honors, **while still allowing CJK composition** because the terminal hosts its own composition flow. |
| TYPE_NULL fallback | `KeyboardMode.kt:29-34` has *no* fallback. | termux keeps `TYPE_NULL` as default but falls back to `VISIBLE_PASSWORD\|NO_SUGGESTIONS` for buggy Samsung/Japanese IMEs `TerminalView.java:313-330` (issue #137/#686). ghostty-android keeps the plain `TYPE_NULL` path `TerminalView.java:1883`. |
| Rich/compose input | `BaseInputConnection` in `TerminalSurface.kt:1191-1272` handles `commitText`/`sendKeyEvent`/`deleteSurroundingText` but has **no `setComposingText`/`finishComposingText` override** — composing IMEs fall back to default behavior and deltas are dropped. | ghostty-android keeps composition on the rich path and restarts the IME on alt-screen `docs/architecture.md:384-397`. Haven reconciles `setComposingText` deltas live in `WaylandDesktopView.kt:265-324` (backspace-on-contraction), robust against Samsung/CJK. |
| Mode signal to IME | snapshot `meta[14]` exists in `bridge.rs` but is not plumbed to the keyboard mode. | ghostty-android disables rich input when terminal enters alt-screen/DECCKM (signaled via `meta[14]`), restarting the IME `docs/architecture.md:384-397`; `terminal_jni.c:599-625` mirrors what torvox should expose. |

**Gap T2.1 (P0):** Change `KeyboardMode.Secure` from `TYPE_NULL` to
`VISIBLE_PASSWORD|NO_SUGGESTIONS` (`KeyboardMode.kt:29-34`), and add a `setComposingText`/
`finishComposingText` override modeled on `WaylandDesktopView.kt:296-316`. Add the termux
Samsung fallback to `toEditorInfo`.

### 2.2 Hardware-key layout correctness (`getUnicodeChar`)

torvox has **no** layout-aware hardware-key path. `TerminalSurface.onKeyDown`
(`TerminalSurface.kt:1388-1419`) sends `event.unicodeChar` — the *full-metaState* value —
directly to the encoder, with no `FLAG_SOFT_KEYBOARD`/`SOURCE_KEYBOARD`/`isFromSource`
filtering and no `KeyCharacterMap` recomputation.

| Concern | torvox | Reference |
|---|---|---|
| Layout-aware char | none — `bridge.processKeyEvent(keyCode, modifiers, action, event.unicodeChar)` `TerminalSurface.kt:1397-1398` | Haven intercepts `Activity.dispatchKeyEvent` *before* the View for physical keyboards only, then `event.getUnicodeChar(meta)` for layout-correct chars (German QWERTZ `Shift+2`→`"`, AltGr+Q→`@`) `TerminalScreen.kt:1320-1407`. |
| Source filtering | none (only `isFromSource(SOURCE_MOUSE)` at `TerminalSurface.kt:1444`) | Haven guards `FLAG_SOFT_KEYBOARD`, `VIRTUAL_KEYBOARD`, `SOURCE_KEYBOARD`, `hasHardwareKeyboard` `TerminalScreen.kt:1334-1351` so IME composition text is never hijacked. |
| Dead/combining accents | not buffered; relies on libghostty via single `unicodeChar` | termux buffers `COMBINING_ACCENT` and composes via `KeyCharacterMap.getDeadChar` `TerminalView.java:825-835`; `getUnicodeChar` result computed at `:817`. |
| AltGr (right Alt) | not distinguished from left Alt in Kotlin | termux keeps right-Alt for composition, strips `META_ALT_ON` for left alt `TerminalView.java:806-811`; Haven `TerminalScreen.kt` lets AltGr through to `getUnicodeChar`. |

**Gap T2.2 (P0):** Add a `KeyEventInterceptor`-style Activity-level `dispatchKeyEvent`
hook applying `getUnicodeChar(meta)` for physical, non-special, non-Ctrl/Alt keys, exactly
like `TerminalScreen.kt:1320-1407`. Termux's `onKeyDown` at `TerminalView.java:769-835` is
the secondary reference for combining-accent + AltGr handling.

### 2.3 Ctrl / Alt encoding & the Ghostty encoder contract

torvox adopted libghostty-vt's key encoder but **populates its fields incorrectly**.
`KeyEncode` at `ghostty_terminal.rs:474-526`:

```rust
event.set_unshifted_codepoint(character);                       // :514
event.set_utf8(Some(character.encode_utf8(&mut utf8_buf)));     // :515-516  <- SAME value
```

The encoder contract (`key/event.h:155-186`) requires these be **two distinct values**:

- `utf8` = produced text **without** Ctrl/Alt (but Shift-as-produced kept).
- `unshifted_codepoint` = the base character with **no** modifiers.

The **correct** reference path (`TerminalView.java:2147-2168` → `terminal_jni.c:1273-1309`):

```java
int unicode   = event.getUnicodeChar(meta & ~(CTRL|ALT));   // text without ctrl/alt
int unshifted = event.getUnicodeChar(0);                    // no modifiers at all
if ((mods & SHIFT) != 0 && utf8 != null && unicode != unshifted)
    mods &= ~SHIFT;                                          // strip shift consumed by char
session.sendKey(keyCode, mods, utf8, unshifted);            // two values passed
```

Concretely, torvox's bugs vs the reference:

| # | Bug | torvox | Reference |
|---|---|---|---|
| 1 | `utf8` == `unshifted_codepoint` | both = `char::from_u32(unicode_char)` `ghostty_terminal.rs:511-517` | distinct: `utf8` from `getUnicodeChar(meta & ~(CTRL\|ALT))`, `unshifted` from `getUnicodeChar(0)` `TerminalView.java:2147-2150` |
| 2 | C0 control passed as `utf8` | `Ctrl+A` (0x01) sent as `utf8` | reference passes `utf8 = null` for pure control keys `terminal_jni.c:83-84`; encoder header forbids C0 in `utf8`. |
| 3 | SHIFT not stripped when char changed | `Shift+;`→`:` still carries SHIFT → encoder emits `\033[59;2u` at Kitty levels 1–2 | reference strips SHIFT `TerminalView.java:2159-2161` |
| 4 | encoder/event allocated per keypress | `key::Encoder::new()` + `key::Event::new()` every press `ghostty_terminal.rs:489-506` | allocated **once** per `TermCtx` `terminal_jni.c:182-188`, reused; can lose per-encoder state otherwise. |
| 5 | two values not plumbed through bridge | `processKeyEvent(keyCode, modifiers, action, unicodeChar)` single value `TerminalSurface.kt:1398`; `bridge.rs:1264-1294` | `sendKey(code, mods, utf8, unshifted)` `TerminalView.java:2168` (`bridge.rs` + `TorvoxBridge.kt` must gain an `unshifted_char` field — AGENTS.md requires FFI sync). |

**Gap T2.3 (P0):** In Kotlin compute `utf8 = event.getUnicodeChar(meta & ~(CTRL|ALT))` and
`unshifted = event.getUnicodeChar(0)`, strip SHIFT when `unicode != unshifted`, pass `null`
for C0 controls, and extend `processKeyEvent`/`bridge.rs:1264` to carry both. Store one
`key::Encoder` + `key::Event` in the `GhosttyTerminal` worker and reuse
(`ghostty_terminal.rs:489-506`).

### 2.4 Kitty keyboard protocol

torvox correctly calls `encoder.set_options_from_terminal(&terminal)` before each encode
(`ghostty_terminal.rs:497`), matching the reference `terminal_jni.c:1276`. Mode negotiation
(DECCKM, keypad app mode, alt-esc, modifyOtherKeys, Kitty flags) is therefore honored. The
*protocol-level* support is fine; the field-population bugs in section 2.3 are what break it
in practice (e.g. Shift printable keys emitting spurious `\033[59;2u`).

### 2.5 Composing IME & dedup

torvox's `InputCoalescer` (`InputCoalescer.kt:14-52`) uses a **50 ms time-window** dedup
(`DEDUP_WINDOW_NS = 50_000_000` `:18`). This is imprecise: it can (a) drop a legitimate
fast repeated char typed within 50 ms, or (b) pass through a genuine IME double-fire that
arrives >50 ms apart.

| Concern | torvox | Reference |
|---|---|---|
| Dedup strategy | time window `InputCoalescer.kt:29-33` | Haven detects the IME double-fire as **exactly two identical bytes in one batch** (`buffer.size == 2 && buffer[0] == buffer[1]`) `TerminalViewModel.kt:185-225` — matches Android's message model. |
| Composing overlay | none | Haven hosts a terminal-local compose buffer / overlay at the cursor `HavenTerminal.kt:24-28`, `KeyboardToolbar.kt:182-189`. |

**Gap T2.5 (P1):** Replace the time-window coalescer with Haven's message-batch identical-pair
signature (`InputCoalescer.kt:14-34`). It is both more correct and matches Android delivery.

### 2.6 Summary table — keyboard/IME adoption map

| Sub-area | torvox severity | Primary reference | torvox file:line |
|---|---|---|---|
| Secure mode = TYPE_NULL (no CJK) | P0 | Haven `HavenKeyboardMode.kt:15-21` | `KeyboardMode.kt:29-34` |
| No `setComposingText` override | P0 | Haven `WaylandDesktopView.kt:296-316` | `TerminalSurface.kt:1191-1272` |
| No `getUnicodeChar` layout path | P0 | Haven `TerminalScreen.kt:1320-1407`; termux `TerminalView.java:817` | `TerminalSurface.kt:1397-1398` |
| Encoder utf8==unshifted | P0 | ghostty-android `TerminalView.java:2147-2150` | `ghostty_terminal.rs:511-517` |
| C0 passed as utf8 | P0 | ghostty-android `terminal_jni.c:83-84` | `ghostty_terminal.rs:511-517` |
| SHIFT-not-stripped | P0 | ghostty-android `TerminalView.java:2159-2161` | `ghostty_terminal.rs:508-510` |
| Per-keypress encoder alloc | P1 | ghostty-android `terminal_jni.c:182-188` | `ghostty_terminal.rs:489-506` |
| 50 ms time-window dedup | P1 | Haven `TerminalViewModel.kt:185-225` | `InputCoalescer.kt:18,29-33` |
| Samsung/IME TYPE_NULL fallback | P1 | termux `TerminalView.java:313-330` | `KeyboardMode.kt:29-34` |
| Dead/combining accent buffer | P2 | termux `TerminalView.java:825-835` | `TerminalSurface.kt:1397` |
| AltGr vs left-Alt split | P2 | termux `TerminalView.java:806-811` | `TerminalSurface.kt:1395` |

---

## 3. PTY & Process Management

| Concern | torvox | Reference (correct) |
|---|---|---|
| PTY creation | `pty.rs` fork/exec (only allowed `unsafe`) `torvox-terminal/src/pty.rs` | termux `open("/dev/ptmx", O_RDWR\|O_CLOEXEC)`, `grantpt/unlockpt/ptsname_r` `termux.c:25`; ghostty-android `pty_jni.c:53-131`. |
| `IUTF8` / flow control | verify ghostty sets these | termux **enables `IUTF8`** and **disables `IXON\|IXOFF`** (`termux.c:54-59`) so Ctrl+S doesn't freeze the display; re-asserted on resize `termux.c:195-196`. |
| `setsid()` + controlling tty | verify ghostty child does this | termux `setsid()` + `dup2` 0/1/2 + close stray fds by scanning `/proc/self/fd` `termux.c:87-96`; ghostty-android `setsid()` + controlling-tty `pty_jni.c:53-131`. |
| Pixel winsize | verify `ws_xpixel/ws_ypixel` set | termux sets `ws_row/col` **and** `ws_xpixel = cols*cellW, ws_ypixel = rows*cellH` `termux.c:62`, re-issued on resize `termux.c:187`. ghostty-android seeds pixel size in initial winsize `pty_jni.c:34-49`. Needed for Kitty `icat`. |
| EIO-as-EOF | verify reader treats EIO as EOF | termux reader treats EIO as end-of-pty `TerminalSession.java`; ghostling `pty_read` `main.c:132-156` models EAGAIN drop + EINTR retry. |
| Signal handling / exit | `Drop` SIGKILLs child `pty.rs` (exploration-rust:51) | termux `waitpid` returns **negated signal** `-WTERMSIG` on crash `termux.c:208`, prints `[Process completed (signal N)]` `TerminalSession.java:353`. |
| SIGHUP foreground group | — | ghostty-android `ptyHangupForeground` sends SIGHUP to foreground pgrp via `TIOCGPGRP` `pty_jni.c`. Adoptable. |
| Orphaned child (BUG) | `into_raw_fd` `pty.rs:254-258` `std::mem::forget(self)` leaks the child (Drop never reaps). **Never called** — delete. | n/a (no equivalent dead code in references). |
| `TERM` env | config-driven; verify no `xterm-ghostty` assumption | ghostty-android deliberately uses `TERM=xterm-256color` (no terminfo db on device) `pty_jni.c`; matches `filesDir/home` rule (AGENTS.md pitfall #16/#18). |

**Gap T3.1 (P1):** Verify/ensure libghostty-vt sets `IUTF8`, clears `IXON|IXOFF`, does
`setsid()`, closes stray fds, and sets `ws_xpixel/ws_ypixel`. Adopt negated-signal exit
reporting + a "process completed" banner. **Delete** dead `into_raw_fd` (`pty.rs:254-258`).

---

## 4. Renderer

| Axis | torvox | termux / Haven / ghostty-android |
|---|---|---|
| Technology | wgpu (Vulkan) GPU `torvox-renderer/src/gpu.rs` | Canvas `TerminalRenderer.java:19`, `TerminalView.java:187-216`, ghostty-android `TerminalView.onDraw`. **torvox is ahead — keep GPU.** |
| Glyph atlas | `guillotiere` packer + `swash` raster + `cosmic-text` shaping `font.rs` (AGENTS.md) | none — termux `Typeface.MONOSPACE` + `Paint`; per-cell `drawText`. |
| Font shaping | real shaping (CJK fallback scoring `font.rs`) | platform text stack; wide CJK uses spacer tail cell `TerminalView.java` grapheme overflow. |
| Style-run batching | builds `Instance[]` per glyph — verify batching by (style, atlas page) | termux groups identical-style runs `TerminalRenderer.java:114`; ghostty-android batches same-color runs into one `drawText`. |
| Font-width mismatch | verify glyph anchored to cell | termux scales glyph when measured width != wcwidth `TerminalRenderer.java:106-112`; adopt: never overflow into next cell. |
| Dirty tracking | `DirtyMask { partitions: Vec<u64> }` `torvox-core/src/cell.rs:112`; per-row `dirty: Vec<bool>` | ghostling clears `ROW_OPTION_DIRTY` per row `main.c:936-939`; termux `mTopRow` scroll offset `TerminalView.java:450`. |
| Reflow on resize | verify Ghostty reflows scrollback | termux reflows `TerminalBuffer.resize` (`TerminalEmulator.resize:386`); ghostling `ghostty_terminal_resize` `main.c:1436-1462`. |
| Grapheme clustering | `CellSnapshot.graphemes: Vec<u32>` `ghostty_terminal.rs:12-94` | ghostling ships full cluster in overflow buffer + `GRAPHEME` row flag `main.c:847-882`; re-asserts DEC 2027 after each feed `terminal_jni.c:339-348`. **Verify torvox re-asserts forced modes after RIS.** |
| Diagnostics hygiene | leftover `DIAG_BIND_GROUP` logs `gpu.rs:1220-1290`; contradicted R8Unorm/sRGB comment `gpu.rs:1162-1164` | n/a. |

**Gap T4.1 (P2):** Remove `DIAG_BIND_GROUP` instrumentation and fix the R8Unorm comment in
`gpu.rs`. Verify glyphs are anchored to the cell grid and that forced DEC modes (2027) are
re-asserted after `RIS`. Keep the GPU path — it is superior to every reference.

---

## 5. Lifecycle

| Concern | torvox | Reference |
|---|---|---|
| Survive Activity recreation | `TerminalSurface.kt:666` re-creates the surface from the live `ANativeWindow`; session stays in `TorvoxRuntime` (exploration-kotlin:92-94) | Haven re-adopts the **same** emulator after recreation, never rebuilds it `TerminalViewModel.onCleared:479-517`, `SshTerminalEmulatorOwner.kt:30-57`. termux `TermuxService` `TermuxActivity.java:260`. ghostty-android `SessionManager` singleton `docs/architecture.md:420-425`. |
| Foreground service + wakelock | `TerminalForegroundService` `PARTIAL_WAKE_LOCK` 30 min (exploration-kotlin:217) | termux `TermuxService` keeps shells alive; ghostty-android `SessionService.java:34` `FOREGROUND_SERVICE_TYPE_SPECIAL_USE`. |
| **Wakelock release (BUG)** | `TorvoxRuntime` **never calls** `TerminalForegroundService.updateSessionCount` (exploration-kotlin:218-221) → 30-min wakelock + service can outlive the last session | termux wires session count so the service stops when the last session closes (by design). |
| No-op resize skip | — | ghostty-android skips no-op resizes (mksh wipes prompt on `SIGWINCH`) `pty_jni.c` notes; termux resizes only when dims change `TerminalView.java:991`. **Adopt.** |
| Crash/teardown | `TorvoxApp.kt` global handler writes stacktrace to file `TorvoxApp.kt:45,60` | termux `CrashHandler.java:18` writes Markdown report; `TerminalEmulator.java:20-262` no-op-after-close guard `handle==0`. |
| Compose surface per Activity | re-creates Compose terminal surface per Activity (exploration-kotlin:179) | Haven keeps a single persistent emulator; only the View is swapped. **Verify torvox does not reinstantiate the Ghostty `Terminal`/grid on rotation.** |
| Theme re-apply | `MainActivity.onConfigurationChanged` rebuilds full `TerminalConfig` on dark/light (exploration-kotlin:222) | ghostty-android `onResume` re-applies theme to all open sessions `MainActivity.onResume`. |

**Gap T5.1 (P1):** Wire `closeSession` → `TerminalForegroundService.updateSessionCount(0)`
so the wakelock/service releases (exploration-kotlin:218-221). **Gap T5.2 (P2):** verify
the Ghostty `Terminal`/grid is not recreated on rotation; formalize swappable input/resize
sinks like Haven `SshTerminalEmulatorOwner.kt:216-230`.

---

## 6. Error Handling Patterns

| Concern | torvox | Reference |
|---|---|---|
| FFI panic isolation | `std::panic::catch_unwind` at FFI boundary `bridge.rs:1651` (exploration-kotlin) | ghostty-android same: buffers callbacks into `TermCtx`, no native→Java upcalls `terminal_jni.c:98-112`. |
| Silent input drop (BUG) | `write_to_pty` / `process_key_event` return `Ok(())` on missing/poisoned session → keystrokes silently lost `bridge.rs:1247,1264` (exploration-rust:137-140) | Haven logs-and-continues at I/O boundary **with a rationale comment** `LocalSession.kt:101-109`; never silently drops. |
| Swallow-with-rationale discipline | inconsistent; render-loop catches swallow clipboard/notification failures `TorvoxRuntime.kt:1041-1075` | Haven comments every swallow explaining the race `#208 findings`; termux `TerminalRecorder.record` swallows only raced `dispose()` `TerminalViewModel.kt:73-94`. |
| Untrusted-length buffers | `WireReader.readString/readI32` no bounds check (exploration-kotlin:120-124) | Haven `ensureOutputCapacity` grows first to avoid remote-triggered overflow from accumulated OSC 52 payload `OscHandler.kt:257-264`. |
| Crash reporter | `TorvoxApp.kt` writes raw stacktrace | termux `CrashHandler.java:18,89` writes **Markdown** report (thread, timestamp, full stack, device info) + `ACTION_NOTIFY_APP_CRASH` broadcast `TermuxCrashUtils.java:34`. |
| Structured errors | `bridge.rs` returns `-1` on every error `:1611,:1640` (exploration-termux) | termux `Errno`/`Error` typed error codes `errors/Error.java`; not bare `-1`. |
| OOM in render/encode | — | ghostty-android drops response bytes on `realloc` failure `terminal_jni.c:105-108`; returns NULL when a key encodes to nothing `terminal_jni.c:1304` (not a warn storm). |
| Use-after-close | verify `Terminate` refuses further commands | ghostty-android `handle==0` no-op guard after `close()` `TerminalEmulator.java:20-262`. |

**Gap T6.1 (P0):** `write_to_pty`/`process_key_event` must return `Err(TerminalError::…)`
when the session is missing/poisoned instead of `Ok(())` (`bridge.rs:1247,1264`). **Gap
T6.2 (P1):** add bounds validation to `WireReader` (exploration-kotlin:120-124) and a
Markdown uncaught-exception handler on the Kotlin side like termux `CrashHandler.java:89`.
**Gap T6.3 (P2):** replace bare `-1` bridge returns with typed errors; adopt the
"swallow only with a rationale comment" discipline (extend AGENTS.md `#[allow]` ban to
Kotlin).

---

## 7. Test Strategy (ghostling behavior catalog → 6 torvox test types)

ghostling (`ghostling/main.c`) is a **libghostty-vt demo**, not a test tool — it has no
DSL, no assertion engine, no screenshot diff; its only verification is a human-watched
`demo.gif`. Its value is (a) a **catalog of terminal behaviors** a consumer must support
and (b) the canonical `RenderState` cell model that is the correct **oracle** for
text-based conformance tests (`main.c:798-1012`). torvox already has this model
(`GridSnapshot`/`CellSnapshot`, `ghostty_terminal.rs:12-94`) and all six test types. The
recommendation is to **extend coverage**, not adopt a new framework.

### 7.1 Behavior catalog (B1–B15)

| # | Behavior | ghostling location |
|---|---|---|
| B1 | VT parse + SGR colors/styles | `main.c:884-931` |
| B2 | Multi-codepoint graphemes (CJK, ZWJ) | `main.c:847-882` |
| B3 | Cursor pos/visibility/shape | `main.c:956-978` |
| B4 | Resize **with reflow** | `main.c:1436-1462` |
| B5 | Scrollback + viewport scroll | `main.c:1278,431-439,988-1012` |
| B6 | Kitty keyboard (mode-aware) | `main.c:447-563` |
| B7 | Mouse tracking (X10/normal/any-event; SGR/URxvt/UTF8) | `main.c:304-440` |
| B8 | Kitty Graphics (images, z-layers) | `main.c:651-784,818-985` |
| B9 | Focus CSI I / O (gated DECSET 1004) | `main.c:1464-1486` |
| B10 | Bracketed paste | (libghostty; encoder) |
| B11 | OSC title/color-scheme/clipboard (52) | `main.c:1324-1327` |
| B12 | VT queries: DA, xterm version, size | `main.c:1318-1322` |
| B13 | Per-row dirty tracking | `main.c:936-939` |
| B14 | Inverse video (fg/bg swap) | `main.c:904-910` |
| B15 | Wheel → scrollback vs forwarded | `main.c:409-431` |

### 7.2 Coverage matrix

| Behavior | Unit (Rust) | Roborazzi | Compose UI | Maestro | UIAutomator | Espresso |
|---|---|---|---|---|---|---|
| B1 SGR/colors/styles | `.ref` `sgr_*`, `esctest_*` | `screenshot/*ScreenshotTest` | `ThemeSettingsComposeTest` | `theme-verify.yml` | — | `TextDecorationTest` |
| B2 Graphemes/CJK | `FontSwitchingAndCjkTest` | `FontFallbackTest` | `FontFallbackTest` | `terminal-i18n.yml` | — | — |
| B3 Cursor | `ref_runner` cursor asserts | `TerminalScreenScreenshotTest` | — | — | — | `TerminalScreenTest` |
| B4 Resize/reflow | `dpi_scaling.rs`, `layout.rs` | (partial) | — | `resize-survives.yml` | — | — |
| B5 Scrollback/scrollbar | `ref_runner`, `scroll.rs` | `TerminalScreenScreenshotTest` | — | `terminal-scroll.yaml` | — | — |
| B6 Kitty keyboard | `keyboard.rs`, `TerminalInputEncoderTest` | — | — | `modifier-*.yml` | — | `KeyboardJellyInstrumentedTest`, `CtrlAltModifierInstrumentedTest` |
| B7 Mouse | `mouse_protocol.rs` | — | `GestureInteractionTest`, `TouchGestureTest` | `long-press-copy.yml` | `TerminalUiAutomatorTest` | `TerminalActivityEspressoTest` |
| B8 Kitty graphics | `terminal_render_test.rs` | **(gap — add)** | — | — | — | — |
| B9 Focus | (mode read) | — | — | — | — | `KeyboardJellyInstrumentedTest` |
| B10 Bracketed paste | `bracketed_paste.rs` | — | — | — | — | — |
| B11 OSC title/clipboard | `osc52.rs`, `core_integration.rs` | `ModifierBarScreenshotTest` | — | `selection-copy-paste.yml` | — | `SelectionEspressoTest` |
| B12 VT queries | `.ref` **disabled** `ref_runner.rs:168` | — | — | — | — | — |
| B13 Dirty tracking | `grid.rs`, `cell.rs`, `property_tests.rs` | — | — | — | — | — |
| B14 Inverse | `ref_runner` (reverse field) | `ThemeScreenshotTest` | — | `theme-app-mode-switch.yml` | — | — |
| B15 Wheel to scroll | `mouse_protocol.rs` (concept) | — | — | `terminal-scroll.yaml` | — | — |

### 7.3 Highest-value test adoptions

- **Enable the disabled `.ref` runner** (`ref_runner.rs:168`): regenerate `.ref` JSON from
  `GhosttyTerminal::take_snapshot()` (`ghostty_terminal.rs:639`) — libghostty as oracle, zero
  pixel flakiness. Covers B1,B3,B4,B5,B11,B12,B14 deterministically.
- **Extend `compare_snapshots`** (`ref_runner.rs:189-205`) to assert `graphemes: Vec<u32>`,
  resolved `foreground/background` RGB, and `kgp_placements` (today only single `codepoint`).
- **Add byte-exact VT-query tests (B12)** so vim/tmux/htop startup queries are verified like
  ghostling's effects callbacks (`main.c:1318-1322`).
- **Add Roborazzi pixel test for Kitty Graphics (B8)** to catch atlas/packing regressions.
- **Add per-row dirty-region unit test (B13)** mirroring ghostling's `ROW_OPTION_DIRTY`
  clearing (`main.c:936-939`).
- **Map input modes to on-device flows** (B6/B7/B9/B15 → Maestro/Espresso/UIAutomator),
  respecting AGENTS.md pitfall #15 (no ADB touch injection on phone emulator).

**Do NOT adopt:** ghostling's manual/visual-only verification and its single-threaded Raylib
loop — torvox's multi-layer strategy already exceeds it.

---

## 8. Gap Analysis — What torvox MUST adopt

Priorities: **P0** = correctness/regression, do first; **P1** = robustness/lifecycle;
**P2** = hygiene/completeness.

### P0 (must fix — correctness)

| # | torvox (current, broken) | reference (correct) | recommended change |
|---|---|---|---|
| G1 | `ghostty_terminal.rs:511-517` sets `utf8` == `unshifted_codepoint` to the same char; `Ctrl+A`→0x01 sent as `utf8` (forbidden C0) | ghostty-android `TerminalView.java:2147-2150` computes two distinct values; `terminal_jni.c:83-84` passes `utf8=NULL` for control keys | Compute `utf8=getUnicodeChar(meta&~(CTRL\|ALT))`, `unshifted=getUnicodeChar(0)`; pass `null` for C0. Extend `bridge.rs:1264` + `TorvoxBridge.kt` (`processKeyEvent`) with an `unshifted_char` field. |
| G2 | `ghostty_terminal.rs:508-510` keeps SHIFT in mods even when Shift only changed the printed char → spurious `\033[59;2u` under Kitty | ghostty-android `TerminalView.java:2159-2161` strips SHIFT when `unicode != unshifted` | Replicate the shift-strip in Kotlin before calling the bridge. |
| G3 | `KeyboardMode.kt:29-34` Secure mode = `TYPE_NULL` → **disables CJK/voice composition** | Haven `HavenKeyboardMode.kt:15-21` Secure = `VISIBLE_PASSWORD\|NO_SUGGESTIONS` (CJK still works) | Change Secure to `VISIBLE_PASSWORD\|NO_SUGGESTIONS`; add `setComposingText`/`finishComposingText` on `BaseInputConnection` (`TerminalSurface.kt:1191-1272`), modeled on `WaylandDesktopView.kt:296-316`. |
| G4 | `TerminalSurface.kt:1397-1398` sends raw `event.unicodeChar` with no `getUnicodeChar`/source filtering → wrong symbols on non-US hardware layouts | Haven `TerminalScreen.kt:1320-1407` `getUnicodeChar` for physical keyboards; termux `TerminalView.java:817` | Add Activity-level `dispatchKeyEvent` hook applying `getUnicodeChar(meta)` for physical, non-special, non-Ctrl/Alt (except AltGr) keys; add termux Samsung/IME `TYPE_NULL` fallback (`TerminalView.java:313-330`). |
| G5 | `bridge.rs:1247,1264` `write_to_pty`/`process_key_event` return `Ok(())` on missing/poisoned session → keystrokes silently dropped | Haven `LocalSession.kt:101-109` logs-and-continues with rationale; never silently drops | Return `Err(TerminalError::…)` so the UI can surface the failure. |
| G6 | `osc_handler.rs:21-26` `HANDLED_OSC` omits `7`; `OscEvent::Cwd` never emitted; `session.rs:338` dead no-op; doc at `osc_handler.rs:6` contradicted | OSC 7 is a standard cwd report | Add `OSC_CWD` to `HANDLED_OSC` (`osc_handler.rs:21-26`) and reconcile the dead `Cwd(_)=>{}` arm in `session.rs:338`. |

### P1 (robustness / lifecycle)

| # | torvox (current) | reference | recommended change |
|---|---|---|---|
| G7 | `ghostty_terminal.rs:489-506` allocates `key::Encoder`/`key::Event` every keypress | ghostty-android `terminal_jni.c:182-188` allocates once per `TermCtx` and reuses | Store one encoder + event in the worker struct; reset fields each encode. |
| G8 | `TorvoxRuntime.kt` never calls `TerminalForegroundService.updateSessionCount` → 30-min wakelock outlives last session (exploration-kotlin:218-221) | termux `TermuxService` stops when last session closes | Wire `closeSession` → `updateSessionCount(0)`. |
| G9 | `pty.rs:254-258` `into_raw_fd` `std::mem::forget(self)` leaks the child; dead code | n/a | Delete it. |
| G10 | PTY flags (`IUTF8`, `IXON\|IXOFF`, `ws_xpixel/ws_ypixel`, `setsid`, stray-fd close) unverified | termux `termux.c:54-96`, ghostty-android `pty_jni.c:53-131` | Verify libghostty sets these; adopt negated-signal exit banner (termux `termux.c:208`). |
| G11 | `WireReader.readString/readI32` no bounds check (exploration-kotlin:120-124) | Haven `OscHandler.kt:257-264` `ensureOutputCapacity` | Validate `position+length <= data.size` before copy. |
| G12 | `.ref` conformance runner **disabled** `ref_runner.rs:168`; `compare_snapshots` only checks single `codepoint` | ghostling `main.c:798-1012` `RenderState` oracle | Regenerate `.ref` from libghostty snapshots; assert graphemes/RGB/kgp. |
| G13 | `InputCoalescer.kt:18,29-33` 50 ms time-window dedup | Haven `TerminalViewModel.kt:185-225` exact-pair-in-batch signature | Replace with message-batch identical-pair detection. |
| G14 | No Markdown uncaught-exception handler on Kotlin side (only Rust `catch_unwind` at `bridge.rs:1651`) | termux `CrashHandler.java:18,89` | Add `Thread.setDefaultUncaughtExceptionHandler` writing Markdown + surfacing. |

### P2 (hygiene / completeness)

| # | torvox | reference | recommended change |
|---|---|---|---|
| G15 | `gpu.rs:1220-1290` leftover `DIAG_BIND_GROUP` logs; `gpu.rs:1162-1164` contradicted R8Unorm/sRGB comment | — | Remove diagnostics; fix comment. |
| G16 | `ghostty_terminal.rs:534-544` `vt_write` appends `ST+SGR` after every write | ghostty-android buffers responses in-call `terminal_jni.c:98-112` | Document the contract; skip when a full sequence is in flight. |
| G17 | `pty_write` unconditional `\n`→`\r\n` `ghostty_terminal.rs:554-567` → `\r\r\n` double-CR | — | Skip insertion when previous byte is already `\r`. |
| G18 | `bridge.rs:283-313` lossy `From<TerminalConfig>` drops home/user/path/theme | — | Full round-trip or delete the impls. |
| G19 | `take_snapshot_with_scroll` `ghostty_terminal.rs:643` returns silent empty fallback (no log) | every other accessor uses `recv_or_fallback` which logs | Log a warning on fallback. |
| G20 | No DECCKM/alt-screen "input-mode" signal to IME (`meta[14]` unplumbed) | ghostty-android `docs/architecture.md:384-397` | Plumb `meta[14]`; restart IME on alt-screen. |
| G21 | Dead code: `TerminalInputEncoder.kt` legacy manual encoder (pre-encoder path) | ghostty-android routes all keys through the encoder | Route all keys through the encoder; retire the legacy encoder. |
| G22 | `TorvoxBridge.kt:521` `nativeLib` not `@Volatile` (double-checked locking bug); `:777` selection 16-bit overflow; `:564-567` wire no bounds (exploration-kotlin) | — | Mark `@Volatile`; validate selection packing; bounds-check wire. |

---

## 9. Dependency Recommendations (DO NOT apply — Cargo.toml/build.gradle are config)

These are documented for the upcoming refactor review. Per AGENTS.md, crate/dependency
changes are "Ask First" and `Cargo.toml`/`build.gradle` are config (cannot be edited here).

| Capability | torvox hand-rolled code | recommendation | why |
|---|---|---|---|
| VT parsing / OSC / mouse / bracketed-paste | `torvox-terminal/src/osc_handler.rs` (custom state machine) | **Keep** the custom parser; do **not** pull `vte`/`ansi-parser` | would conflict with the Ghostty VT engine and add deps (exploration-rust:176-178). |
| Theme / config key=value parse | `torvox-core/src/config.rs::parse_custom` (~120 lines) | **Keep** unless format grows; then consider `toml` | simple dialect today; avoids new dep (AGENTS.md). |
| Keyboard encoding | `ghostty_terminal.rs:474-526` (libghostty encoder — correct choice) | **Keep** libghostty-vt encoder; delete torvox's parallel `TerminalInputEncoder.kt` | reference routes all keys through the encoder (G21). |
| IME input connection | `TerminalSurface.kt:1191-1272` `BaseInputConnection` | **Keep** the hand-rolled `BaseInputConnection`; it is the correct Android pattern | matches termux/ghostty-android. |
| Coalescing / dedup | `InputCoalescer.kt` (time-window) | **Keep** the class but change algorithm to Haven's batch-pair signature (G13) | no new dependency needed. |
| Crash reporting | `TorvoxApp.kt` raw stacktrace | consider **termux's `CrashHandler` pattern** (in-repo reimplementation, no external lib) | Markdown report + broadcast; no third-party dep required. |
| Structured errors | `bridge.rs` returns `-1` | adopt termux `Errno`/`Error` **typed enum** style in `torvox-core` | in-code enum, no external crate. |
| Font shaping / raster | `cosmic-text` + `swash` + `guillotiere` (already used) | **Keep** — ahead of every reference | GPU atlas + real shaping (section 4). |
| wgpu/Vulkan renderer | `torvox-renderer/src/gpu.rs` | **Keep** | strictly more advanced than Canvas references (section 4). |
| Unicode width | `unicode-width` (via `torvox-core`) | **Keep** | already the de-facto crate; matches termux's `wcwidth` needs. |
| Unsafe policy | `unsafe` only in `pty.rs` (fork) + gui-android FFI | **Keep** `forbid(unsafe_code)` in `torvox-core`/`torvox-renderer` | cargo geiger must stay clean (AGENTS.md). |

**Summary:** torvox should **not** add dependency-replacing crates for VT parsing, theme
parsing, or shaping — those are already correct or intentionally hand-rolled. The real
gains are **algorithmic/behavioral** (section 8), not dependency swaps. The only "dependency-style"
adoption that makes sense is re-implementing termux's `CrashHandler` Markdown pattern and
typed error enums *in-repo* (no new crate). Keep `cosmic-text`/`swash`/`guillotiere`/`wgpu`
as they are uniformly ahead of the Canvas-based references.

---

## Appendix — Citation index (torvox)

| File | Lines | Used in |
|---|---|---|
| `torvox-terminal/src/ghostty_terminal.rs` | 474-526, 489-506, 511-517, 534-544, 546-567, 643 | section 2.3, 4, 8 G1/G2/G7/G16/G17/G19 |
| `torvox-gui-android/src/bridge.rs` | 1247, 1264, 283-313, 1651 | section 2.3, 6, 8 G5/G18 |
| `android/app/.../ui/KeyboardMode.kt` | 29-34, 27-75 | section 2.1, 8 G3 |
| `android/app/.../ui/InputCoalescer.kt` | 14-34, 18 | section 2.5, 8 G13 |
| `android/app/.../ui/TerminalSurface.kt` | 1191-1272, 1388-1419, 1397, 1444 | section 2.1, 2.2, 2.3, 8 G3/G4 |
| `android/app/.../ui/TerminalInputEncoder.kt` | entire | section 8 G21 |
| `torvox-terminal/src/osc_handler.rs` | 21-26, 29, 6 | section 8 G6 |
| `torvox-terminal/src/session.rs` | 338 | section 8 G6 |
| `torvox-terminal/src/pty.rs` | 254-258 | section 3, 8 G9 |
| `torvox-renderer/src/gpu.rs` | 1220-1290, 1162-1164 | section 4, 8 G15 |
| `android/app/.../TorvoxBridge.kt` | 521, 777, 564-567 | section 8 G22 |
| `android/app/.../TorvoxRuntime.kt` | 1041-1075, 812 | section 6, 8 G8 |

## Appendix — Citation index (references)

| File | Lines | Used in |
|---|---|---|
| `ghostty-android/.../TerminalView.java` | 2147-2168, 1883 | section 2.3, 2.1 |
| `ghostty-android/.../cpp/terminal_jni.c` | 1273-1309, 182-188, 98-112, 599-625 | section 2.3, 6 |
| `ghostty-android/.../cpp/pty_jni.c` | 53-131, 34-49 | section 3 |
| `Haven/.../HavenKeyboardMode.kt` | 15-21, 59-86 | section 2.1, 8 G3 |
| `Haven/.../feature/terminal/TerminalScreen.kt` | 1320-1407 | section 2.2, 8 G4 |
| `Haven/.../core/wayland/WaylandDesktopView.kt` | 265-324, 296-316 | section 2.1, 8 G3 |
| `Haven/.../LocalSession.kt` | 34-41, 75-83, 101-109, 155-159 | section 6 |
| `termux/.../TerminalView.java` | 307-339, 313-330, 769-835, 817, 825-835, 806-811 | section 2.1, 2.2, 8 G4 |
| `termux/.../jni/termux.c` | 54-96, 57-62, 187, 195-196, 208 | section 3 |
| `termux/.../TerminalSession.java` | 31, 44, 49, 133, 150, 166, 353 | section 1, 3 |
| `termux/.../TerminalRenderer.java` | 19, 47, 106-112, 114 | section 4 |
| `termux/.../CrashHandler.java` | 18, 89 | section 6, 8 G14 |
| `ghostling/main.c` | 798-1012, 847-882, 884-931, 936-939, 1278, 1318-1322, 1436-1462, 1464-1486 | section 7 |

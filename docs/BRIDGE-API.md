# Bridge API Reference

The Torvox FFI bridge connects Rust (the terminal engine + renderer) to Kotlin (the Android UI). It uses **boltffi** for plain C FFI with POD data types, plus **JNI** for the specific case of `ANativeWindow_fromSurface()`.

---

## Architecture

```
Kotlin (JNA) → boltffi C ABI structs → Rust (TorvoxBridge)
                  ↕                          ↕
           TorvoxBridge.kt              bridge.rs
           (JNA interface)              (#[boltffi::data] + #[boltffi::export])
```

All terminal operations go through this bridge. Kotlin never accesses Rust state directly.

---

## Data Types (boltffi)

### BridgeCell

```rust
pub struct BridgeCell {
    pub character: u32,      // Unicode code point
    pub foreground: u32,     // ARGB color
    pub background: u32,     // ARGB color
    pub bold: u8,           // 0/1
    pub italic: u8,
    pub underline: u8,      // 0=none, 1=single, 2=double, 3=curly, 4=dotted, 5=dashed
    pub strikethrough: u8,
    pub blink: u8,
    pub dim: u8,
    pub reverse: u8,
    pub invisible: u8,
    pub overline: u8,
    pub crossed_out: u8,
    pub fraktur: u8,
}
```

### BridgeAttrs

Color-attribute packing for renderer. All color values are 32-bit ARGB.

### BridgeTheme

```rust
pub struct BridgeTheme {
    pub palette: [u32; 16],  // ANSI colors 0-15
    pub foreground: u32,
    pub background: u32,
    pub cursor_text: u32,
    pub cursor: u32,
    pub selection: u32,
    pub scrollbar: u32,
}
```

### TerminalEvent

```rust
pub struct TerminalEvent {
    pub event_type: u32,     // 0=OutputReady, 1=Bell, 2=TitleChanged,
                             // 3=ClipboardRequest, 4=CursorChanged,
                             // 5=SelectionChanged, 6=DirtyRegion
    pub data: [u8; 4096],   // Event payload (UTF-8 text, title, etc.)
    pub data_len: u32,
}
```

---

## Exported Functions (`#[boltffi::export]`)

### Session Management

| Function | Description |
|----------|-------------|
| `terminal_spawn(config: TerminalConfig) -> Result<u64>` | Create new terminal session, return handle |
| `terminal_close(handle: u64)` | Close session and clean up |
| `terminal_resize(handle: u64, rows: u32, cols: u32)` | Resize terminal grid + PTY window |
| `terminal_write_input(handle: u64, data: &[u8])` | Write data to PTY master (input) |
| `terminal_save_session(handle: u64, path: &str) -> Result<()>` | Serialize session via rkyv |
| `terminal_restore_session(path: &str) -> Result<u64>` | Deserialize and resume session |

### Rendering

| Function | Description |
|----------|-------------|
| `render_frame(handle: u64, snapshot: &mut SessionSnapshot)` | Render one frame, fill snapshot |
| `set_native_window(handle: u64, window_ptr: u64)` | Set ANativeWindow for rendering surface |
| `native_window_ready(handle: u64, width: u32, height: u32)` | Signal surface is ready with dimensions |

### Configuration

| Function | Description |
|----------|-------------|
| `set_font(handle: u64, config: FontConfig)` | Update font family/size/features |
| `set_theme(handle: u64, theme: BridgeTheme)` | Update terminal color scheme |
| `set_background_image(handle: u64, path: &str, opacity: f32)` | Set background wallpaper |
| `set_cursor_style(handle: u64, style: u32, blink: bool)` | Set cursor appearance |

### Input

| Function | Description |
|----------|-------------|
| `encode_key(handle: u64, key: &KeyEvent) -> Vec<u8>` | Encode Android key event to VT sequence |
| `encode_mouse(handle: u64, x: u32, y: u32, button: u32, action: u32) -> Vec<u8>` | Encode mouse event |
| `encode_paste(handle: u64, text: &str) -> Vec<u8>` | Encode paste text (bracketed paste aware) |

### Selection

| Function | Description |
|----------|-------------|
| `selection_set(handle: u64, mode: u32, anchor_row: u32, anchor_col: u32)` | Start selection |
| `selection_extend(handle: u64, row: u32, col: u32)` | Extend selection to position |
| `selection_get_text(handle: u64) -> String` | Get selected text |
| `selection_clear(handle: u64)` | Clear selection |
| `selection_word_at(handle: u64, row: u32, col: u32) -> String` | Get word at position (double-tap) |

### Search

| Function | Description |
|----------|-------------|
| `search_set_query(handle: u64, query: &str, case_sensitive: bool)` | Set search query |
| `search_next(handle: u64) -> bool` | Find next match |
| `search_prev(handle: u64) -> bool` | Find previous match |
| `search_clear(handle: u64)` | Clear search highlights |

### Clipboard

| Function | Description |
|----------|-------------|
| `clipboard_set(handle: u64)` | Set clipboard from OSC 52 request |

### Debug

| Function | Description |
|----------|-------------|
| `dump_grid(handle: u64) -> String` | Dump grid state for debugging |

---

## JNI Bridge

The JNI bridge has exactly one purpose: obtaining an `ANativeWindow*` pointer from a Kotlin `Surface`.

**Java declaration** (`NativeWindow.kt`):
```kotlin
object NativeWindow {
    external fun getNativeWindowPtr(surface: Surface): Long
}
```

**Rust implementation** (`jni_bridge.rs`):
```rust
#[no_mangle]
pub extern "system" fn Java_io_torvox_bridge_NativeWindow_getNativeWindowPtr(
    env: JNIEnv,
    _class: JClass,
    surface: JObject,
) -> i64
```

Returns the raw `ANativeWindow*` pointer as an `i64`, which is then passed to `set_native_window()`.

---

## Wire Format

Session save/restore uses **rkyv** (zero-copy serialization):

```
┌─────────────────┐
│ SessionSnapshot │  ← rkyv::Archive
│ ├── Grid        │     Vec<Line<Cell>>
│ ├── Scrollback  │     Vec<Line<Cell>>
│ ├── Cursor      │     position, style
│ ├── Modes       │     DEC private modes, SGR state
│ ├── Title       │     window/icon title
│ └── Config      │     TerminalConfig
└─────────────────┘
       ↓ rkyv::to_bytes
┌─────────────────┐
│  Archived bytes  │  → written to file
└─────────────────┘
       ↓ rkyv::from_bytes (bytecheck validated)
┌─────────────────┐
│ SessionSnapshot  │  → restored session
└─────────────────┘
```

---

## Thread Safety

The bridge surface is behind a `Mutex<TorvoxBridge>`:

- **Kotlin (JNA)**: calls bridge functions from the main/Compose thread
- **Rust**: `TorvoxBridge` internally dispatches to the session's render thread via `Arc<Mutex<Session>>`
- **Render thread**: single-threaded wgpu context, woken by `CountDownLatch`
- **PTY reader**: separate thread feeding `GhosttyTerminal` via flume channel

Lock ordering: bridge mutex → session mutex.

---

## Error Handling

All bridge functions return `Result<T, BridgeError>` where:

```rust
pub enum BridgeError {
    InvalidHandle,
    SessionNotInitialized,
    PtyError(String),
    RenderError(String),
    SerializationError(String),
    SurfaceNotReady,
}
```

`BridgeError` is serialized via boltffi and thrown as a Kotlin exception on the JNA side.

---

## Adding a New Bridge Function

1. Define the function in `bridge.rs` with `#[boltffi::export]`
2. Add the JNA binding in `TorvoxBridge.kt`
3. If the function touches `torvox-core` types, add serde/rkyv derives
4. Add SAFETY comments for any `unsafe` block
5. Add `catch_unwind` guard for panic safety
6. Write FFI contract test in `tests/ffi_contract_tests.rs`
7. Write Kotlin-side test in `android/app/src/test/`

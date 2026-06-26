# Haven Feature Comparison

Deep-read comparison of Haven's non-terminal features against torvox equivalents.

---

## 1. USB Serial

### Haven Implementation

**Files read:**
- `core/usb/src/main/kotlin/sh/haven/core/usb/UsbSerialManager.kt` (325 lines)
- `core/usb/src/main/kotlin/sh/haven/core/usb/UsbSerialDevice.kt`
- `core/usb/src/main/kotlin/sh/haven/core/usb/UsbForegroundParticipantModule.kt`
- `core/usb/src/test/kotlin/sh/haven/core/usb/UsbSerialManagerTest.kt`

**What it does:**
- Full Android USB Host API wrapper supporting CDC-ACM, FTDI, CH340/CH341, CP210x driver types
- USB permission request flow with `PendingIntent` + `BroadcastReceiver`
- Endpoint discovery: scans interfaces for `USB_CLASS_CDC_DATA`, `USB_CLASS_COMM`, `USB_CLASS_VENDOR_SPEC`, finds bulk IN/OUT endpoints
- Background read thread with 4096-byte buffer, 1s timeout, 1s write timeout
- `SerialListener` callback interface: `onDeviceAttached`, `onDeviceDetached`, `onDataReceived`, `onError`
- Permission-grant flow with proper `RECEIVER_NOT_EXPORTED` flag
- Driver type detection by vendor ID (FTDI=0x0403, CH340=0x1A86, CP210x=0x10C4)
- `Closeable` implementation with clean disconnect lifecycle
- `UsbForegroundParticipantModule`: Hilt module providing `UsbSerialManager` as a singleton, wired into Android's foreground service lifecycle

### Torvox Implementation

**File:** `android/app/src/main/java/io/torvox/usb/UsbSerialManager.kt` (325 lines)

**What it does:**
- Nearly identical USB Host API wrapper supporting CDC-ACM, FTDI, CH340/CH341, CP210x
- Same endpoint discovery logic (CDC_DATA, COMM, VENDOR_SPEC interfaces)
- Same background read thread with 4096 buffer, 1s timeouts
- Same `SerialListener` callback interface
- Same driver type detection by vendor ID
- Same `Closeable` lifecycle
- Same permission request with `RECEIVER_NOT_EXPORTED`

### Comparison

| Aspect | Haven | Torvox | Delta |
|--------|-------|--------|-------|
| Driver support | CDC-ACM, FTDI, CH340, CP210x | CDC-ACM, FTDI, CH340, CP210x | **Equal** |
| Read buffer size | 4096 | 4096 | Equal |
| Timeout values | 1s read/write | 1s read/write | Equal |
| Permission flow | PendingIntent + BroadcastReceiver | PendingIntent + BroadcastReceiver | Equal |
| Hilt/DI integration | `UsbForegroundParticipantModule` | None (manual instantiation) | Haven has DI |
| Test coverage | 3 tests (permission, attach, detach) | 0 tests | Gap |
| Code provenance | Original | "Referenced from Haven's USB subsystem" | Torvox credits Haven |

**Verdict:** Torvox's `UsbSerialManager` is a near-verbatim copy of Haven's. The only meaningful difference is Haven integrates it via Hilt DI (`UsbForegroundParticipantModule`) and has test coverage. Torvox's is standalone with no tests.

---

## 2. MCP (Model Context Protocol)

### Haven Implementation

**Files read:**
- `core/mcp/src/main/kotlin/sh/haven/core/mcp/McpServer.kt`
- `core/mcp/src/main/kotlin/sh/haven/core/mcp/McpSessionBridge.kt`
- `core/mcp/src/main/kotlin/sh/haven/core/mcp/SshSessionBridge.kt`

**What it does:**
- Kotlin MCP server over Unix domain sockets (stdio for AI agents)
- JSON-RPC 2.0 wire protocol
- Tools: `list_sessions`, `read_terminal`, `send_input`, `list_files`, `read_file`, `screenshot`, `serve_file`
- `McpSessionBridge`: connects MCP to Haven's SSH session manager, reads terminal grid, sends keystrokes
- `SshSessionBridge`: SFTP-backed file access for AI agents (list, read, serve files from remote)
- Background Kotlin coroutine server with `SelectorManager`

### Torvox Implementation

**Files:** `torvox-mcp/src/main.rs` (51 lines), `torvox-mcp/src/lib.rs` (1455+ lines)

**What it does:**
- Rust MCP server over Unix domain sockets, JSON-RPC 2.0
- `#![forbid(unsafe_code)]` — zero unsafe in the crate
- 17 tools exposed (vs Haven's ~7):
  - `list_sessions`, `read_grid`, `read_scrollback`, `read_cursor`, `read_selection`, `read_title`
  - `send_input`, `send_signal` (SIGINT/SIGTERM/SIGHUP/SIGQUIT)
  - `scrollback_search` (regex pattern matching)
  - `set_terminal_size`
  - `list_directory`, `read_file` (local filesystem)
  - `read_clipboard`, `write_clipboard`
  - `raise_notification`
  - `scroll_terminal`, `feed_terminal_output`
  - `queue_terminal_input` (prompt-matching input queue for AI automation)
  - `list_queued_inputs`, `cancel_queued_input`
- `SessionStore` trait for testability with `MockStore`
- `InputQueue`: prompt-pattern-matching queue that watches scrollback and injects text when patterns appear (inspired by Haven #161)
- Write consent gating (`--mcp-allow-write` flag)
- Full test suite: unit tests + quickcheck property tests

### Comparison

| Aspect | Haven | Torvox | Delta |
|--------|-------|--------|-------|
| Language | Kotlin | Rust | Different |
| Wire protocol | JSON-RPC 2.0 | JSON-RPC 2.0 | Equal |
| Transport | Unix socket | Unix socket | Equal |
| Tool count | ~7 | 17 | Torvox 2.4x more |
| File access | SFTP (remote) | Local filesystem | Haven has remote |
| Terminal read | Grid read | Grid + scrollback + cursor + selection + title | Torvox more granular |
| Terminal write | send_input | send_input + send_signal + feed_output + scroll | Torvox more granular |
| Search | None | scrollback_search (regex) | Torvox only |
| Resize | None | set_terminal_size | Torvox only |
| Clipboard | None | read/write clipboard | Torvox only |
| Notifications | None | raise_notification | Torvox only |
| AI automation | None | queue_terminal_input (prompt matching) | Torvox only |
| Safety | No explicit safety | `#![forbid(unsafe_code)]` + write consent | Torvox safer |
| Tests | Unknown | 10+ unit tests + quickcheck | Torvox better tested |
| Protocol version | Unknown | 2024-11-05 | Torvox explicit |

**Verdict:** Torvox's MCP implementation significantly exceeds Haven's in scope, safety, and tool count. Haven's only advantage is SFTP-based remote file access. Torvox added clipboard, notifications, signals, search, resize, and AI automation features that Haven lacks entirely.

---

## 3. File Manager

### Haven Implementation

**Files read:**
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/SftpViewModel.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/SftpScreen.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/FilterSection.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/CompressionSection.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/MediaActions.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/LocalPasteIO.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/LocalFileBackend.kt` (171 lines)
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/SftpTransport.kt` (131 lines)
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/FileBackend.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/transport/TransportSelector.kt`
- `feature/sftp/src/main/kotlin/sh/haven/feature/sftp/SftpStreamServer.kt`
- `core/rclone/src/main/kotlin/sh/haven/core/rclone/RcloneClient.kt` (592 lines)
- `core/rclone/src/main/kotlin/sh/haven/core/rclone/RcloneSessionManager.kt` (315 lines)

**What it does:**
- **Dual-transport architecture**: `FileBackend` trait with `LocalFileBackend` + `SftpTransport` implementations
- **LocalFileBackend**:
  - Synthetic root `/` listing: Internal Storage, Downloads, removable storage (USB/SD), PRoot rootfs, App Cache
  - `StorageManager` enumeration for removable volumes (API 30+)
  - Standard file ops: list, delete, mkdir, rename, readBytes, writeBytes, openInputStream, stat
  - Permission string construction (rwx)
- **SftpTransport**:
  - SSH SFTP session wrapper with list, upload, download, openInputStream, stat, mkdir, rename, delete, chmod, chown
  - Symlink resolution after list completes (avoids SFTP buffer corruption)
  - chown via SSH exec channel (SFTP chown requires numeric UID)
- **RcloneClient** (592 lines):
  - Full rclone RC JSON-RPC interface: listRemotes, createRemote, updateRemote, deleteRemote
  - File operations: listDirectory, mkdir, copyFile, deleteFile, deleteDir, moveFile, publicLink, directorySize
  - Sync operations: startSync, getJobStatus, cancelJob, resetStats (async with job IDs)
  - Transfer stats, errored transfers
  - Media server (HTTP streaming via rclone VFS)
  - DLNA server
  - SOCKS5 proxy routing through VPN tunnels
  - OAuth flow management with browser redirect capture via logcat monitoring
  - Remote capabilities caching
- **RcloneSessionManager** (315 lines):
  - Session lifecycle: register, connect, disconnect
  - OAuth flow with 5-minute timeout, browser URL capture, worker thread pool
  - Error surfacing with human-readable messages
  - Audit logging to Room database
- **SftpViewModel**: Full MVVM with search, filter (include/exclude patterns, size limits, bandwidth), compression, media actions
- **SftpScreen**: Compose UI with dual-pane, file operations, breadcrumbs, selection mode

### Torvox Implementation

**File:** `android/app/src/main/java/io/torvox/ui/FileManagerScreen.kt` (307 lines)

**What it does:**
- Single-file Compose screen for local file browsing
- Simple directory listing with `java.io.File.listFiles()`
- File preview (reads first 200 lines)
- Sort: directories first, then alphabetical by name
- Top bar with path display, back navigation, close button
- `FileManagerEntry` data class: name, isDirectory, size, lastModified

### Comparison

| Aspect | Haven | Torvox | Delta |
|--------|-------|--------|-------|
| Transport abstraction | `FileBackend` trait (local + SFTP) | Hardcoded `java.io.File` | Haven extensible |
| Remote file access | SFTP + rclone (cloud storage) | None | Haven only |
| Cloud storage | rclone (Google Drive, Dropbox, etc.) | None | Haven only |
| File operations | CRUD + chmod + chown + symlinks | Navigate + preview | Haven far richer |
| Search/filter | Include/exclude patterns, size limits | None | Haven only |
| Media features | Streaming, DLNA | None | Haven only |
| OAuth/VPN | OAuth flow + SOCKS5 proxy | None | Haven only |
| Compose UI | Full MVVM, dual-pane, breadcrumbs | Single screen, basic list | Haven more polished |
| Android storage | Synthetic root, StorageManager, PRoot | `getExternalStorageDirectory()` | Haven handles edge cases |
| Code size | ~1500+ lines across 12+ files | 307 lines, 1 file | Haven 5x larger |

**Verdict:** Haven's file management is a full-featured file manager with remote access, cloud storage, media streaming, and sophisticated filtering. Torvox has a minimal local-only file browser. This is the largest gap between the two projects. Torvox's implementation is a thin convenience screen, not a file management system.

---

## 4. Keyboard Toolbar

### Haven Implementation

**Files read:**
- `core/toolbar/src/main/kotlin/sh/haven/core/toolbar/KeyboardToolbar.kt` (1168+ lines)
- `core/toolbar/src/main/kotlin/sh/haven/core/toolbar/SnippetsBottomSheet.kt` (212 lines)

**What it does:**
- **Massive, feature-rich toolbar** (1168+ lines):
  - Two-row layout with aligned navigation block (Home/End/PgUp/PgDn + arrow keys)
  - Key repeat with 400ms delay, 80ms interval (MotionEvent interop for reliable repeat in scrollable parent)
  - Modifier keys: Ctrl, Alt, Shift, AltGr (with visual toggle states)
  - Special keys: Esc, Enter (⏎), Tab, Insert, Delete, F1-F12
  - Voice/secure keyboard toggle (lock icon)
  - Raw keyboard mode
  - Clipboard paste with bracket paste mode (`\e[200~` ... `\e[201~`)
  - Attach button (file picker via SAF)
  - Snippets sheet
  - Custom key support (user-defined keys with any label + send value)
  - Reorder mode (drag-and-drop toolbar customization)
  - VNC/RDP desktop key integration
  - Keyboard show/hide toggle
  - Minimum key width setting
  - Nav block modes: ALIGNED (grid layout) vs STACKED
  - Edit mode controls placement (LEFT/RIGHT)
  - Desktop key placement (LEFT/RIGHT/HIDDEN)
  - DECCKM-aware key dispatch via `onDispatchKey`
  - `ToolbarCallbacks` data class bundling all callbacks
  - `CompositionLocal` for toolbar callbacks
  - Uniform `ToolbarKeyButton` primitive with tonal Surface styling
- **SnippetsBottomSheet** (212 lines):
  - Modal bottom sheet with search filtering
  - Add/delete snippets
  - Label + command fields with escape sequence help (`\n` for Enter, `\u001b` for Escape)
  - Snippet library (on-toolbar + off-toolbar)
  - Display send sequence preview

### Torvox Implementation

**Files:** None found. Torvox has no dedicated toolbar implementation. The terminal UI appears to rely on the standard Android soft keyboard without a custom toolbar.

### Comparison

| Aspect | Haven | Torvox | Delta |
|--------|-------|--------|-------|
| Custom toolbar | Full 2-row, 1168+ line implementation | None | Haven only |
| Modifier keys | Ctrl, Alt, Shift, AltGr | None | Haven only |
| Navigation block | Aligned grid with key repeat | None | Haven only |
| F-keys | F1-F12 | None | Haven only |
| Snippets | Bottom sheet with search, add/delete | None | Haven only |
| Custom keys | User-defined with reorder mode | None | Haven only |
| Bracket paste | Supported | None | Haven only |
| Voice/secure toggle | Supported | None | Haven only |
| VNC integration | Desktop key with loading state | None | Haven only |
| File attach | SAF picker integration | None | Haven only |
| Key repeat | MotionEvent-based with delay/interval | None | Haven only |

**Verdict:** This is the second-largest gap. Haven has a sophisticated, fully customizable keyboard toolbar that is essential for terminal use on Android (where the soft keyboard lacks many keys torvox needs). Torvox has no equivalent — users must rely on the system keyboard, which lacks Esc, Ctrl, Tab, arrow keys, F-keys, and custom snippets. This is a critical UX gap for a terminal emulator.

---

## 5. Summary: Feature Gaps

### Torvox Advantages over Haven

1. **MCP Server**: 17 tools vs ~7, `#![forbid(unsafe_code)]`, clipboard/signals/search/resize/notifications/AI automation, full test suite
2. **Code Safety**: `torvox-core` is `#![no_std]` with zero `unsafe`, Rust memory safety throughout
3. **Render Pipeline**: GPU-only via wgpu (Haven uses Android Canvas)
4. **Architecture**: Strict one-way crate dependency graph, `no_std` core

### Haven Advantages over Torvox

1. **File Management**: Full SFTP + rclone cloud storage + media streaming + DLNA vs basic local browser
2. **Keyboard Toolbar**: 1168+ line customizable toolbar vs nothing
3. **USB Integration**: Hilt DI + test coverage vs standalone copy
4. **SFTP**: Remote file operations, SSH exec, symlink resolution
5. **VPN/Proxy**: SOCKS5 tunnel integration for rclone traffic
6. **OAuth Flow**: Browser redirect capture via logcat for cloud storage auth

### Priority Gaps for Torvox

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| P0 | Keyboard toolbar (Esc, Ctrl, Tab, arrows, F-keys) | High | Critical for terminal usability |
| P1 | Snippets system | Medium | Power user essential |
| P2 | SFTP/remote file access | High | Remote workflow support |
| P3 | Cloud storage (rclone) | Very High | Niche but differentiating |
| P4 | USB test coverage | Low | Quality gate |

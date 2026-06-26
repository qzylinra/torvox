# Haven Deep Review — Reference for Torvox

> Reference project: `/tmp/reference-projects/Haven/`
> Review date: 2026-06-24
> Focus: Features torvox needs to implement

---

## Summary

| Metric | Value |
|--------|-------|
| Total source files in Haven | 583 (Kotlin + Rust) |
| Files directly read & analyzed | 12 core files |
| Key features torvox is missing | USB device connection, GUI file browser, in-app text search, OSC 133 semantic awareness, MCP consent/audit framework |
| Priority | P0 (USB), P1 (search + file browser), P1 (OSC 133 surface), P2 (MCP hardening) |

### Key Features Torvox Is Missing

1. **USB device proxy** — Haven has a complete USB broker/proxy/IP-server stack (6 source files in `core/usb/`) that lets a proot guest and MCP agents reach phone-attached USB devices. Torvox has zero USB code.
2. **In-app text search** — Haven's terminal provides scrollback search via `read_terminal_scrollback` + `scrollback_search` MCP tools and a UI search button. Torvox's MCP has `scrollback_search` in the protocol but no UI search button in the session panel.
3. **GUI file browser** — Haven has a full SFTP/file-browser feature module (`feature/sftp/`) with `ShellFileBrowser`, transport abstraction (SFTP/SMB/rclone/local), upload/download, and an `open_file_in_editor` tool. Torvox has `list_directory`/`read_file` MCP tools but no GUI file browser panel.
4. **OSC 133 semantic exposure** — Haven's `read_terminal_snapshot` MCP tool exposes OSC 133 prompt/input/output segments per line. Torvox reads `SemanticContent` internally (`ghostty_terminal.rs:580`) but does not expose it to MCP or the bridge.
5. **MCP consent/audit framework** — Haven has a tiered consent system (`ConsentLevel::NEVER/ONCE_PER_SESSION/EVERY_CALL`), agent audit recorder, standing policy enforcer, paired-client allowlist, and session management. Torvox has a simple `write_consent` boolean flag.

### Priority Recommendations

| Priority | Feature | Effort | Rationale |
|----------|---------|--------|-----------|
| P0 | USB device connection + settings toggle | Large | Differentiating feature; Haven's `core/usb/` is the reference |
| P1 | In-app text search (scrollback) | Medium | High user value; torvox MCP has the backend, needs UI button |
| P1 | OSC 133 semantic segments in MCP | Small | Torvox already reads them; just needs bridge surface |
| P1 | GUI file browser | Large | Haven's `feature/sftp/` is the reference; torvox has MCP primitives |
| P2 | MCP consent tiers + audit log | Medium | Security hardening for agent integration |
| P2 | MCP session management (UUID, TTL) | Small | Torvox uses a simple per-connection model |

---

## 1. USB Device Connection

### Haven's Architecture (6 files in `core/usb/`)

```
UsbBroker.kt          — Android UsbManager owner, permission handling, raw transfers
UsbAccessGate.kt      — Exclusive-access coordination (FIDO vs USB/IP)
UsbModels.kt          — USB device descriptor models
UsbProxyProtocol.kt   — Proxy framing over LocalSocket
UsbProxyServer.kt     — Userspace USB proxy for proot guest
UsbIpServer.kt        — USB/IP network export for remote guests
```

**Key patterns in Haven:**

- `UsbBroker` (`core/usb/UsbBroker.kt:46-351`): Singleton, Hilt-injected. Owns `UsbManager`, handles `FLAG_MUTABLE` PendingIntent for permission, broadcasts `ACTION_USB_DEVICE_DETACHED` to clean up. Exposes `enumerate()`, `openDevice()`, `claimInterface()`, `bulkTransfer()`, `controlTransfer()`.
- `UsbAccessGate` (`core/usb/UsbAccessGate.kt:28-67`): ReentrantLock + Condition for FIDO/auth contention. `acquire()`/`release()` lease model, `awaitClear()` with timeout.
- `UsbProxyServer` (line 153 of McpTools): Constructed lazily from `UsbBroker`, served via `usb_attach_to_guest` MCP tool.
- Settings toggle: `usb_guest_exposure_enabled` preference, read via `get_preference` tool.
- MCP tools: `usb_list_devices`, `usb_attach_to_guest`, `usb_detach_from_guest`.

**What torvox lacks:**

- No USB crate or module at all (`grep` for `usb|USB|hidraw` returns zero hits in the entire torvox repo).
- No Android USB permission handling.
- No settings toggle for USB.
- No MCP tool surface for USB devices.

**Implementation plan for torvox:**

1. Create `torvox-usb` crate (Rust, `no_std` compatible for core types):
   - `UsbDevice` descriptor model
   - `UsbBroker` trait (Android FFI via JNI)
   - `UsbProxyProtocol` framing
2. Add `android/app/src/main/kotlin/.../UsbService.kt`:
   - Android `UsbManager` integration
   - Permission request flow with `FLAG_MUTABLE`
   - Detach receiver
3. Add settings toggle in `TorvoxBridge.kt`:
   - `usb_guest_exposure_enabled` preference
   - JNI bridge for `listUsbDevices()`, `attachUsbDevice()`, `detachUsbDevice()`
4. Add MCP tools in `torvox-mcp`:
   - `usb_list_devices`, `usb_attach_to_guest`, `usb_detach_from_guest`

---

## 2. File Management

### Haven's Architecture

**`feature/sftp/` module** — Full file browser with:
- `SftpScreen.kt` — Compose UI (list, navigate, upload, download, delete, rename)
- `SftpViewModel.kt` — State management
- `transport/` abstraction: `FileBackend`, `SftpTransport`, `SmbFileBackend`, `RcloneFileBackend`, `LocalFileBackend`, `RcloneFileBackend`
- `ShellFileBrowser.kt` — Fallback `ls -la` parser for servers without SFTP

**`core/ssh/ShellFileBrowser.kt`** (`core/ssh/src/main/kotlin/.../ShellFileBrowser.kt:20-157`):
- Parses `ls -la --time-style=full-iso` output into `Entry(name, size, modifiedTimeSeconds, isDirectory, isSymlink, permissions, owner, group)`
- Fallback to `ls -1` for non-GNU systems
- Used when SFTP subsystem is unavailable

**MCP tools for file management:**
- `list_directory` — Unified: local, SSH/SFTP, SMB, rclone (by profileId)
- `read_file` — Read file content
- `upload_file` — Write to any backend
- `delete_file` — Delete file
- `serve_file` — Expose file as HTTP URL for agent download
- `navigate_sftp_browser` — Switch to Files tab
- `open_file_in_editor` — Open text file in built-in editor
- `open_convert_dialog_with_args` — Stage ffmpeg conversion

**What torvox has:**
- `list_directory` and `read_file` MCP tools (`torvox-mcp/src/lib.rs:537-545`)
- `DirEntry` and `FileContent` response types (`lib.rs:216-206`)
- But these are LOCAL filesystem only — no SSH/SFTP/remote backends
- No GUI file browser panel
- No upload/delete/rename capabilities
- No transport abstraction

**What torvox is missing:**
1. No GUI file browser (Compose panel in session view)
2. No remote file backends (SSH/SFTP)
3. No upload/download/delete/rename
4. No file viewer/editor
5. No transport abstraction layer

**Implementation plan for torvox:**
1. Add file browser panel to session Compose UI (button in session panel → overlay)
2. Implement local file browsing first (uses existing MCP `list_directory`/`read_file`)
3. Add upload/download via JNI bridge
4. Later: SSH/SFTP backend via `torvox-terminal`'s existing SSH connection

---

## 3. Text Search (Software-Level)

### Haven's Architecture

**MCP tools:**
- `scrollback_search` — Regex search of scrollback buffer, returns `SearchMatch { line_number, text, start_col, end_col }`
- `read_terminal_scrollback` — Raw bytes of SSH stdout
- `read_terminal_snapshot` — Structured snapshot with optional OSC 133 semantic segments

**OscHandler** (`feature/terminal/src/main/kotlin/.../OscHandler.kt:32-411`):
- State machine that parses OSC sequences from terminal output
- Handles: OSC 52 (clipboard), OSC 7 (cwd), OSC 8 (hyperlinks), OSC 9/777 (notifications)
- Strips handled OSCs from output stream
- Retains last-seen values for agent assertions

**What torvox has:**
- `scrollback_search` MCP tool (`lib.rs:759-782`) — regex search, returns `SearchMatch`
- `ReadRequest::ScrollbackSearch` variant
- But NO UI search button in the session panel
- No highlight of matches in the terminal view

**What torvox is missing:**
1. UI search button in session panel toolbar
2. Search overlay with input field + next/prev navigation
3. Match highlighting in the terminal grid
4. Keyboard shortcut for search (Ctrl+F equivalent)

**Implementation plan for torvox:**
1. Add search icon button to session panel toolbar (in Kotlin Compose UI)
2. Implement search overlay: text field + match count + next/prev buttons
3. Highlight matches in GridSnapshot by modifying `CellSnapshot` with a search-match flag
4. Add JNI bridge method for search to pass results to Rust renderer

---

## 4. OSC 133 (Shell Integration)

### Haven's Architecture

**OscHandler** (`feature/terminal/src/main/kotlin/.../OscHandler.kt`):
- Processes terminal output byte-by-byte through a state machine
- Detects and dispatches: OSC 52, 7, 8, 9, 777
- Unhandled OSC (including 133) passes through to terminal emulator
- OSC 133 is handled by the terminal emulator itself, not the OscHandler

**TerminalSession** (`core/ssh/src/main/kotlin/.../TerminalSession.kt:150-163`):
- Strips ANSI/OSC escapes before checking for prompt characters
- Detects prompt chars: `$`, `#`, `%`, `>`, `❯` (fish/starship)
- Uses this to trigger pending command delivery
- OSC 133-aware prompt detection: strips escape codes first, then checks for prompt chars

**MCP integration:**
- `read_terminal_snapshot` with `includeSemanticSegments: true` returns OSC 133 markers per line
- `TerminalSessionRegistry` tracks sessions for MCP access

**What torvox has:**
- `SemanticContent` enum in `ghostty_terminal.rs:44-49` (Output, Input, Prompt)
- `CellSnapshot.semantic` field (`ghostty_terminal.rs:65`)
- `read_semantic_content()` reads OSC 133 from grid (`ghostty_terminal.rs:580-584`)
- `poll_shell_integration()` in `session.rs:406` returns `ShellIntegration` state
- `ShellIntegration` exposed via JNI bridge

**What torvox is missing:**
1. MCP tool does NOT expose OSC 133 semantic segments per line
2. No `read_terminal_snapshot` MCP tool (Haven's equivalent)
3. Semantic content not exposed in bridge for AI agent consumption
4. No shell integration state exposed to MCP

**Implementation plan for torvox:**
1. Add `read_terminal_snapshot` MCP tool (or enhance existing `read_grid`) to include `SemanticContent` per cell
2. Add `semantic_segments` field to grid snapshot response
3. Expose `ShellIntegration` state via MCP tool
4. Add `includeSemanticSegments` option to existing grid/scrollback tools

---

## 5. MCP Agent Integration

### Haven's Architecture

**McpServer** (`app/src/main/kotlin/.../McpServer.kt:128-1087+`):
- Streamable HTTP transport (POST /mcp endpoint)
- Session management: UUID minted on `initialize`, tracked in `ConcurrentHashMap<String, Session>`
- Loopback auto-trust (`trustLoopbackEnabled` flag)
- LAN bind + WireGuard bind for remote access
- Consent framework: `ConsentLevel::NEVER/ONCE_PER_SESSION/EVERY_CALL`
- `AgentAuditRecorder` logs every tool call with outcome
- `StandingPolicyEnforcer` for tier-3 standing policies
- Paired client allowlist with pairing prompts
- 8730-8739 port range for reconnection

**McpTools** (`app/src/main/kotlin/.../McpTools.kt:68-1236+`):
- `ToolHandler` pattern: name + description + inputSchema + consentLevel + summarise lambda + handler
- 60+ tools covering: connections, sessions, file ops, rclone, mail, USB, desktop, terminal, workspace
- Each tool has a `consentLevel` that gates interactive prompts
- `summarise` lambdas produce human-readable consent prompts

**Key MCP tools (beyond what torvox has):**
- `list_connections` — All saved profiles
- `connect_profile` / `disconnect_profile` — Session lifecycle
- `list_desktop_sessions` — VNC/RDP tabs
- `serve_file` — HTTP URL for agent file download
- `present_media` / `present_web` / `present_app` — Agent→user presentation
- `usb_list_devices` / `usb_attach_to_guest` / `usb_detach_from_guest`
- `queue_terminal_input` — Prompt-matching input injection
- `upload_file` / `delete_file` — File mutation
- `add_port_forward` / `remove_port_forward` — SSH port forwarding
- `navigate_sftp_browser` / `open_file_in_editor` — UI navigation
- `read_terminal_snapshot` — Structured terminal state with OSC 133

**What torvox has (`torvox-mcp/src/lib.rs`):**
- 21 tools (vs Haven's 60+)
- Unix domain socket transport (vs Haven's HTTP)
- `write_consent` boolean (vs Haven's tiered consent)
- `SessionStore` trait for testability
- `InputQueue` for prompt-matching (inspired by Haven #161)
- No session UUID management
- No audit logging
- No client pairing/allowlist

**What torvox is missing:**
1. **Session management** — No UUID tracking, no TTL, no client identity
2. **Consent tiers** — Only boolean, not NEVER/ONCE_PER_SESSION/EVERY_CALL
3. **Audit logging** — No record of tool calls/outcomes
4. **Client pairing** — Anyone on the socket can call any tool
5. **HTTP transport** — Only Unix sockets (may be fine for Android)
6. **LAN/WireGuard bind** — Only loopback
7. **UI navigation tools** — No `navigate_*` / `focus_*` tools
8. **File mutation tools** — No upload/delete
9. **Presentation tools** — No `present_media`/`present_web`
10. **USB MCP tools** — None
11. **Port forwarding tools** — None

**Implementation plan for torvox:**
1. Add session UUID tracking with TTL (small, high value)
2. Add per-tool consent levels (NEVER/ONCE/EVERY_CALL)
3. Add audit log (ring buffer or file-backed)
4. Add client pairing prompt (for LAN access)
5. Add missing MCP tools incrementally, prioritizing:
   - `upload_file` / `delete_file` (P1)
   - `navigate_session` / `focus_session` (P1)
   - `usb_list_devices` / `usb_attach_to_guest` (P0)
   - `read_terminal_snapshot` with OSC 133 (P1)

---

## 6. Comparison Matrix

| Feature | Haven | Torvox | Gap |
|---------|-------|--------|-----|
| USB device proxy | Full stack (6 files) | None | Complete gap |
| File browser (GUI) | Full SFTP module | None (MCP only) | Large gap |
| Text search (UI) | Implied by MCP tools | MCP backend only, no UI | Medium gap |
| OSC 133 | OscHandler + MCP exposure | Reads internally, not exposed | Medium gap |
| MCP tools | 60+ tools | 21 tools | 39 tools missing |
| MCP transport | HTTP + Unix socket | Unix socket only | Different design |
| Consent system | 3-tier + audit | Boolean flag | Large gap |
| Client pairing | Allowlist + prompts | None | Large gap |
| Session management | UUID + TTL | Per-connection | Medium gap |
| Audit logging | AgentAuditRecorder | None | Medium gap |
| Remote backends | SSH/SMB/rclone/local | Local only | Large gap |
| Terminal reconnect | Channel swap + resize replay | N/A (local PTY) | N/A |
| OSC handling | OscHandler (52/7/8/9/777) | libghostty-vt (built-in) | Different approach |
| Shell integration | OSC 133 + prompt detection | SemanticContent enum | Partial |

---

## 7. Key Haven Patterns Worth Adopting

### 7.1 ToolHandler Pattern
```kotlin
ToolHandler(
    description = "...",
    inputSchema = JSONObject().apply { ... },
    consentLevel = ConsentLevel.EVERY_CALL,
    summarise = { args -> "Human-readable prompt" },
) { args -> handler(args) }
```
**Torvox equivalent:** Add `consent_level` and `description` to the tool list in `lib.rs`.

### 7.2 ShellFileBrowser Fallback
Parse `ls -la` output as a fallback when SFTP is unavailable. Simple, robust, works on any POSIX system.
**Torvox:** Could use this for remote file browsing over SSH exec channel.

### 7.3 UsbAccessGate Coordination
ReentrantLock + Condition for exclusive USB access between consumers (FIDO auth vs USB/IP export).
**Torvox:** Needed when USB support is added.

### 7.4 OscHandler State Machine
Byte-by-byte OSC parsing with buffer boundary handling, payload cap (1MB), and passthrough for unhandled sequences.
**Torvox:** libghostty-vt handles this internally; torvox just needs to expose the results.

### 7.5 TerminalSession Prompt Detection
Strips ANSI/OSC escapes, checks each line for prompt chars, triggers pending command delivery.
**Torvox:** Could enhance `SemanticContent::Prompt` usage for MCP agent automation.

---

## 8. Files Read During Review

| # | File | Lines | Key Takeaway |
|---|------|-------|--------------|
| 1 | `core/usb/UsbBroker.kt` | 351 | Android UsbManager ownership, permission, transfers |
| 2 | `core/usb/UsbAccessGate.kt` | 67 | Exclusive USB access coordination |
| 3 | `core/usb/UsbModels.kt` | ~50 | Device descriptor models |
| 4 | `core/usb/UsbProxyProtocol.kt` | ~100 | Proxy framing protocol |
| 5 | `core/usb/UsbProxyServer.kt` | ~200 | Userspace USB proxy for guest |
| 6 | `core/usb/UsbIpServer.kt` | ~200 | USB/IP network export |
| 7 | `core/ssh/ShellFileBrowser.kt` | 157 | `ls -la` parser for remote file listing |
| 8 | `core/ssh/TerminalSession.kt` | 380 | SSH channel bridging, reconnect, prompt detection |
| 9 | `app/agent/McpServer.kt` | 1087+ | HTTP MCP server, session mgmt, consent, audit |
| 10 | `app/agent/McpTools.kt` | 1236+ | 60+ MCP tool implementations |
| 11 | `feature/terminal/OscHandler.kt` | 411 | OSC sequence parser (52/7/8/9/777) |
| 12 | `torvox-mcp/src/lib.rs` | 1681 | Torvox MCP server (21 tools) |

# LetsFLUTssh Reference Review

**Date:** 2026-06-24
**Reviewed by:** opencode
**Project:** LetsFLUTssh (Flutter SSH terminal client)
**Reference to:** torvox (Rust-based terminal emulator for Android)

---

## Summary

| Metric | Value |
|--------|-------|
| Total Dart files reviewed | ~35 |
| Total Rust files reviewed | ~12 |
| Total C/C++ files | 0 (pure Dart + Rust) |

### Key Features torvox Is Missing

LetsFLUTssh is a **full SSH client** while torvox is a **local terminal emulator**. The feature gap is architectural, not incremental:

1. **SSH connection management** — Host key TOFU verification, connection lifecycle actor with state machine, progress tracking per phase
2. **SFTP file browser** — Full remote filesystem operations (list, upload, download, mkdir, rename, remove, recursive walk)
3. **Key manager** — Ed25519/RSA generation, PEM/PPK import, FIDO2 sk-* hardware keys, PKCS#11 tokens, TPM 2.0, Apple Secure Enclave, Windows Hello, Android Keystore
4. **OpenSSH config parser** — `~/.ssh/config` import with `Include` directive expansion, wildcard `Host *` cascade
5. **Known hosts manager** — TOFU (Trust On First Use) with hashed hostname support, import/export, key-changed detection
6. **Port forwarding** — `-L` local, `-D` dynamic (SOCKS5), `-R` remote, with per-rule enable/disable
7. **ProxyJump / bastion** — Multi-hop SSH with cycle detection and depth limits
8. **Session management** — Persistent sessions with encrypted credential storage, folder organization, notes
9. **Terminal recording / replay** — Session recording to disk with replay viewer
10. **Multi-transport** — SSH + WebDAV + S3 backends for file browsing

### Bugs / Anti-patterns torvox Should Avoid

1. **No bugs found in LetsFLUTssh source** — the codebase is well-structured with extensive tests
2. **Over-engineering risk** — LetsFLUTssh's `ConnectionActor` + `EventBus` + `PromptRegistry` pattern is heavy for torvox's simpler scope
3. **FRB boundary complexity** — LetsFLUTssh crosses the Flutter-Rust bridge extensively; torvox should keep Rust-owned data (grid, cell) to minimize bridge traffic

### Best Practices torvox Should Adopt

1. **Structured error types** — LetsFLUTssh's `SSHError` hierarchy (`AuthError`, `ConnectError`, `HostKeyError`, `ProxyJumpCycleError`) with `userMessage` getter for clean UI display
2. **Adapter pattern for terminal views** — `TerminalController` abstract class with `LiveTerminalController` and `ReplayTerminalController` — same view renders live shells and read-only replays
3. **Progress tracker per-consumer** — `ProgressTracker` merges shared transport steps with consumer-local channel steps (e.g. "Opening SFTP channel") so each pane shows its own context
4. **Feature flags via config** — `TerminalViewConfig` with `interactive`, `selectable`, `pasteable`, `mouseReportable`, `searchable`, `showCursor` — composable capabilities per surface
5. **Content-addressable key fingerprinting** — SHA-256 of normalized text for dedup, separate from display fingerprint
6. **Bus event coalescing** — `notify_changed` pattern with `EventBus` + consumer-side collapse to avoid redundant refreshes
7. **Defensive PPK Argon2 validation** — Cap memory/passes/parallelism before the KDF runs to prevent DoS from hostile `.ppk` files
8. **File size guards on import** — 32 KiB cap for key files, 16 KiB for certificates — prevents accidentally loading binaries

### Priority Recommendations

| Priority | Recommendation | Rationale |
|----------|---------------|-----------|
| **P1** | Adopt `TerminalController` adapter pattern | torvox's `TerminalView` could benefit from the live/replay split if recording is added |
| **P1** | Add structured error types to `SessionError` | Currently flat `#[derive(Error)]`; typed variants improve UI error handling |
| **P2** | Implement `TerminalViewConfig` capability flags | Enables read-only replay surfaces without duplicating widget code |
| **P2** | Add progress tracking to session lifecycle | torvox's session spawn has no multi-phase progress reporting |
| **P3** | Study SFTP transfer queue for future file transfer | If torvox adds remote file access, the worker-pool + progress-throttle pattern is proven |
| **P3** | Study known_hosts TOFU pattern | If torvox adds SSH, the `PromptRegistry` + oneshot pattern is clean |

---

## Detailed File Analysis

### 1. SSH Transport & Connection

**LetsFLUTssh files:**
- `lib/core/ssh/transport/ssh_transport.dart` (393 lines) — Abstract `SshTransport` with `openShell`, `openSftp`, `openDirectTcpip`, `requestRemoteForward`, `disconnect`
- `lib/core/ssh/transport/rust_transport.dart` — Concrete implementation backed by russh
- `rust/crates/lfs_core/src/connection/mod.rs` (1624 lines) — `ConnectionActor` state machine with `Disconnected → Connecting → Connected → Disconnected`
- `rust/crates/lfs_core/src/connection/auth_compose.rs` — Auth method composition

**Key patterns:**
- Connection actor owns the state machine; Dart mirrors state via bus events
- Transport monitor thread watches for silent socket death (sleeping laptop case)
- Generation counter prevents stale reconnect results
- Auth methods are a sealed class hierarchy (`SshAuthMethod` with 11 variants: password, pubkey, cert, sk-*, pkcs11, enclave, hello, tpm, keystore, agent)

**torvox comparison:**
- torvox has no SSH networking — local PTY only via `torvox-terminal/src/pty.rs`
- `SessionError` is flat (5 variants) vs LetsFLUTssh's typed hierarchy
- No connection lifecycle management needed for local PTY

**Recommendation:** If torvox adds SSH later, adopt the connection actor + bus event pattern. The sealed auth method hierarchy is excellent for exhaustive matching.

### 2. Terminal View & Controller

**LetsFLUTssh files:**
- `lib/widgets/terminal/terminal_view.dart` (797 lines) — Single renderer over `TerminalController`
- `lib/widgets/terminal/terminal_controller.dart` (339 lines) — Abstract `TerminalController` with `LiveTerminalController` and `ReplayTerminalController`
- `lib/widgets/terminal/terminal_grid_painter.dart` — Grid rendering via `CustomPaint`
- `lib/widgets/terminal/terminal_pointer_input.dart` — Mouse tracking, selection, tap counting

**Key patterns:**
- **Adapter pattern:** `TerminalController` abstract class; live sessions implement `sendKey`, `sendMouse`, `paste`, `search`; replay sessions no-op these
- **Config flags:** `TerminalViewConfig` with 6 boolean flags compose exact capability per surface
- **Grid is Rust-owned:** Controller pulls a fresh `snapshot()` each repaint; never caches frame data Dart-side
- **Repaint coalescing:** `_scheduleRepaint` gates one frame pull per vsync even if controller notified many times
- **`scheduleFrame()` for idle streaming:** Forces a frame when app is otherwise idle so streamed output repaints on its own
- **Mouse tracking vs local selection:** Shift key forces local selection even in mouse-tracking mode
- **Multi-tap selection:** Single-click = char, double = word, triple = line, with timing window
- **Right-click always opens context menu** even under mouse tracking (TUI apps don't use right-click)

**torvox comparison:**
- torvox's `TerminalView` is Android Compose-based, not Flutter
- `Session` owns the terminal directly; no adapter pattern needed (no replay)
- `DirtyMask` + `Condvar` wake pattern is similar to LetsFLUTssh's `repaint` + `scheduleFrame`

**Recommendation:** Adopt `TerminalViewConfig` capability flags concept. If torvox adds recording/replay, the adapter pattern is essential. The repaint coalescing pattern (gate one pull per vsync + force frame when idle) is directly applicable.

### 3. Key Management

**LetsFLUTssh files:**
- `rust/crates/lfs_core/src/keys.rs` (1032 lines) — Ed25519/RSA generation, PEM/PPK import, sk-* parsing, cert parsing
- `lib/core/security/ssh_key.dart` (417 lines) — Key entry model with certificate, FIDO2, PKCS#11, TPM, Enclave, Keystore backends

**Key patterns:**
- **KeyMaterial struct:** Unified output shape for generation/import (private PEM + public OpenSSH + algorithm name)
- **PPK Argon2 DoS protection:** Validates `Argon2-Memory`, `Argon2-Passes`, `Argon2-Parallelism` caps before `russh-keys` touches the file
- **File size guards:** 32 KiB for key files, 16 KiB for certificates
- **`is_encrypted_pem`:** Covers PKCS#1, PKCS#8, new OpenSSH format — single function for all encryption detection
- **`normalized_text_fingerprint`:** SHA-256 of CRLF→LF + trimmed text for content-addressable dedup
- **`is_obvious_non_key_filename`:** Pre-filter to skip `.pub`, `config`, `authorized_keys*`, `known_hosts*`
- **Certificate support:** Parse `id_*-cert.pub`, extract principals/validity/critical_options, verify cert-key binding via fingerprint comparison
- **Agent policy per-key:** `always` / `ask` / `deny` for in-process ssh-agent endpoint

**torvox comparison:**
- torvox has no SSH key management (local terminal only)
- If added, the key type hierarchy (software → FIDO2 → PKCS#11 → TPM → Enclave → Hello → Keystore) is the gold standard

**Recommendation:** Study the PPK Argon2 validation pattern if torvox ever handles untrusted key files. The `is_encrypted_pem` multi-format detection is a useful utility.

### 4. SFTP / File Transfer

**LetsFLUTssh files:**
- `lib/core/sftp/sftp_fs.dart` (434 lines) — `RemoteSftpFs` abstract class with `RustSftpFs` implementation
- `lib/core/sftp/sftp_models.dart` (112 lines) — `FileEntry`, `FlatFileLeaf`, `TransferProgress`
- `lib/core/sftp/file_system.dart` (147 lines) — `FileSystem` abstract class with `FileSystemCapabilities`
- `rust/crates/lfs_core/src/transfer/mod.rs` (678 lines) — Transfer queue with worker pool
- `rust/crates/lfs_core/src/sftp/mod.rs` — SFTP client operations

**Key patterns:**
- **Recursive walk Rust-side:** `flatWalkFiles` runs in a single FRB call instead of N Dart-side `list` round-trips
- **Capability struct:** `FileSystemCapabilities` with `posixMode` and `owner` booleans — backends declare what they support
- **Transfer progress throttling:** `PROGRESS_BYTES_THRESHOLD` (256 KiB) + `PROGRESS_TIME_THRESHOLD` (100ms) caps bus event rate
- **Worker pool sizing:** Read from config store, clamped to `[1, MAX_TRANSFER_WORKERS]`
- **Symlink awareness:** `FileEntry.isSymlink` flag prevents recursing through symlinked directories

**torvox comparison:**
- torvox has no file transfer (local terminal only)
- The progress throttling pattern (byte delta + time delta) is useful for any streaming progress reporting

**Recommendation:** The transfer progress throttle pattern is directly applicable if torvox adds any download/upload feature. The capability struct pattern is good for future feature negotiation.

### 5. Known Hosts / TOFU

**LetsFLUTssh files:**
- `rust/crates/lfs_core/src/known_hosts.rs` (410 lines) — Import/export, TOFU prompt protocol, `PromptRegistry`

**Key patterns:**
- **`PromptRegistry`:** Process-singleton of pending TOFU prompts keyed by UUID; `register` returns a oneshot receiver, `resolve` wakes it
- **`HostCheckResult` enum:** `Accepted` vs `Mismatch(Unknown | Changed)` — exhaustive, compile-time safe
- **Additive import:** Never overwrites a TOFU-accepted entry with a possibly-stale paste
- **Hashed hostname detection:** Counts skipped hashed lines separately from parse failures
- **IPv6 bracket handling:** `split_host_port` handles `[::1]:2222` correctly with `find(']')` instead of `rsplit_once(':')`

**torvox comparison:**
- torvox has no SSH host key verification
- The `PromptRegistry` + oneshot pattern is clean for any async user-prompt flow

**Recommendation:** If torvox adds SSH, the `PromptRegistry` pattern (register oneshot → publish event → await → resolve) is the correct async TOFU prompt mechanism.

### 6. Port Forwarding

**LetsFLUTssh files:**
- `lib/core/ssh/port_forward_rule.dart` (130 lines) — Immutable rule model with validation
- `lib/core/ssh/port_forward_runtime.dart` (154 lines) — Lifecycle management via `ConnectionExtension`

**Key patterns:**
- **Validation delegated to Rust:** `portForwardValidateRule` is a single source shared by UI and runtime
- **Loopback-only warning:** `bindsLoopbackOnly` getter for UI safety check when user types `0.0.0.0`
- **`ConnectionExtension` trait:** `onConnected` / `onDisconnecting` / `onReconnecting` hooks
- **Armed rules tracking:** `_armed` map of rule-id → kind; teardown stops exactly the armed rules

**torvox comparison:**
- torvox has no port forwarding
- The validation-delegation pattern (Rust validates, Dart displays) is good for any shared validation logic

**Recommendation:** No immediate need. The `ConnectionExtension` hook pattern is good for any plugin/extension lifecycle.

### 7. Error Handling

**LetsFLUTssh files:**
- `lib/core/ssh/errors.dart` (102 lines) — `SSHError` hierarchy with `userMessage` getter

**Key patterns:**
- **Root cause unwrapping:** `_rootCauseMessage` recursively unwraps `SSHError` chains
- **Prefix stripping:** Removes `SocketException:`, `SSHAuthFailError:`, etc. for cleaner display
- **Typed subclasses:** `AuthError(user, host)`, `ConnectError(host, port)`, `HostKeyError(host, port)`, `ProxyJumpCycleError(offendingSessionId)`, `ProxyJumpDepthError(depth)`, `ProxyJumpBastionError(bastionLabel, cause)`, `HardwareKeyPromptCancelled`

**torvox comparison:**
- torvox's `SessionError` has 5 flat variants: `Pty`, `Io`, `Ghostty`, `Closed`
- `PtyError` is separate with `Open`, `Resize`, `Write`, `Spawn` variants

**Recommendation:** Add a `user_message()` or `display_message()` method to `SessionError` for clean UI error display. Consider splitting `Io` into `Io(std::io::Error)` and `Network(String)` for SSH-era errors.

### 8. Terminal Controller & Events

**LetsFLUTssh files:**
- `lib/widgets/terminal/terminal_controller.dart` — `LiveTerminalController` wraps `TerminalSession`, `ReplayTerminalController` wraps `TerminalReplay`
- `rust/crates/lfs_core/src/terminal/frame.rs` — `TerminalFrame` snapshot
- `rust/crates/lfs_core/src/terminal/input.rs` — Key encoding

**Key patterns:**
- **Single subscription:** `_session.events()` subscribed once in constructor; `Wakeup` → bump `repaint`, others → `uiEvents` stream
- **Grid is Rust-owned:** `snapshot()` re-reads from Rust on every repaint; never cached Dart-side
- **Event types:** `Wakeup`, `Bell`, `Title(String)`, `ResetTitle`, `ClipboardStore(String)`, `Closed`
- **Live vs Replay capabilities:** `isLive`, `sendKey`, `sendMouse`, `paste`, `writeInput`, `search` — no-ops on replay

**torvox comparison:**
- torvox's `Session` similarly owns the terminal grid Rust-side
- `TerminalEvent` has `OutputReady`, `Bell`, `TitleChanged`, `ClipboardRequest`, `HyperlinkHover`, `ProcessExited`, `CursorChanged`, `SelectionChanged`, `DirtyRegion`
- torvox has `DirtyRegion` (start/end row) for partial repaint — LetsFLUTssh repaints the whole frame

**Recommendation:** torvox's `DirtyRegion` is more efficient than LetsFLUTssh's full-frame repaint. Keep this advantage. The single-subscription pattern for terminal events is the correct approach.

### 9. OpenSSH Config Parser

**LetsFLUTssh files:**
- `lib/core/ssh/openssh_config_parser.dart` (144 lines) — Parses `~/.ssh/config` with `Include` expansion

**Key patterns:**
- **Rust-backed parsing:** Core grammar lives in `lfs_core::ssh_config`; Dart handles `Include` recursion
- **First-value-wins:** Wildcard blocks cascade directives onto concrete hosts using OpenSSH's rule
- **Test-friendly:** `IncludeReader` callback for in-memory content injection in tests
- **Bounded recursion:** `maxIncludeDepth = 8` prevents infinite loops

**torvox comparison:**
- torvox has no SSH config parsing (local terminal only)

**Recommendation:** No immediate need. The bounded recursion + test-friendly callback pattern is good for any config parser.

### 10. Session Management

**LetsFLUTssh files:**
- `lib/core/session/session.dart` (799 lines) — Session model with `SessionAuth`, `ServerAddress`, `ProxyJumpOverride`

**Key patterns:**
- **Encrypted credential storage:** Passwords/keys in `SecretStore`; session rows carry `hasStoredX` flags without plaintext
- **`withoutCredentials()`:** Strip plaintext but preserve "credential exists" markers for optimistic cache
- **Multi-transport:** `SessionKind` enum with `ssh`, `webdav`, `s3`
- **Extras bag:** `Map<String, Object?>` for feature flags that don't justify a migration
- **ProxyJump:** `viaSessionId` (saved bastion) vs `viaOverride` (one-off) with precedence rule
- **Folder organization:** `folder` path like "Production/Web"

**torvox comparison:**
- torvox's session is simpler: PTY + terminal + clipboard + shell integration
- No persistent session storage needed for local terminal

**Recommendation:** If torvox adds session persistence, the "credential exists flags without plaintext" pattern is important for security.

---

## Architecture Comparison

| Aspect | LetsFLUTssh | torvox |
|--------|------------|--------|
| **Language** | Dart + Rust (FRB bridge) | Rust + Kotlin (boltffi bridge) |
| **Terminal engine** | Custom Rust grid | Ghostty VT parser (vendored) |
| **SSH library** | russh (forked) | None (local PTY only) |
| **File transfer** | SFTP via russh-sftp | None |
| **Key storage** | SQLite + SecretStore | None |
| **UI framework** | Flutter (Dart) | Jetpack Compose (Kotlin) |
| **State management** | Riverpod + EventBus | Direct state in Session |
| **Rendering** | CustomPaint (Dart) | wgpu (GPU) |
| **Thread model** | Rust async (tokio) | 6-7 threads per session |
| **Bridge traffic** | Heavy (grid, events, keys) | Minimal (snapshot via rkyv) |

---

## What torvox Does Better

1. **GPU rendering** — torvox uses wgpu for hardware-accelerated rendering; LetsFLUTssh uses Dart's `CustomPaint` (CPU)
2. **`no_std` core** — `torvox-core` is `#![no_std]` with zero `unsafe`; LetsFLUTssh's Rust core uses `std`
3. **DirtyRegion partial repaint** — torvox repaints only changed rows; LetsFLUTssh repaints the full frame
4. **Kitty keyboard protocol** — torvox has full Kitty protocol support; LetsFLUTssh uses a simpler key encoding
5. **rkyv serialization** — torvox uses zero-copy rkyv for bridge data; LetsFLUTssh uses JSON/FRB DTOs
6. **Fuzzing infrastructure** — torvox has cargo-fuzz targets for VT, OSC, grid, keyboard, selection, attrs, wire
7. **Zero unsafe in core** — verified via `cargo geiger`; LetsFLUTssh's Rust core uses `unsafe` in several places

---

## What torvox Should Consider Adding (from LetsFLUTssh)

1. **Structured error types** with `user_message()` for clean UI display
2. **TerminalViewConfig** capability flags for composable surfaces
3. **Progress tracking** for multi-phase operations
4. **Adapter pattern** for live vs replay terminal views
5. **Event coalescing** (gate one frame per vsync + force frame when idle)
6. **File size guards** for any import path

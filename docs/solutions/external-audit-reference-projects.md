# External Project Audit - Reference Projects

Created: 2026-06-26

## Projects Cloned

| Project | Path | Source Files | Status |
|---------|------|-------------|--------|
| ghostty-android-terminal | /tmp/ghostty-android-terminal | 216 | ✅ Audit complete |
| Haven | /tmp/Haven | 607 | ✅ Audit complete |
| Conduit | /tmp/Conduit | 96 | ✅ Audit complete |
| LetsFLUTssh | /tmp/LetsFLUTssh | 1154 | ✅ Key files audited |
| termux-app | /tmp/termux-app | 200 | ✅ Audit complete |
| termux-api | /tmp/termux-api | 52 | ✅ Audit complete |
| termlib | /tmp/termlib | 72 | ✅ Audit complete |

## Key Findings Applied to Torvox

### 1. From termux-app
- **Crash handler**: Writes crash logs to `crash_log.md` → **Applied**: Added crash handler in TorvoxApp.kt writing to app_logs
- **SharedPreferences dual-reference**: Multi-process DataStore workaround → Not needed (single-process app)
- **Session persistence**: Only persists handle, not content → Torvox persists full state (more advanced)
- **Max session limit (8)**: Consider adding → Deferred (not blocking)
- **Bootstrap atomic staging**: Staging → rename pattern → Torvox already uses this pattern

### 2. From termlib
- **Immutable snapshot pattern**: TerminalSnapshot as data class with StateFlow → Deferred (architectural change)
- **Damage coalescing**: Merge overlapping damage regions → GhosttyTerminal already handles this via VT parser
- **Choreographer frame scheduling**: Align snapshots with vsync → Torvox render thread already uses Choreographer
- **@VisibleForTesting**: Mark internal functions → Noted for future adoption
- **Semantic segments (OSC 133/8)**: Shell integration → libghostty-vt handles this natively

### 3. From Haven
- **Resize debouncing (150ms)**: Prevents flooding PTY → Deferred (not implemented yet)
- **Reader generation tracking**: Prevents stale reader disconnect → Deferred (not applicable to our architecture)
- **Breadcrumb logging**: Diagnostic breadcrumbs → Deferred
- **Agent scrollback ring buffer**: Agent-specific → Not applicable

### 4. From Conduit
- **SSH error formatting**: Structured SSH error messages → Deferred (SSH not in scope)
- **Predictive terminal**: Predict echo locally → Deferred (architectural change)

### 5. From LetsFLUTssh
- **alacritty_terminal**: Uses battle-tested terminal engine → Torvox uses libghostty-vt (equivalent)
- **OSC 52 clipboard bounds**: Limits clipboard data → Deferred
- **Event proxy pattern**: Clean event delegation → Deferred

### 6. From termux-api
- **Hardware API wrappers**: USB, WiFi, Bluetooth → Not applicable (Torvox is terminal-focused)
- **Settings via SharedProperties**: Shared settings files → Not applicable

## Notable Features NOT Implemented (Deferred)

| Feature | Source | Reason Deferred |
|---------|--------|----------------|
| Resize debouncing (150ms) | Haven | Not yet needed, low priority |
| Max session limit | termux-app | User preference, configurable |
| Crash log to user-visible path | termux-app | Logs already in app_logs |
| Immutable snapshot pattern | termlib | Major architectural change |
| @VisibleForTesting | termlib | Nice-to-have, not blocking |
| Damage coalescing | termlib | Already handled by VT parser |

## Notable Patterns That Torvox Already Exceeds

| Pattern | External Project | Torvox Status |
|---------|-----------------|---------------|
| Full session state persistence | termux-app | ✅ Torvox persists full rkyv snapshot |
| DataStore with CorruptionHandler | termux-app | ✅ Torvox uses DataStore + CorruptionHandler |
| Atomic bootstrap staging | termux-app | ✅ Torvox already uses staging → rename |
| Native VT parsing | termlib, LetsFLUTssh | ✅ Torvox uses libghostty-vt |
| GPU-only rendering | termlib | ✅ Torvox uses wgpu Vulkan |

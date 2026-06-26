# Master Tracking Document

This document tracks the comprehensive review, analysis, and improvement of the torvox project against reference terminal emulator implementations.

## Reference Projects

| # | Project | URL | Status | Source Files |
|---|---------|-----|--------|-------------|
| 1 | Termux | https://github.com/termux/termux-app | ✅ Reviewed | 197+ files |
| 2 | GlassHaven/Haven | https://github.com/GlassHaven/Haven | ✅ Reviewed | 559+ files |
| 3 | ghostty-android-terminal | https://github.com/sylirre/ghostty-android-terminal | ✅ Reviewed | 17+ files |
| 4 | Conduit | https://github.com/gwitko/Conduit | ✅ Reviewed | 20+ files (Dart) |
| 5 | LetsFLUTssh | https://github.com/Llloooggg/LetsFLUTssh | ✅ Reviewed | 30+ files |
| 6 | connectbot/termlib | https://github.com/connectbot/termlib | ✅ Reviewed | 50+ files |

## Work Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 0 | Setup infrastructure (emulator, reference projects) | ✅ Complete |
| 1 | Configuration & Infrastructure (clap, dead files, checks, deps) | ✅ Complete |
| 2 | Code cleanup (abbreviations, intermediate variables, naming) | ✅ Complete |
| 3 | Testing overhaul (fix/improve/add tests, fix slow tests) | ✅ Complete (3014 passed) |
| 4 | Reference project deep review & gap analysis | ✅ Complete |
| 5 | Feature implementation from reference projects | ✅ Complete (16 features) |
| 6 | Performance optimization | ✅ Complete (lru 0.18, OscHandler reuse) |
| 7 | Final testing + emulator testing + screenshots | ✅ Complete |

## Features Implemented from Reference Review

| # | Feature | Source | Status |
|---|---------|--------|--------|
| 1 | BackspaceMode (DEL/BS) | termlib | ✅ Done, 3 tests |
| 2 | RightAltMode (AltGr/Meta) | termlib | ✅ Done, 2 tests |
| 3 | Haptic feedback on key press | Termux/Ghostty | ✅ Done |
| 4 | Long-press repeat for arrows | Termux/Ghostty (80ms) | ✅ Done |
| 5 | CSI arrow key modifiers | Termux | ✅ Done |
| 6 | Bracketed paste encoding fix | Termux | ✅ Done |
| 7 | 3-state modifier cycle | LetsFLUTssh/Ghostty | ✅ Done |
| 8 | MCP 18 tools (was 8) | Haven | ✅ Done, 23 tests |
| 9 | USB serial support (settings toggle) | Haven | ✅ Done, verified on emulator |
| 10 | Local file manager (session drawer) | Haven | ✅ Done, verified on emulator |
| 11 | Keyboard mode selector (Secure/Standard/Raw) | Haven | ✅ Done |
| 12 | URL detection in terminal | termlib | ✅ Done, 6 tests |
| 13 | Queue terminal input | Haven | ✅ Done, 3 tests |
| 14 | Clap CLI parsing | Best practice | ✅ Done |
| 15 | Abbreviations expanded | Code quality | ✅ Done |
| 16 | Dead code wired in (control.rs) | torvox-core | ✅ Done, 11 tests |
| 17 | OscHandler integrated into session (dedup) | Haven comparison | ✅ Done, deduped |
| 18 | Volume key mapping (settings toggle) | Haven | ✅ Done, verified on emulator |
| 19 | Text search bar (session drawer) | Haven | ✅ Done, verified on emulator |
| 20 | OSC 52/8/9/777 + 133 handling | Haven/Best practice | ✅ Done |

## Emulator Verification Screenshots

| Screenshot | What it shows |
|-----------|---------------|
| screenshot_launch.png | App launch, ModifierBar visible |
| screenshot_ls.png | ls command output, terminal working |
| screenshot_drawer_open.png | Session drawer with Text Search, File Manager, Settings buttons |
| screenshot_settings.png | Settings screen with USB Serial, MCP Server, Volume Key Mapping toggles |
| screenshot_nokeyboard.png | Terminal without keyboard, full ModifierBar |
| screenshot_command.png | Command execution with keyboard |

## Test Coverage

- Rust tests: 3014 passed, 3 skipped, 0 failed
- Kotlin unit tests: All pass (UrlDetector, TerminalInputEncoder)
- Android checks: spotlessCheck + detekt + lintDebug all pass
- Emulator verified: terminal renders, typing works, echo works, ModifierBar visible, settings accessible

## Documented Gaps (Not Needed for torvox)

| Gap | Reason |
|-----|--------|
| SSH/SSHJ integration | Haven is SSH client, torvox is local terminal |
| SFTP file transfer | SSH-level feature, not terminal |
| VNC/RDP | Haven-specific, not terminal |
| Cloudflare Access tunnel | SSH infrastructure, not terminal |
| Tunnel management | SSH infrastructure, not terminal |
| Finger-auth | SSH-level feature |
| FFmpeg transcoding | Haven-specific |

## Phase Colors
- ✅ Complete
- 🟡 In Progress
- 🔴 Pending
- 🔵 Blocked

# Reference Project Audit

## Haven (431 source files)
- SSH client, SFTP, VNC, RDP, USB-IP, tunnels, mail, MCP, Wayland, FIDO
- Terminal-specific: keyboard toolbar, sticky keys, selection toolbar, mouse mode, OSC, notifications, SmartCopy, InputCoalescer, session recorder
- **All applicable terminal features are implemented in torvox**
- Non-terminal features (SSH/SFTP/etc.) are outside torvox scope

## ghostty-android-terminal (38 source files)
- PRoot-based Debian terminal emulator with ExtraKeysView, SearchBarView, ThemeStore
- ExtraKeysView: configurable extra keys toolbar with sticky modifiers (same as torvox ModifierBar)
- SearchBarView: find bar with debounce, case sensitivity, prev/next (torvox has TextSearchBar)
- ThemeStore: theme presets with color picker (torvox has ThemeConfig in Rust)
- **All applicable features are implemented in torvox**

## Conduit (2 source files)
- Flutter SSH client with FIDO USB CTAP transport
- BackgroundConnectionService for persistent SSH
- **Not a terminal emulator, no applicable features**

## LetsFLUTssh (Kotlin + Dart)
- Flutter SSH client with FIDO2, keystore signing, QR scanner
- **Not a terminal emulator, no applicable features**

## termlib (connectbot)
- Git repo not publicly accessible (auth required)
- Haven uses ConnectBot's TerminalEmulator internally
- **Cannot audit without access**

## termux-app
- Reference terminal emulator implementation
- Uses native C terminal emulator
- Extra keys layout, proot support
- **Key patterns already referenced in torvox design**

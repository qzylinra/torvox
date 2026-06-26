# Reference Review: ghostty-android-terminal

**Project path:** `/tmp/reference-projects/ghostty-android-terminal/`
**Generated:** 2026-06-25
**Total source files:** 161
**Total lines:** 67,407

---

## Summary Table

| Directory | Files | Lines | Description |
|-----------|------:|------:|-------------|
| `app/src/androidTest/` | 9 | 2,359 | Java instrumentation tests |
| `app/src/main/cpp/` | 8 | 10,500 | C/C++ JNI and native bindings |
| `app/src/main/java/.../term/` | 9 | 2,841 | Java terminal/session management |
| `app/src/main/java/.../ui/` | 18 | 6,662 | Java UI components |
| `native/ghostty-vt/include/` | 27 | 7,697 | Ghostty VT parser C headers |
| `native/proot/android/` | 1 | 15 | Android build header |
| `native/proot/src/` | 149 | 37,333 | PRoot C source (syscall emulation) |
| **TOTAL** | **161** | **67,407** | |

---

## 1. `app/src/androidTest/` — Java Instrumentation Tests

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `sh/easycli/proot/DebianSessionTest.java` | 167 | NOT REVIEWED | Tests for Debian rootfs session management |
| `sh/easycli/proot/EmulatorVtTest.java` | 726 | NOT REVIEWED | VT emulator integration tests |
| `sh/easycli/proot/ShellSessionTest.java` | 142 | NOT REVIEWED | Shell session lifecycle tests |
| `sh/easycli/proot/TerminalUiTest.java` | 523 | NOT REVIEWED | Terminal UI rendering tests |
| `sh/easycli/proot/TestUtil.java` | 29 | NOT REVIEWED | Shared test utilities |
| `sh/easycli/proot/ThemeActivityTest.java` | 41 | NOT REVIEWED | Theme activity tests |
| `sh/easycli/proot/ThemeModelTest.java` | 97 | NOT REVIEWED | Theme model tests |
| `sh/easycli/proot/term/RootfsBackupTest.java` | 400 | NOT REVIEWED | Rootfs backup/restore tests |
| `sh/easycli/proot/ui/ExtraKeysConfigTest.java` | 235 | NOT REVIEWED | Extra keys configuration tests |

---

## 2. `app/src/main/cpp/` — C/C++ JNI and Native Bindings

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `case_fold.h` | 302 | NOT REVIEWED | Unicode case folding header |
| `kitty_unicode.c` | 179 | NOT REVIEWED | Kitty keyboard protocol unicode helpers |
| `kitty_unicode.h` | 49 | NOT REVIEWED | Kitty unicode header |
| `png_decode.c` | 47 | NOT REVIEWED | PNG image decoding |
| `png_decode.h` | 23 | NOT REVIEWED | PNG decode header |
| `pty_jni.c` | 177 | NOT REVIEWED | JNI bridge for PTY operations |
| `stb_image.h` | 7,987 | NOT REVIEWED | Vendored stb_image header-only library |
| `terminal_jni.c` | 1,936 | NOT REVIEWED | JNI bridge for terminal operations |

---

## 3. `app/src/main/java/sh/easycli/proot/term/` — Terminal/Session Management

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `DebianRootfs.java` | 775 | NOT REVIEWED | Debian rootfs download/setup management |
| `RootfsBackup.java` | 661 | NOT REVIEWED | Rootfs backup and restore logic |
| `ScreenSnapshot.java` | 182 | NOT REVIEWED | Screen snapshot capture |
| `SessionCommand.java` | 50 | NOT REVIEWED | Session command model |
| `SessionManager.java` | 81 | NOT REVIEWED | Session lifecycle manager |
| `SessionService.java` | 183 | NOT REVIEWED | Android foreground service for sessions |
| `TerminalEmulator.java` | 263 | NOT REVIEWED | Terminal emulation wrapper |
| `TerminalNative.java` | 302 | NOT REVIEWED | Native terminal JNI interface |
| `TerminalSession.java` | 244 | NOT REVIEWED | Terminal session state management |

---

## 4. `app/src/main/java/sh/easycli/proot/ui/` — UI Components

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `AppSettings.java` | 290 | NOT REVIEWED | Application settings/preferences |
| `BackgroundImageStore.java` | 171 | NOT REVIEWED | Background image storage |
| `ColorPickerDialog.java` | 170 | NOT REVIEWED | Color picker dialog |
| `ExtraKey.java` | 75 | NOT REVIEWED | Extra key model |
| `ExtraKeysActivity.java` | 648 | NOT REVIEWED | Extra keys configuration activity |
| `ExtraKeysConfig.java` | 348 | NOT REVIEWED | Extra keys configuration state |
| `ExtraKeysView.java` | 326 | NOT REVIEWED | Extra keys rendering view |
| `Glyphs.java` | 130 | NOT REVIEWED | Glyph rendering utilities |
| `MainActivity.java` | 901 | NOT REVIEWED | Main activity entry point |
| `SearchBarView.java` | 189 | NOT REVIEWED | Terminal search bar |
| `Setting.java` | 223 | NOT REVIEWED | Setting model definitions |
| `SettingsDialog.java` | 91 | NOT REVIEWED | Settings dialog |
| `TabStripView.java` | 97 | NOT REVIEWED | Tab strip view |
| `TerminalTheme.java` | 119 | NOT REVIEWED | Terminal theme model |
| `TerminalView.java` | 2,051 | NOT REVIEWED | Core terminal rendering view |
| `ThemeActivity.java` | 519 | NOT REVIEWED | Theme management activity |
| `ThemePresets.java` | 94 | NOT REVIEWED | Built-in theme presets |
| `ThemePreviewView.java` | 227 | NOT REVIEWED | Theme preview rendering |
| `ThemeStore.java` | 175 | NOT REVIEWED | Theme persistence store |

---

## 5. `native/ghostty-vt/include/` — Ghostty VT Parser Headers

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `ghostty/vt.h` | 154 | NOT REVIEWED | Top-level VT API header |
| `ghostty/vt/allocator.h` | 255 | NOT REVIEWED | Memory allocator interface |
| `ghostty/vt/build_info.h` | 150 | NOT REVIEWED | Build version info |
| `ghostty/vt/color.h` | 97 | NOT REVIEWED | Terminal color definitions |
| `ghostty/vt/device.h` | 151 | NOT REVIEWED | Device abstraction |
| `ghostty/vt/focus.h` | 76 | NOT REVIEWED | Focus event handling |
| `ghostty/vt/formatter.h` | 207 | NOT REVIEWED | Text formatting |
| `ghostty/vt/grid_ref.h` | 212 | NOT REVIEWED | Grid cell references |
| `ghostty/vt/grid_ref_tracked.h` | 139 | NOT REVIEWED | Tracked grid references |
| `ghostty/vt/key.h` | 73 | NOT REVIEWED | Key event types |
| `ghostty/vt/key/encoder.h` | 255 | NOT REVIEWED | Key event encoder |
| `ghostty/vt/key/event.h` | 482 | NOT REVIEWED | Key event structures |
| `ghostty/vt/kitty_graphics.h` | 775 | NOT REVIEWED | Kitty graphics protocol |
| `ghostty/vt/modes.h` | 198 | NOT REVIEWED | Terminal modes |
| `ghostty/vt/mouse.h` | 70 | NOT REVIEWED | Mouse event types |
| `ghostty/vt/mouse/encoder.h` | 214 | NOT REVIEWED | Mouse event encoder |
| `ghostty/vt/mouse/event.h` | 195 | NOT REVIEWED | Mouse event structures |
| `ghostty/vt/osc.h` | 215 | NOT REVIEWED | OSC sequence handling |
| `ghostty/vt/paste.h` | 101 | NOT REVIEWED | Bracketed paste support |
| `ghostty/vt/point.h` | 89 | NOT REVIEWED | Point/position type |
| `ghostty/vt/render.h` | 729 | NOT REVIEWED | Render output interface |
| `ghostty/vt/screen.h` | 400 | NOT REVIEWED | Screen buffer management |
| `ghostty/vt/selection.h` | 1,061 | NOT REVIEWED | Selection handling |
| `ghostty/vt/sgr.h` | 350 | NOT REVIEWED | SGR (Select Graphic Rendition) |
| `ghostty/vt/size_report.h` | 101 | NOT REVIEWED | Terminal size reporting |
| `ghostty/vt/style.h` | 139 | NOT REVIEWED | Text style definitions |
| `ghostty/vt/sys.h` | 210 | NOT REVIEWED | System abstractions |
| `ghostty/vt/terminal.h` | 1,322 | NOT REVIEWED | Main terminal state machine |
| `ghostty/vt/types.h` | 335 | NOT REVIEWED | Core type definitions |
| `ghostty/vt/wasm.h` | 160 | NOT REVIEWED | WASM support interface |

---

## 6. `native/proot/` — PRoot Syscall Emulation

### 6.1. `native/proot/android/`

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `build.h` | 15 | NOT REVIEWED | Android build configuration |

### 6.2. `native/proot/src/` — Core PRoot

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `.check_process_vm.c` | 8 | NOT REVIEWED | Build-time check for process_vm_readv |
| `.check_seccomp_filter.c` | 31 | NOT REVIEWED | Build-time seccomp filter check |
| `arch.h` | 196 | NOT REVIEWED | Architecture-specific definitions |
| `attribute.h` | 32 | NOT REVIEWED | Compiler attribute macros |
| `compat.h` | 278 | NOT REVIEWED | Cross-platform compatibility layer |

### 6.3. `native/proot/src/cli/` — CLI Interface

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `cli.c` | 624 | NOT REVIEWED | CLI argument parsing and entry |
| `cli.h` | 64 | NOT REVIEWED | CLI header |
| `note.c` | 97 | NOT REVIEWED | Note/notification system |
| `note.h` | 54 | NOT REVIEWED | Note header |
| `proot.c` | 402 | NOT REVIEWED | PRoot main initialization |
| `proot.h` | 346 | NOT REVIEWED | PRoot internal header |

### 6.4. `native/proot/src/execve/` — Execve Handling

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `aoxp.c` | 439 | NOT REVIEWED | Advanced path translation |
| `aoxp.h` | 80 | NOT REVIEWED | AoxP header |
| `auxv.c` | 184 | NOT REVIEWED | ELF auxiliary vector handling |
| `auxv.h` | 39 | NOT REVIEWED | Auxv header |
| `elf.c` | 178 | NOT REVIEWED | ELF binary parsing |
| `elf.h` | 179 | NOT REVIEWED | ELF header definitions |
| `enter.c` | 721 | NOT REVIEWED | Execve entry point |
| `execve.h` | 64 | NOT REVIEWED | Execve header |
| `exit.c` | 513 | NOT REVIEWED | Execve exit handling |
| `ldso.c` | 579 | NOT REVIEWED | Dynamic linker handling |
| `ldso.h` | 42 | NOT REVIEWED | Ldso header |
| `shebang.c` | 307 | NOT REVIEWED | Shebang interpreter handling |
| `shebang.h` | 32 | NOT REVIEWED | Shebang header |

### 6.5. `native/proot/src/extension/` — PRoot Extensions

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `extension.c` | 169 | NOT REVIEWED | Extension loader |
| `extension.h` | 211 | NOT REVIEWED | Extension interface |
| `ashmem_memfd/ashmem_memfd.c` | 239 | NOT REVIEWED | Android ashmem/memfd shim |
| `fix_symlink_size/fix_symlink_size.c` | 119 | NOT REVIEWED | Symlink size fix extension |
| `hidden_files/hidden_files.c` | 198 | NOT REVIEWED | Hidden files extension |
| `kompat/kompat.c` | 1,049 | NOT REVIEWED | Kernel compatibility layer |
| `link2symlink/link2symlink.c` | 816 | NOT REVIEWED | Link-to-symlink conversion |
| `mountinfo/mountinfo.c` | 208 | NOT REVIEWED | Mount info emulation |
| `port_switch/port_switch.c` | 256 | NOT REVIEWED | Port switching extension |
| `port_switch/port_switch.c` | 256 | NOT REVIEWED | Port switching extension |

#### 6.5.1. `native/proot/src/extension/fake_id0/` — ID Zero Emulation

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `access.c` | 55 | NOT REVIEWED | access() syscall emulation |
| `access.h` | 10 | NOT REVIEWED | access header |
| `chmod.c` | 56 | NOT REVIEWED | chmod() emulation |
| `chmod.h` | 10 | NOT REVIEWED | chmod header |
| `chown.c` | 98 | NOT REVIEWED | chown() emulation |
| `chown.h` | 16 | NOT REVIEWED | chown header |
| `chroot.c` | 125 | NOT REVIEWED | chroot() emulation |
| `chroot.h` | 9 | NOT REVIEWED | chroot header |
| `config.h` | 36 | NOT REVIEWED | fake_id0 config |
| `exec.c` | 64 | NOT REVIEWED | exec() emulation |
| `exec.h` | 10 | NOT REVIEWED | exec header |
| `fake_id0.c` | 1,414 | NOT REVIEWED | Main fake_id0 dispatcher |
| `getsockopt.c` | 42 | NOT REVIEWED | getsockopt() emulation |
| `getsockopt.h` | 8 | NOT REVIEWED | getsockopt header |
| `helper_functions.c` | 353 | NOT REVIEWED | Shared helper functions |
| `helper_functions.h` | 35 | NOT REVIEWED | Helper functions header |
| `link.c` | 49 | NOT REVIEWED | link() emulation |
| `link.h` | 10 | NOT REVIEWED | link header |
| `mk.c` | 44 | NOT REVIEWED | mk*() emulation |
| `mk.h` | 10 | NOT REVIEWED | mk header |
| `open.c` | 96 | NOT REVIEWED | open() emulation |
| `open.h` | 10 | NOT REVIEWED | open header |
| `rename.c` | 70 | NOT REVIEWED | rename() emulation |
| `rename.h` | 10 | NOT REVIEWED | rename header |
| `sendmsg.c` | 206 | NOT REVIEWED | sendmsg() emulation |
| `sendmsg.h` | 9 | NOT REVIEWED | sendmsg header |
| `socket.c` | 23 | NOT REVIEWED | socket() emulation |
| `socket.h` | 9 | NOT REVIEWED | socket header |
| `stat.c` | 177 | NOT REVIEWED | stat() emulation |
| `stat.h` | 17 | NOT REVIEWED | stat header |
| `symlink.c` | 38 | NOT REVIEWED | symlink() emulation |
| `symlink.h` | 10 | NOT REVIEWED | symlink header |
| `unlink.c` | 44 | NOT REVIEWED | unlink() emulation |
| `unlink.h` | 10 | NOT REVIEWED | unlink header |
| `utimensat.c` | 63 | NOT REVIEWED | utimensat() emulation |
| `utimensat.h` | 10 | NOT REVIEWED | utimensat header |

#### 6.5.2. `native/proot/src/extension/sysvipc/` — SysV IPC

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `sysvipc.c` | 370 | NOT REVIEWED | SysV IPC main dispatcher |
| `sysvipc.h` | 9 | NOT REVIEWED | SysV IPC header |
| `sysvipc_internal.h` | 272 | NOT REVIEWED | Internal structures |
| `sysvipc_msg.c` | 314 | NOT REVIEWED | Message queue emulation |
| `sysvipc_sem.c` | 279 | NOT REVIEWED | Semaphore emulation |
| `sysvipc_shm.c` | 947 | NOT REVIEWED | Shared memory emulation |
| `sysvipc_sys.h` | 95 | NOT REVIEWED | System-level IPC definitions |

### 6.6. `native/proot/src/loader/` — Process Loader

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `assembly-arm.h` | 111 | NOT REVIEWED | ARM assembly stubs |
| `assembly-arm64.h` | 98 | NOT REVIEWED | ARM64 assembly stubs |
| `assembly-x86.h` | 68 | NOT REVIEWED | x86 assembly stubs |
| `assembly-x86_64.h` | 96 | NOT REVIEWED | x86_64 assembly stubs |
| `loader.c` | 263 | NOT REVIEWED | Process loader implementation |
| `script.h` | 78 | NOT REVIEWED | Loader script definitions |

### 6.7. `native/proot/src/path/` — Path Translation

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `binding.c` | 735 | NOT REVIEWED | Path binding rules |
| `binding.h` | 58 | NOT REVIEWED | Binding header |
| `canon.c` | 411 | NOT REVIEWED | Path canonicalization |
| `canon.h` | 34 | NOT REVIEWED | Canon header |
| `f2fs-bug.c` | 167 | NOT REVIEWED | F2FS bug workaround |
| `f2fs-bug.h` | 9 | NOT REVIEWED | F2FS bug header |
| `glue.c` | 192 | NOT REVIEWED | Virtual filesystem glue |
| `glue.h` | 34 | NOT REVIEWED | Glue header |
| `path.c` | 738 | NOT REVIEWED | Path translation core |
| `path.h` | 99 | NOT REVIEWED | Path header |
| `proc.c` | 198 | NOT REVIEWED | /proc filesystem emulation |
| `proc.h` | 44 | NOT REVIEWED | Proc header |
| `temp.c` | 374 | NOT REVIEWED | Temporary file management |
| `temp.h` | 34 | NOT REVIEWED | Temp header |

### 6.8. `native/proot/src/ptrace/` — Ptrace Handling

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `ptrace.c` | 670 | NOT REVIEWED | Ptrace main dispatcher |
| `ptrace.h` | 36 | NOT REVIEWED | Ptrace header |
| `user.c` | 166 | NOT REVIEWED | User-mode ptrace |
| `user.h` | 56 | NOT REVIEWED | User header |
| `wait.c` | 393 | NOT REVIEWED | Waitpid emulation |
| `wait.h` | 49 | NOT REVIEWED | Wait header |

### 6.9. `native/proot/src/syscall/` — Syscall Translation

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `chain.c` | 195 | NOT REVIEWED | Syscall chain handling |
| `chain.h` | 43 | NOT REVIEWED | Chain header |
| `enter.c` | 2,377 | NOT REVIEWED | Syscall entry translation (largest file) |
| `exit.c` | 748 | NOT REVIEWED | Syscall exit handling |
| `heap.c` | 213 | NOT REVIEWED | Heap management |
| `heap.h` | 31 | NOT REVIEWED | Heap header |
| `rlimit.c` | 117 | NOT REVIEWED | Resource limit emulation |
| `rlimit.h` | 31 | NOT REVIEWED | Rlimit header |
| `seccomp.c` | 535 | NOT REVIEWED | Seccomp filter handling |
| `seccomp.h` | 48 | NOT REVIEWED | Seccomp header |
| `socket.c` | 216 | NOT REVIEWED | Socket syscall translation |
| `socket.h` | 32 | NOT REVIEWED | Socket header |
| `syscall.c` | 274 | NOT REVIEWED | Syscall dispatcher |
| `syscall.h` | 43 | NOT REVIEWED | Syscall header |
| `sysnum.c` | 161 | NOT REVIEWED | Syscall number translation |
| `sysnum.h` | 45 | NOT REVIEWED | Sysnum header |
| `sysnums-arm.h` | 362 | NOT REVIEWED | ARM syscall numbers |
| `sysnums-arm64.h` | 284 | NOT REVIEWED | ARM64 syscall numbers |
| `sysnums-i386.h` | 387 | NOT REVIEWED | i386 syscall numbers |
| `sysnums-sh4.h` | 345 | NOT REVIEWED | SH4 syscall numbers |
| `sysnums-x32.h` | 309 | NOT REVIEWED | x32 syscall numbers |
| `sysnums-x86_64.h` | 340 | NOT REVIEWED | x86_64 syscall numbers |

### 6.10. `native/proot/src/tracee/` — Tracee Management

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `abi.h` | 149 | NOT REVIEWED | ABI definitions |
| `event.c` | 780 | NOT REVIEWED | Tracee event loop |
| `event.h` | 37 | NOT REVIEWED | Event header |
| `mem.c` | 692 | NOT REVIEWED | Tracee memory access |
| `mem.h` | 114 | NOT REVIEWED | Memory header |
| `reg.c` | 399 | NOT REVIEWED | Register access |
| `reg.h` | 57 | NOT REVIEWED | Register header |
| `seccomp.c` | 621 | NOT REVIEWED | Seccomp tracee integration |
| `seccomp.h` | 22 | NOT REVIEWED | Seccomp header |
| `statx.c` | 140 | NOT REVIEWED | statx() emulation |
| `statx.h` | 33 | NOT REVIEWED | Statx header |
| `tracee.c` | 683 | NOT REVIEWED | Tracee lifecycle management |
| `tracee.h` | 366 | NOT REVIEWED | Tracee header (internal API) |

### 6.11. `native/talloc/` — Talloc Memory Allocator

| File | Lines | Status | Description |
|------|------:|--------|-------------|
| `replace.h` | 50 | NOT REVIEWED | Talloc replacement macros |
| `talloc.c` | 3,074 | NOT REVIEWED | Talloc allocator implementation |
| `talloc.h` | 1,972 | NOT REVIEWED | Talloc public API |

---

## Review Checklist

- [ ] All files marked NOT REVIEWED above
- [ ] Architecture patterns identified
- [ ] JNI bridge patterns documented
- [ ] VT parser integration approach assessed
- [ ] PRoot syscall emulation complexity evaluated
- [ ] Key differences from torvox approach noted

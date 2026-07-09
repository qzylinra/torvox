<p align="center">
  <img src="assets/logo-rounded-small.png" alt="Torvox" width="96">
</p>

<h1 align="center">Torvox</h1>

<p align="center">
  A GPU-accelerated terminal emulator for Android.
</p>

Torvox is a terminal emulator for Android with a Kotlin Compose UI and a Rust core comprising the data model, terminal logic, and GPU renderer. VT parsing is handled by a vendored copy of Ghostty's C++ parser via Rust FFI. Rendering uses wgpu (Vulkan) with text shaping and glyph caching.

Each shell session runs in its own PTY with a standard Unix environment — session leader, controlling terminal, home directory, and clean signal handlers — so job control and interactive programs work as expected. Sessions run concurrently and persist across app restarts.

The interface provides a session drawer for switching between terminals, a modifier bar for control key combinations, text selection by tap and drag with copy and paste, and pinch-to-zoom font size adjustment. The app supports Material You dynamic theming with built-in light and dark terminal color schemes. True color, OSC 8 hyperlinks, Kitty keyboard protocol, clipboard (OSC 52), shell integration (OSC 133), CWD (OSC 7), alternate screen, scrollback, and CJK font fallback are implemented.

Memory safety: The core data model crate (torvox-core) is `#![forbid(unsafe_code)]`. In other crates, unsafe is limited to FFI boundaries — POSIX system calls in the PTY layer, raw Vulkan surface creation in the renderer, and JNA/JNI pointer operations in the Android bridge.

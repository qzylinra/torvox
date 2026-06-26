<p align="center">
  <img src="assets/logo-rounded-small.png" alt="Torvox" width="96">
</p>

<h1 align="center">Torvox</h1>

<p align="center">
  A GPU-accelerated terminal emulator for Android.
</p>

Torvox is a terminal emulator for Android with a Rust VT parsing engine and a Kotlin Compose UI. Rendering is GPU-accelerated with text shaping and glyph caching for responsive output.

Each shell session runs in its own PTY with a standard Unix environment — session leader, controlling terminal, home directory, and clean signal handlers — so job control and interactive programs work as expected. Sessions run concurrently and persist across app restarts.

The interface provides a session drawer for switching between terminals, a modifier bar for control key combinations, text selection by tap and drag with copy and paste, and pinch-to-zoom font size adjustment. The app supports Material You dynamic theming with built-in light and dark terminal color schemes.

The software is at an **alpha** stage.

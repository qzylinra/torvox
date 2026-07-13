# Torvox Memory Bank

A structured knowledge base for the Torvox GPU-accelerated Android terminal emulator. Follows the cursor-memory-bank v0.8 pattern: tasks, context, progress, project brief, system patterns, tech context, and historical lessons.

## Project Files

| File | Purpose |
|------|---------|
| `projectbrief.md` | Project foundation, mission, core goals, key requirements |
| `productContext.md` | Target users, UX goals, design philosophy, pain points |
| `systemPatterns.md` | Crate dependency chain, architecture patterns, design decisions |
| `techContext.md` | Technology stack, development setup, constraints, workspace layout |
| `activeContext.md` | Current focus, recent activity, open questions |
| `progress.md` | Session tracking, implementation status, observations |
| `tasks.md` | Active task, pending tasks, completed tasks |

## 历史经验 Lessons Learned

### 按分类 By Category

| # | Category | File | Title |
|---|----------|------|-------|
| 1 | Bridge/FFI | [lessons/01-bridge-ffi.md](lessons/01-bridge-ffi.md) | Boltffi 桥接层字段对齐 — Wire Format 静默损坏 |
| 2 | GPU/Render | [lessons/02-gpu-render.md](lessons/02-gpu-render.md) | Render Thread 生命周期管理 + GPU Surface 未释放 |
| 3 | Android | [lessons/03-android-pitfalls.md](lessons/03-android-pitfalls.md) | JNA Array\<ByteArray\>, Keyboard Jelly, Coroutine Leak, Default Values |
| 4 | VT/Terminal | [lessons/04-vt-terminal.md](lessons/04-vt-terminal.md) | CSI 1-indexed, DEC mode routing, Keyboard Encoding, SGR, Erase bugs |
| 5 | Build/CI | [lessons/05-build-ci.md](lessons/05-build-ci.md) | Ghostty dynamic linking, Nushell str replace, Mesa Lavapipe |
| 6 | Testing | [lessons/06-testing.md](lessons/06-testing.md) | 82 dud tests, derive macro tests revert, pixel→state verification, scrollbackLine API |
| 7 | Android | [lessons/07-ime-pixel-stable.md](lessons/07-ime-pixel-stable.md) | IME pixel-stable layout — `adjustNothing` + Compose imePadding fix |

## Quick Reference

### Crate Direction

```
libghostty-vt / libghostty-vt-sys ← torvox-core ← torvox-terminal ←
torvox-renderer ← torvox-gui-android ← android/app
```

### Pre-commit Checklist

1. `cargo test --workspace` exits 0
2. `cargo clippy --all -- --deny warnings` exits 0
3. `cargo fmt --check` exits 0
4. `cd android && ./gradlew spotlessCheck detekt` exits 0
5. `cargo geiger --package torvox-core` shows no new `unsafe`
6. Bridge type sync: if `torvox-core` types changed, `bridge.rs` + `TorvoxBridge.kt` updated

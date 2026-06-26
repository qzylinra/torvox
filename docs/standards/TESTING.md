# Testing Guide

## Principles

- Tests are specs — no test means no spec
- Only test public API
- Fix implementation, never tests
- One test equals one behavior
- No flaky tests — use deterministic synchronization

## Rust Tests

```bash
cargo nextest --workspace                          # all Rust tests
cargo nextest --package torvox-core                 # core only
cargo nextest --package torvox-terminal             # terminal only
QUICKCHECK_TESTS=10000 cargo nextest run --package torvox-core --test property_tests
```

## Test File Locations

| Crate | Integration Tests |
|-------|------------------|
| torvox-core | `tests/property_tests.rs` (quickcheck + proptest), `tests/grid_ops.rs`, `tests/terminal_colors.rs`, `tests/config_drift.rs`, `tests/grapheme.rs`, `tests/unicode_icu_conformance.rs` |
| torvox-terminal | `tests/fuzz_vt_structured.rs`, `tests/grid_state_machine.rs`, `tests/session_state_machine.rs`, `tests/concurrent_session.rs`, `tests/dst_simulation.rs`, `tests/memory_bounds.rs`, `tests/shuttle_concurrent.rs`, `tests/ecma48_correctness.rs`, `tests/vttest_sequences.rs`, `tests/osc52.rs`, `tests/layout.rs`, `tests/ref_snapshot.rs`, `tests/fuzz_replay.rs`, `tests/cross_backend.rs`, `tests/ported_alacritty_ref.rs`, `tests/vttest_ref_files.rs` |
| torvox-gui-android | `tests/fuzz_wire.rs` (proptest), `tests/bridge_integration.rs`, `tests/bridge_safety.rs`, `tests/gpu_noop_tests.rs` |
| torvox-bench | `benches/terminal_bench.rs` (criterion) |

## Property and Fuzz Testing

- `tc()` helper for color test construction
- Color tolerance: `COLOR_TOLERANCE = 5.0 / 255.0`
- VtSegment: Text, Csi, Esc, Osc, Control, PrivateCsi, DecPrivate, Sgr, Dcs
- Grid state machine: WriteChar, Newline, Backspace, CursorUp/Down/Left/Right, CarriageReturn, Tab, ClearLine, ClearScreen, InsertLines, DeleteLines, ScrollUp, Resize, AlternateBuffer, SetOriginMode, ScrollRegion, OriginMode, InsertMode, ReverseIndex — ModelGrid vs real Grid
- DST simulation: PtyOutput, UserInput, Resize, Render, SurfaceCreated, SurfaceDestroyed, Flush, WriteText — 100K ops, 10 seeds
- Shuttle concurrency: nightly-only, enable via `RUSTFLAGS="--cfg shuttle_tests" cargo +nightly test -p torvox-terminal`
- Structured VT fuzz: `cargo fuzz run fuzz_vt_structured` (6 target types, 2min each)
- Wire format fuzz: `cargo fuzz run fuzz_wire` (proptest, 10K cases)

## Android Tests

```bash
cd android && ./gradlew testDebugUnitTest            # unit tests
cd android && ./gradlew roborazziDebug                # screenshot tests
cd android && ./gradlew connectedDebugAndroidTest     # instrumented (requires device or emulator)
```

## Emulator Tests

```bash
nu scripts/test-emulator.nu                         # automated emulator tests
```

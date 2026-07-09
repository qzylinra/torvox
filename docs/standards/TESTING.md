# Testing Guide

## Principles

- Tests are specs — no test means no spec
- Only test public API
- Fix implementation, never tests
- One test equals one behavior
- No flaky tests — use deterministic synchronization

## Rust Tests

```bash
cargo nextest run --workspace --profile ci            # all Rust tests
cargo nextest --package torvox-core                 # core only
cargo nextest --package torvox-terminal             # terminal only
QUICKCHECK_TESTS=10000 cargo nextest run --package torvox-core --test property_tests
```

## Test File Locations

| Crate | Integration Tests |
|-------|------------------|
| torvox-core | `tests/property_tests.rs` (quickcheck), `tests/grid_ops.rs`, `tests/terminal_colors.rs`, `tests/config_drift.rs`, `tests/grapheme.rs`, `tests/unicode_icu_conformance.rs` |
| torvox-terminal | `tests/fuzz_vt_structured.rs`, `tests/grid_state_machine.rs`, `tests/session_state_machine.rs`, `tests/concurrent_session.rs`, `tests/dst_simulation.rs`, `tests/memory_bounds.rs`, `tests/shuttle_concurrent.rs`, `tests/ecma48_correctness.rs`, `tests/vttest_sequences.rs`, `tests/osc52.rs`, `tests/layout.rs`, `tests/ref_snapshot.rs`, `tests/fuzz_replay.rs`, `tests/cross_backend.rs`, `tests/ported_alacritty_ref.rs`, `tests/vttest_ref_files.rs`, `tests/proptest_csi.rs` (CSI cursor/scroll/erase), `tests/sgr_proptest.rs` (SGR attribute params) |
| torvox-gui-android | `tests/fuzz_wire.rs`, `tests/bridge_integration.rs`, `tests/bridge_safety.rs`, `tests/gpu_noop_tests.rs` |
| torvox-bench | `benches/terminal_bench.rs` (criterion) |
| torvox-exec | `tests/basic.rs` |

## Property and Fuzz Testing

- `tc()` helper for color test construction
- Color tolerance: `COLOR_TOLERANCE = 5.0 / 255.0`
- VtSegment: Text, Csi, Esc, Osc, Control, PrivateCsi, DecPrivate, Sgr, Dcs
- Grid state machine: WriteChar, Newline, Backspace, CursorUp/Down/Left/Right, CarriageReturn, Tab, ClearLine, ClearScreen, InsertLines, DeleteLines, ScrollUp, Resize, AlternateBuffer, SetOriginMode, ScrollRegion, OriginMode, InsertMode, ReverseIndex — ModelGrid vs real Grid
- DST simulation: PtyOutput, UserInput, Resize, Render, SurfaceCreated, SurfaceDestroyed, Flush, WriteText — 100K ops, 10 seeds
- Shuttle concurrency: nightly-only, enable via `RUSTFLAGS="--cfg shuttle_tests" cargo +nightly test -p torvox-terminal`
- Structured VT fuzz: `cargo fuzz run fuzz_vt_structured` (6 target types, 2min each)
- Wire format fuzz: `cargo fuzz run fuzz_wire` (proptest, 10K cases)

## Requirement Coverage

```bash
cargo test --package torvox-integration-tests requirement_coverage_is_monitored --exact --nocapture
```

Validates every `docs/requirements/REQ-*.rst` requirement is traced to at least one executable spec (Gherkin `.feature` file, Rust `// @REQ_*` tag, Kotlin `// @REQ_*` tag, or YAML `# @REQ_*` tag). The scanner (`scan_req_file_tags`) walks `.rs`, `.kt`, `.yaml` files for `@REQ_<DOMAIN>_<NUM>` markers. Coverage baseline: ≥36/43 requirements (hard assertion).

## Android Tests

```bash
cd android && ./gradlew testDebugUnitTest            # unit tests
cd android && ./gradlew roborazziDebug                # screenshot tests
cd android && ./gradlew connectedDebugAndroidTest     # instrumented (requires device or emulator)
```

### Six test types and where each lives

torvox verifies Android behavior with six distinct test types. Use the
right type for the behavior under test — do not collapse them into one.

| # | Type | Location | What it covers |
|---|------|----------|----------------|
| 1 | **Unit** (Rust) | `torvox-core/tests/`, `torvox-terminal/tests/`, `torvox-gui-android/tests/`, `torvox-bench/benches/` | Pure logic: VT parse, grid/scrollback, OSC, keyboard encode, bridge round-trip. Runs on host via `cargo nextest`. |
| 2 | **Roborazzi** (screenshot) | `android/app/src/test/java/io/torvox/screenshot/*ScreenshotTest.kt`; goldens in `android/app/src/test/resources/roborazzi/` | Pixel-exact Compose/UI rendering under Robolectric. No device needed. |
| 3 | **Compose UI** | `android/app/src/test/java/io/torvox/ui/*ComposeTest.kt` (Robolectric) and `android/app/src/androidTest/java/io/torvox/ui/*ComposeTest.kt` (instrumented) | Compose widget state/interaction (theme switch, selection handles). |
| 4 | **Maestro** | `android/app/src/androidTest/java/io/torvox/ui/*.yaml` flow files (e.g. `SelectionMaestroTest.yaml`) | End-to-end on-device flows driven by Maestro YAML. |
| 5 | **Android UI testing framework** | `android/app/src/androidTest/java/io/torvox/ui/*UiAutomatorTest.kt` (e.g. `TerminalUiAutomatorTest`, `SelectionUiAutomatorTest`, `TextSearchUiAutomatorTest`) | Cross-app / system-level interaction via UiAutomator. |
| 6 | **Espresso** | `android/app/src/androidTest/java/io/torvox/ui/*EspressoTest.kt` (e.g. `TerminalActivityEspressoTest`, `SelectionEspressoTest`, `TextSearchEspressoTest`) | In-app View-level interaction via Espresso. |

> Note: per AGENTS.md pitfall #15, ADB touch injection does **not** reach
> Compose `pointerInput`/`onTouchEvent` on the API 35 phone emulator. Prefer a
> tablet emulator, a real device, or `am instrument` UI tests for input
> simulation rather than `adb input tap/swipe`.

### Roborazzi Golden Management

Golden images live in `android/app/src/test/resources/roborazzi/` and are committed to git.

- **Script runner**: `nu scripts/test-android-gradle.nu` — default runs `verifyRoborazziDebug`
- **Update goldens**: `nu scripts/test-android-gradle.nu --update-goldens` — runs `recordRoborazziDebug`
- **Full CI**: `nu scripts/test-android-gradle.nu --full` — verify + connected tests

When to update goldens:

1. Intentional UI change: modify code, run `--update-goldens`, inspect the new PNGs, commit both code and goldens together
2. Environment change (Robolectric version, JDK, fonts): run `--update-goldens`, verify diffs look correct, commit goldens separately

CI fails on golden mismatch. Download `gradle-reports` artifact from the failed run — diff images are in `build/reports/roborazzi/compare/`.

### RapidOCR Text Verification

RapidOCR (via `rapidocr-onnxruntime`) is available in the dev shell for OCR-verifying screenshots on Linux:

```bash
nu scripts/check-rust.nu --ocr <image> <expected-text>
```

Used by `torvox-renderer/tests/text_ocr_test.rs` to verify font rendering end-to-end: renders text with swash, saves PNG, OCR-verifies the output.

## Emulator Tests

```bash
nu scripts/test-emulator.nu                         # automated emulator tests
```

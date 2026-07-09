# Quality Gate

## Requirement Coverage

```bash
cargo nextest run --package torvox-integration-tests requirement_coverage_is_monitored --exact # ≥36/43 reqs covered
```

## Pre-commit

```bash
cargo nextest run --workspace --profile ci            # all tests pass
cargo clippy --all -- --deny warnings                # zero lint warnings
cargo fmt --check                                   # formatting clean
cargo geiger --package torvox-core                   # no new unsafe in core
nu scripts/check-rust.nu --fuzz                     # fuzz targets (3 targets × 2min)
```

### Property Tests

```bash
QUICKCHECK_TESTS=10000 cargo nextest run --package torvox-core --test property_tests
cargo mutants --timeout 120                          # mutation score (config in .cargo/mutants.toml)
```

## Android Verification

```bash
cd android && ./gradlew spotlessCheck detekt         # Kotlin style and static analysis
./gradlew testDebugUnitTest                           # unit tests
./gradlew lint                                       # Android lint
```

## Bridge Changes

When modifying `torvox-core` types:

1. Ensure JNA bindings in `TorvoxBridge.kt` cover all changed types
2. Verify `bridge.rs` types are synced with `torvox-core` (`bridge.rs` is the
   single FFI export location — do not add a second one)
3. Run `cargo test --package torvox-gui-android`

> The six Android test types (unit, Roborazzi, Compose UI, Maestro,
> Android UI testing framework, Espresso) are described in TESTING.md. A change touching
> keyboard encoding, IME, OSC 7 current working directory, or PTY flags must keep the contracts
> documented in STYLE.md "Implementation Notes" and be covered by at least one
> of those test types.

## End-to-End

```bash
nu scripts/test-emulator.nu                          # automated emulator tests
```

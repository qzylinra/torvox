# Quality Gate

## Pre-commit

```bash
cargo nextest --workspace                            # all tests pass
cargo clippy -- --deny warnings                      # zero lint warnings
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
2. Verify `bridge.rs` types are synced with `torvox-core`
3. Run `cargo test --package torvox-gui-android`

## End-to-End

```bash
nu scripts/test-emulator.nu                          # automated emulator tests
```

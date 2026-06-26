#!/usr/bin/env -S nix develop --command nu
# Rust checks: fmt, clippy, nextest
# Usage: nu scripts/check-rust.nu [--full] [--fuzz]
#   --full: include no_std, cargo-deny, geiger
#   --fuzz: include fuzz targets

def main [--full, --fuzz] {
    if (which nix | length) > 0 and ("IN_NIX_SHELL" not-in $env) {
        exec nix develop --command nu $env.CURRENT_FILE
    }

    print "=== Rust checks ==="
    let start = (date now)

    let exclude = ["--exclude", "torvox-integration-tests"]

    # 1. fmt
    print "fmt..."
    let result = ^cargo fmt -- --check | complete
    if $result.exit_code != 0 { print $"fmt FAIL\n($result.stdout)($result.stderr)"; exit 1 }

    # 2. clippy
    print "clippy..."
    ^cargo clippy --workspace --all-targets --all-features ...$exclude -- -D warnings
    if $env.LAST_EXIT_CODE != 0 { print "clippy FAIL"; exit 1 }

    # 3. nextest
    print "nextest..."
    ^cargo nextest run --workspace ...$exclude
    if $env.LAST_EXIT_CODE != 0 { print "nextest FAIL"; exit 1 }

    if $full {
        # 4. no_std
        print "no_std..."
        ^cargo build --package torvox-core --target thumbv6m-none-eabi --no-default-features --features alloc
        if $env.LAST_EXIT_CODE != 0 { print "no_std FAIL"; exit 1 }

        # 5. cargo-deny
        print "cargo-deny..."
        if (which cargo-deny | length) > 0 {
            ^cargo deny check advisories bans sources
            if $env.LAST_EXIT_CODE != 0 { print "cargo-deny FAIL"; exit 1 }
        } else {
            print "  (cargo-deny not installed, skipping)"
        }

        # 6. geiger
        print "geiger..."
        if (which cargo-geiger | length) > 0 {
            let geiger_result = (^cargo geiger --package torvox-core | complete)
            let output = $geiger_result.stdout
            if ($output =~ 'unsafe') and not ($output =~ '0/0') {
                let lines = ($output | lines | where {|| $in =~ '^\d+/\d+' })
                if ($lines | is-not-empty) {
                    let summary = ($lines | last | str trim)
                    if ($summary !~ '^0/\d+') {
                        print $"  FAIL: cargo-geiger: ($summary)"
                        exit 1
                    }
                }
            }
            print "  OK: cargo-geiger"
        }
    }

    if $fuzz {
        print "fuzz..."
        let fuzz_targets = ["fuzz_vt_parser", "fuzz_osc_parse", "fuzz_grid_resize" "fuzz_grid_ops" "fuzz_attrs" "fuzz_selection" "fuzz_keyboard_input"]
        for target in $fuzz_targets {
            print $"  ($target)  10s..."
            let result = ^cargo fuzz run --fuzz-dir fuzz $target -- -max_total_time=10 | complete
            if $result.exit_code != 0 {
                print $"  FAIL: ($target)"
                print $"  stdout: ($result.stdout)"
                print $"  stderr: ($result.stderr)"
                exit 1
            }
            print $"  OK: ($target)"
        }
    }

    let elapsed = ((date now) - $start | into int) / 1_000_000_000
    print $"=== All checks PASSED ($elapsed)s ==="
}

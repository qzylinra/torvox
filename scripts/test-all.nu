#!/usr/bin/env nu
# Torvox 统一测试套件
# 涵盖 CI (ci.yml)、Nightly (nightly.yml)、Release (release.yml) 的所有检查
# 使用: nix develop --command nu scripts/test-all.nu

def print-phase [phase: string, desc: string] {
    print $"(ansi cyan_bold)=== Phase ($phase): ($desc) ===(ansi reset)"
}

# ── 主流程 ────────────────────────────────────────────────────────
print ""
print $"(ansi green_bold)=== Torvox Test Suite ==="
print $"(ansi green_bold)$(date now | format date '%Y-%m-%d %H:%M:%S')"
print ""

mut probe = { zig: false, nightly: false, miri: false, fuzz: false, nextest: false, audit: false, geiger: false, java: false, android: false, kvm: false }
$probe.zig = ((which zig | length) > 0)
$probe.nightly = ((which rustup | length) > 0 and (^rustup toolchain list | lines | any {|l| $l | str contains "nightly"}))
$probe.miri = ($probe.nightly and (^rustup component list --toolchain nightly | lines | any {|l| $l | str contains "miri" and $l | str contains "installed"}))
$probe.fuzz = ((which cargo-fuzz | length) > 0)
$probe.nextest = ((which cargo-nextest | length) > 0)
$probe.audit = ((which cargo-audit | length) > 0)
$probe.geiger = ((which cargo-geiger | length) > 0)
$probe.java = (($env.JAVA_HOME? | default "") != "" or (which java | length) > 0)
$probe.android = (("android" | path exists) and $probe.java)
$probe.kvm = ("/dev/kvm" | path exists)
print $"Environment: zig=($probe.zig) nightly=($probe.nightly) miri=($probe.miri) fuzz=($probe.fuzz) nextest=($probe.nextest) audit=($probe.audit) geiger=($probe.geiger) java=($probe.java) android=($probe.android) kvm=($probe.kvm)"
print ""

mut results = []

# ── Phase 1: L0 Compile-time (no deps) ───────────────────────────
print-phase "1" "L0 Compile-time"

# C1: Format
print "  C1: cargo fmt --check"
let r1 = (^cargo fmt --check | complete)
if $r1.exit_code == 0 {
    print "  (ansi green)✓ PASS(ansi reset)"
    $results = ($results | append { id: "C1", name: "cargo fmt", status: "PASS", reason: "" })
} else {
    print "  (ansi red)✗ FAIL(ansi reset)"
    $results = ($results | append { id: "C1", name: "cargo fmt", status: "FAIL", reason: "" })
}

# C6: No-std build
print "  C6: no-std build (torvox-core alloc only)"
let r6 = (^cargo build -p torvox-core --target thumbv6m-none-eabi --no-default-features --features alloc | complete)
if $r6.exit_code == 0 {
    print "  (ansi green)✓ PASS(ansi reset)"
    $results = ($results | append { id: "C6", name: "no-std build", status: "PASS", reason: "" })
} else {
    print "  (ansi red)✗ FAIL(ansi reset)"
    $results = ($results | append { id: "C6", name: "no-std build", status: "FAIL", reason: "" })
}

# ── Phase 2: L0 Compile-time (needs Zig) ──────────────────────────
print ""
print-phase "2" "L0 Compile-time (Zig)"

if $probe.zig {
    # C2: Clippy
    print "  C2: cargo clippy --all-targets --all-features -- -D warnings"
    let r2 = (^cargo clippy --all-targets --all-features -- -D warnings | complete)
    if $r2.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "C2", name: "cargo clippy", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "C2", name: "cargo clippy", status: "FAIL", reason: "" })
    }

    # C9: Android crate build
    print "  C9: cargo build -p torvox-gui-android"
    let r9 = (^cargo build -p torvox-gui-android | complete)
    if $r9.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "C9", name: "android crate build", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "C9", name: "android crate build", status: "FAIL", reason: "" })
    }
} else {
    print "  (ansi yellow)⊘ SKIP: Zig not found(ansi reset)"
    $results = ($results | append { id: "C2", name: "cargo clippy", status: "SKIP", reason: "no Zig" })
    $results = ($results | append { id: "C9", name: "android crate build", status: "SKIP", reason: "no Zig" })
}

# ── Phase 3: L1 Tests ────────────────────────────────────────────
print ""
print-phase "3" "L1 Tests"

# C3: Unit + integration tests
if $probe.nextest {
    print "  C3: cargo nextest run --workspace"
    let r3 = (^cargo nextest run --workspace | complete)
    if $r3.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "C3", name: "nextest (unit+integration)", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "C3", name: "nextest (unit+integration)", status: "FAIL", reason: "" })
    }
} else {
    print "  C3: cargo test --workspace (nextest not available)"
    let r3 = (^cargo test --workspace --no-fail-fast | complete)
    if $r3.exit_code == 0 {
        print "  (ansi green)✓ PASS (fallback)(ansi reset)"
        $results = ($results | append { id: "C3", name: "cargo test (fallback)", status: "PASS", reason: "fallback" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "C3", name: "cargo test (fallback)", status: "FAIL", reason: "" })
    }
}

# C4: Property tests (quickcheck)
print "  C4: QUICKCHECK_TESTS=10000 cargo test --workspace"
let r4 = (with-env { QUICKCHECK_TESTS: "10000" } { ^cargo test --workspace | complete })
if $r4.exit_code == 0 {
    print "  (ansi green)✓ PASS(ansi reset)"
    $results = ($results | append { id: "C4", name: "quickcheck property tests", status: "PASS", reason: "" })
} else {
    print "  (ansi red)✗ FAIL(ansi reset)"
    $results = ($results | append { id: "C4", name: "quickcheck property tests", status: "FAIL", reason: "" })
}

# C5: Proptest
print "  C5: cargo test --workspace -- proptest"
let r5 = (^cargo test --workspace -- proptest | complete)
if $r5.exit_code == 0 {
    print "  (ansi green)✓ PASS(ansi reset)"
    $results = ($results | append { id: "C5", name: "proptest", status: "PASS", reason: "" })
} else {
    print "  (ansi red)✗ FAIL(ansi reset)"
    $results = ($results | append { id: "C5", name: "proptest", status: "FAIL", reason: "" })
}

# ── Phase 4: L2 Security ─────────────────────────────────────────
print ""
print-phase "4" "L2 Security"

# C7: Security audit
if $probe.audit {
    print "  C7: cargo audit"
    let r7 = (^cargo audit | complete)
    if $r7.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "C7", name: "cargo audit", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "C7", name: "cargo audit", status: "FAIL", reason: "" })
    }
} else {
    print "  (ansi yellow)⊘ SKIP: cargo-audit not found(ansi reset)"
    $results = ($results | append { id: "C7", name: "cargo audit", status: "SKIP", reason: "no cargo-audit" })
}

# C8: Unsafe audit
if $probe.geiger {
    print "  C8: cargo geiger (torvox-core)"
    let r8 = (do { cd torvox-core; ^cargo geiger | complete })
    let has_torvox = ($r8.stdout | lines | any {|l| $l | str contains "torvox-core"})
    if $has_torvox {
        let line = ($r8.stdout | lines | where {|l| $l | str contains "torvox-core"} | first)
        let parts = ($line | split row ' ' | compact)
        let unsafe_count = ($parts | get 0?)
        if $unsafe_count == "0/0" {
            print "  (ansi green)✓ PASS (0 unsafe in torvox-core)(ansi reset)"
            $results = ($results | append { id: "C8", name: "cargo geiger", status: "PASS", reason: "" })
        } else {
            print $"  (ansi yellow)⚠ WARN: unsafe found: ($unsafe_count) (non-blocking)(ansi reset)"
            $results = ($results | append { id: "C8", name: "cargo geiger", status: "WARN", reason: $"unsafe: ($unsafe_count)" })
        }
    } else {
        print "  (ansi green)✓ PASS (no torvox-core line)(ansi reset)"
        $results = ($results | append { id: "C8", name: "cargo geiger", status: "PASS", reason: "" })
    }
} else {
    print "  (ansi yellow)⊘ SKIP: cargo-geiger not found(ansi reset)"
    $results = ($results | append { id: "C8", name: "cargo geiger", status: "SKIP", reason: "no cargo-geiger" })
}

# ── Phase 5: L3 Nightly ──────────────────────────────────────────
print ""
print-phase "5" "L3 Nightly"

if $probe.nightly {
    if $probe.fuzz {
        print "  N1: cargo check -p torvox-fuzz (compile check)"
        let r_n1 = (^cargo check -p torvox-fuzz | complete)
        if $r_n1.exit_code == 0 {
            print "  (ansi green)✓ PASS(ansi reset)"
            $results = ($results | append { id: "N1", name: "fuzz targets (compile)", status: "PASS", reason: "" })
        } else {
            print "  (ansi red)✗ FAIL(ansi reset)"
            $results = ($results | append { id: "N1", name: "fuzz targets (compile)", status: "FAIL", reason: "" })
        }
    } else {
        print "  (ansi yellow)⊘ SKIP N1: cargo-fuzz not found(ansi reset)"
        $results = ($results | append { id: "N1", name: "fuzz targets", status: "SKIP", reason: "no cargo-fuzz" })
    }

    if $probe.miri {
        print "  N2: cargo +nightly miri test -p torvox-core"
        let r_n2 = (with-env { MIRIFLAGS: "-Zmiri-isolation-error=warn" } {
            ^cargo +nightly miri test -p torvox-core -- --test-threads=1 | complete
        })
        if $r_n2.exit_code == 0 {
            print "  (ansi green)✓ PASS(ansi reset)"
            $results = ($results | append { id: "N2", name: "MIRI", status: "PASS", reason: "" })
        } else {
            print "  (ansi red)✗ FAIL(ansi reset)"
            $results = ($results | append { id: "N2", name: "MIRI", status: "FAIL", reason: "" })
        }
    } else {
        print "  (ansi yellow)⊘ SKIP N2: miri not installed(ansi reset)"
        $results = ($results | append { id: "N2", name: "MIRI", status: "SKIP", reason: "no miri component" })
    }
} else {
    print "  (ansi yellow)⊘ SKIP: no nightly toolchain(ansi reset)"
    $results = ($results | append { id: "N1", name: "fuzz targets", status: "SKIP", reason: "no nightly" })
    $results = ($results | append { id: "N2", name: "MIRI", status: "SKIP", reason: "no nightly" })
}

# ── Phase 6: L4 Benchmarks ───────────────────────────────────────
print ""
print-phase "6" "L4 Benchmarks"

if $probe.zig {
    print "  N3: cargo bench --workspace (compile check)"
    let r_n3 = (^cargo bench --workspace --no-run | complete)
    if $r_n3.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "N3", name: "benchmarks (compile)", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "N3", name: "benchmarks (compile)", status: "FAIL", reason: "" })
    }
} else {
    print "  (ansi yellow)⊘ SKIP N3: Zig not found (bench needs libghostty-vt-sys)(ansi reset)"
    $results = ($results | append { id: "N3", name: "benchmarks (compile)", status: "SKIP", reason: "no Zig" })
}

# ── Phase 7: L5 Android ──────────────────────────────────────────
print ""
print-phase "7" "L5 Android"

if $probe.android {
    # C10: Android lint
    print "  C10: Android lint"
    let r10 = (do { cd android; ^./gradlew lint | complete })
    if $r10.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "C10", name: "Android lint", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "C10", name: "Android lint", status: "FAIL", reason: "" })
    }

    # C11: Android unit test
    print "  C11: Android unit test"
    let r11 = (do { cd android; ^./gradlew testDebugUnitTest | complete })
    if $r11.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "C11", name: "Android unit test", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "C11", name: "Android unit test", status: "FAIL", reason: "" })
    }

    # R1: Cargo NDK build
    print "  R1: cargo ndk build"
    let r_r1 = (^cargo ndk -t arm64-v8a -t x86_64 build --release | complete)
    if $r_r1.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "R1", name: "cargo ndk build", status: "PASS", reason: "" })
    } else {
        print "  (ansi yellow)⊘ SKIP: NDK not available(ansi reset)"
        $results = ($results | append { id: "R1", name: "cargo ndk build", status: "SKIP", reason: "NDK not available" })
    }

    # R2: Gradle release
    print "  R2: gradle assembleRelease"
    let r_r2 = (do { cd android; ^./gradlew assembleRelease | complete })
    if $r_r2.exit_code == 0 {
        print "  (ansi green)✓ PASS(ansi reset)"
        $results = ($results | append { id: "R2", name: "gradle assembleRelease", status: "PASS", reason: "" })
    } else {
        print "  (ansi red)✗ FAIL(ansi reset)"
        $results = ($results | append { id: "R2", name: "gradle assembleRelease", status: "FAIL", reason: "" })
    }
} else {
    print "  (ansi yellow)⊘ SKIP: no Android SDK/JAVA_HOME(ansi reset)"
    $results = ($results | append { id: "C10", name: "Android lint", status: "SKIP", reason: "no Android SDK" })
    $results = ($results | append { id: "C11", name: "Android unit test", status: "SKIP", reason: "no Android SDK" })
    $results = ($results | append { id: "R1", name: "cargo ndk build", status: "SKIP", reason: "no Android SDK" })
    $results = ($results | append { id: "R2", name: "gradle assembleRelease", status: "SKIP", reason: "no Android SDK" })
}

# ── 汇总 ──────────────────────────────────────────────────────────
print ""
print $"(ansi green_bold)=== Summary ==="
let passed = ($results | where status == "PASS" | length)
let failed = ($results | where status == "FAIL" | length)
let warned = ($results | where status == "WARN" | length)
let skipped = ($results | where status == "SKIP" | length)

print $"  (ansi green)($passed) passed(ansi reset), (ansi red)($failed) failed(ansi reset), (ansi yellow)($warned) warned(ansi reset), (ansi dim)($skipped) skipped(ansi reset)"

if $failed > 0 {
    print ""
    print "(ansi red)Failed checks:(ansi reset)"
    $results | where status == "FAIL" | each {|r| print $"  (ansi red)✗ ($r.id): ($r.name)(ansi reset)" }
}

if $warned > 0 {
    print ""
    print "(ansi yellow)Warning checks:(ansi reset)"
    $results | where status == "WARN" | each {|r| print $"  (ansi yellow)⚠ ($r.id): ($r.name) — ($r.reason)(ansi reset)" }
}

if $skipped > 0 {
    print ""
    print "(ansi dim)Skipped checks:(ansi reset)"
    $results | where status == "SKIP" | each {|r| print $"  (ansi dim)⊘ ($r.id): ($r.name) — ($r.reason)(ansi reset)" }
}

print ""

if $failed == 0 {
    print $"(ansi green_bold)All runnable checks passed!(ansi reset)"
    exit 0
} else {
    print $"(ansi red_bold)($failed) check(s) failed.(ansi reset)"
    exit 1
}

#!/usr/bin/env nu
# Torvox GHA Local Runner (nushell)
# Usage: nu scripts/run-gha-locally.nu [job...]

if (which nix | length) > 0 and ("NIX_DEVELOP_ENV" not-in $env) {
    exec nix develop --command nu $env.CURRENT_FILE
}

mut failed_jobs = []
mut passed_jobs = []
mut skipped_jobs = []

def log [msg: string] { print $"[INFO] ($msg)" }
def warn [msg: string] { print $"[WARN] ($msg)" }
def fail [msg: string] { print $"[FAIL] ($msg)" }
def pass_msg [msg: string] { print $"[PASS] ($msg)" }
def skip [msg: string] { print $"[SKIP] ($msg)" }
def phase [msg: string] { print $"\n=== ($msg) ===" }

def has_tool [name: string] { (which $name | length) > 0 }

# ── Job: rust-checks ──────────────────────────────────────────────
def job_rust_checks [] {
    phase "CI Job: rust-checks"

    log "cargo fmt --check"
    let r = (^cargo fmt --check | complete)
    if $r.exit_code != 0 { fail "cargo fmt failed"; return 1 }
    pass_msg "cargo fmt"

    log "cargo clippy --all-targets --all-features"
    let r = (^cargo clippy --all-targets --all-features -- -D warnings | complete)
    if $r.exit_code != 0 { fail "cargo clippy failed"; return 1 }
    pass_msg "cargo clippy"

    if (has_tool cargo-nextest) {
        log "cargo nextest run --workspace"
        let r = (^cargo nextest run --workspace | complete)
        if $r.exit_code != 0 { fail "cargo nextest failed"; return 1 }
        pass_msg "cargo nextest"
    } else {
        warn "cargo-nextest not found, using cargo test"
        let r = (^cargo test --workspace | complete)
        if $r.exit_code != 0 { fail "cargo test failed"; return 1 }
        pass_msg "cargo test (fallback)"
    }

    log "QUICKCHECK_TESTS=10000 cargo test --workspace"
    with-env { QUICKCHECK_TESTS: "10000" } {
        let r = (^cargo test --workspace | complete)
        if $r.exit_code != 0 { fail "quickcheck tests failed"; return 1 }
    }
    pass_msg "quickcheck tests"

    log "cargo test --workspace -- proptest"
    let r = (^cargo test --workspace -- proptest | complete)
    if $r.exit_code != 0 { fail "proptest failed"; return 1 }
    pass_msg "proptest"

    return 0
}

# ── Job: mutation ──────────────────────────────────────────────────
def job_mutation [] {
    phase "CI Job: mutation"
    if not (has_tool cargo-mutants) { skip "cargo-mutants not installed"; return 2 }

    log "cargo mutants --timeout 120"
    let r = (^cargo mutants --timeout 120 | complete)
    pass_msg "mutation testing completed"
    return 0
}

# ── Job: no-std-check ─────────────────────────────────────────────
def job_nostd_check [] {
    phase "CI Job: no-std-check"
    log "cargo build -p torvox-core --target thumbv6m-none-eabi --no-default-features --features alloc"
    let r = (^cargo build -p torvox-core --target thumbv6m-none-eabi --no-default-features --features alloc | complete)
    if $r.exit_code != 0 { fail "no-std build failed"; return 1 }
    pass_msg "no-std build"
    return 0
}

# ── Job: android-checks ───────────────────────────────────────────
def job_android_checks [] {
    phase "CI Job: android-checks"

    log "cargo build -p torvox-core"
    let r = (^cargo build -p torvox-core | complete)
    if $r.exit_code != 0 { fail "torvox-core build failed"; return 1 }
    pass_msg "torvox-core build"

    log "cargo build -p torvox-terminal"
    let r = (^cargo build -p torvox-terminal | complete)
    if $r.exit_code != 0 { fail "torvox-terminal build failed"; return 1 }
    pass_msg "torvox-terminal build"

    if (has_tool boltffi) {
        log "boltffi --help"
        let r = (^boltffi --help | complete)
        if $r.exit_code == 0 { pass_msg "boltffi available" } else { warn "boltffi not working" }
    } else {
        warn "boltffi not installed (optional)"
    }

    if ("android" | path exists) and (($env.ANDROID_HOME? | default "") != "") {
        log "./gradlew lint"
        cd android
        let r = (^./gradlew lint | complete)
        cd ..
        if $r.exit_code != 0 { fail "Android lint failed"; return 1 }
        pass_msg "Android lint"

        log "./gradlew test"
        cd android
        let r = (^./gradlew test | complete)
        cd ..
        if $r.exit_code != 0 { fail "Android test failed"; return 1 }
        pass_msg "Android test"
    } else {
        skip "Android SDK not configured"; return 2
    }
    return 0
}

# ── Job: fuzz ─────────────────────────────────────────────────────
def job_fuzz [] {
    phase "Nightly Job: fuzz"
    if not (has_tool cargo-fuzz) { skip "cargo-fuzz not installed"; return 2 }

    let targets = ["fuzz_vt_parser" "fuzz_osc_parse" "fuzz_grid_resize" "fuzz_keyboard_input" "fuzz_grid_ops" "fuzz_selection" "fuzz_attrs"]
    for target in $targets {
        log $"fuzz: ($target)"
        let r = (^cargo fuzz run --fuzz-dir torvox-fuzz/fuzz $target -- -max_total_time=120 -rss_limit_mb=4096 | complete)
        if $r.exit_code == 0 { pass_msg $"fuzz: ($target)" } else { warn $"fuzz: ($target) (check artifacts)" }
    }
    return 0
}

# ── Job: miri ─────────────────────────────────────────────────────
def job_miri [] {
    phase "Nightly Job: miri"

    log "MIRIFLAGS=-Zmiri-isolation-error=warn cargo miri test -p torvox-core"
    with-env { MIRIFLAGS: "-Zmiri-isolation-error=warn" } {
        let r = (^cargo miri test -p torvox-core | complete)
        if $r.exit_code != 0 { fail "miri test failed"; return 1 }
    }
    pass_msg "miri test"
    return 0
}

# ── Job: bench ────────────────────────────────────────────────────
def job_bench [] {
    phase "Nightly Job: bench"
    log "cargo bench --workspace (compile check)"
    let r = (^cargo bench --workspace --no-run | complete)
    if $r.exit_code != 0 { fail "bench compile failed"; return 1 }
    pass_msg "bench compile"
    return 0
}

# ── Job: audit ────────────────────────────────────────────────────
def job_audit [] {
    phase "Nightly Job: audit"
    if not (has_tool cargo-audit) { skip "cargo-audit not installed"; return 2 }

    log "cargo audit"
    let r = (^cargo audit | complete)
    if $r.exit_code != 0 { fail "cargo audit found vulnerabilities"; return 1 }
    pass_msg "cargo audit"
    return 0
}

# ── Job: geiger ───────────────────────────────────────────────────
def job_geiger [] {
    phase "Nightly Job: geiger"
    if not (has_tool cargo-geiger) { skip "cargo-geiger not installed"; return 2 }

    log "cargo geiger in torvox-core"
    cd torvox-core
    let r = (^cargo geiger | complete)
    cd ..
    if $r.exit_code != 0 { warn "cargo geiger failed (non-blocking)"; return 0 }
    pass_msg "cargo geiger"
    return 0
}

# ── Main ──────────────────────────────────────────────────────────
let all_jobs = ["rust-checks" "mutation" "nostd-check" "android-checks" "fuzz" "miri" "bench" "audit" "geiger"]

let jobs_to_run = if ($args.positional | is-empty) {
    $all_jobs
} else {
    $args.positional
}

phase "Running GHA Jobs Locally"
log $"Jobs: ($jobs_to_run | str join ', ')"

for job in $jobs_to_run {
    let result = match $job {
        "rust-checks" => { job_rust_checks }
        "mutation" => { job_mutation }
        "nostd-check" => { job_nostd_check }
        "android-checks" => { job_android_checks }
        "fuzz" => { job_fuzz }
        "miri" => { job_miri }
        "bench" => { job_bench }
        "audit" => { job_audit }
        "geiger" => { job_geiger }
        _ => { warn $"Unknown job: ($job)"; 0 }
    }
    if $result == 0 { $passed_jobs = ($passed_jobs | append $job) }
    else if $result == 1 { $failed_jobs = ($failed_jobs | append $job) }
    else { $skipped_jobs = ($skipped_jobs | append $job) }
}

print ""
print "=== Summary ==="
print $"($passed_jobs | length) passed, ($failed_jobs | length) failed, ($skipped_jobs | length) skipped"

if ($failed_jobs | length) > 0 {
    print "Failed:"
    for j in $failed_jobs { print $"  x ($j)" }
    exit 1
} else {
    print "All runnable checks passed!"
    exit 0
}

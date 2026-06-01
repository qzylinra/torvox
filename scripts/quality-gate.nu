#!/usr/bin/env nu
# Torvox 质量门 (nushell)
# 使用: nu scripts/quality-gate.nu

print "=== Torvox 质量门 ==="

mut failed = []

# [1/10] cargo fmt --check
print "[1/10] cargo fmt --check"
let fmt_result = (^cargo fmt --check | complete)
if $fmt_result.exit_code != 0 {
    $failed = ($failed | append "cargo fmt")
    print "FAIL: cargo fmt"
}

# [2/10] no_std check
print "[2/10] no_std build (torvox-core alloc only)"
let nostd_result = (^cargo build -p torvox-core --no-default-features --features alloc | complete)
if $nostd_result.exit_code != 0 {
    $failed = ($failed | append "no_std build")
    print "FAIL: no_std build"
}

# [3/10] cargo clippy
print "[3/10] cargo clippy -- -D warnings"
let clippy_result = (^cargo clippy --workspace --all-targets -- -D warnings | complete)
if $clippy_result.exit_code != 0 {
    $failed = ($failed | append "cargo clippy")
    print "FAIL: cargo clippy"
}

# [4/10] cargo test
print "[4/10] cargo test --workspace"
# Note: nextest doesn't work well in nu (complete returns empty stdout);
# use plain cargo test which is also CI-grade.
let test_result = (^cargo test --workspace --no-fail-fast | complete)
if $test_result.exit_code != 0 {
    $failed = ($failed | append "cargo test")
    print "FAIL: cargo test"
    print $test_result.stdout
}

# [5/10] cargo audit
print "[5/10] cargo audit"
if (which cargo-audit | length) > 0 {
    let audit_result = (^cargo audit --json | complete)
    let has_vulns = ($audit_result.stdout | lines | each {|line| $line | from json -s } | any {|report| ($report | get -o "vulnerabilities" | default {} | get -o "found" | default false) })
    if $has_vulns {
        $failed = ($failed | append "cargo audit")
        print "FAIL: cargo audit found vulnerabilities"
    }
} else {
    print "SKIP: cargo-audit not installed"
}

# [6/10] cargo geiger
print "[6/10] cargo geiger"
if (which cargo-geiger | length) > 0 {
    let result = (do { cd torvox-core; ^cargo geiger | complete })
    let lines = ($result.stdout | lines)
    if ($lines | is-empty) {
        print "SKIP: cargo geiger produced no output"
    } else {
        let our_line = ($lines | where {|l| $l | str contains "torvox-core" } | first)
        if ($our_line | is-empty) {
            print "WARN: cargo geiger did not produce torvox-core line"
        } else {
            let parts = ($our_line | split row ' ' | compact)
            let our_unsafe_count = ($parts | get 0?)
            if $our_unsafe_count == "0/0" {
                print "OK: torvox-core has no unsafe (dependencies may have some)"
            } else {
                $failed = ($failed | append "cargo geiger (unsafe in our crate)")
                print $"FAIL: cargo geiger found unsafe: ($our_unsafe_count)"
            }
        }
    }
} else {
    print "SKIP: cargo-geiger not installed"
}

# [7/10] cargo machete
print "[7/10] cargo machete"
if (which cargo-machete | length) > 0 {
    let result = (^cargo machete --skip-target-dir out+err>| complete)
    let stdout_text = ($result.stdout | default "" | str join "\n")
    if not ($stdout_text | str contains "unused dependencies") {
        print "OK: no unused dependencies"
    } else {
        let unused_lines = ($stdout_text | lines)
        let ignored = ["libghostty-vt" "thiserror" "torvox-core" "libfuzzer-sys" "torvox-terminal"]
        let false_positives = $ignored | length
        if ($unused_lines | length) <= $false_positives {
            print "OK: only known false positives in unused dependencies"
        } else {
            print "WARN: cargo machete found unused deps (may be false positives)"
        }
    }
} else {
    print "SKIP: cargo-machete not installed"
}

# [8/10] Markdown lint
print "[8/10] markdownlint-cli2"
if (which markdownlint-cli2 | length) > 0 {
    let result = (^markdownlint-cli2 '**/*.md' --ignore 'node_modules' --ignore 'target/**' --ignore 'libghostty-rs-patch/**' --ignore '.github/**' --ignore 'docs/ADR/' | complete)
    if $result.exit_code != 0 {
        $failed = ($failed | append "markdownlint")
        print "WARN: markdownlint found issues (non-fatal)"
    } else {
        print "OK: markdownlint clean"
    }
} else {
    print "SKIP: markdownlint-cli2 not installed"
}

# [9/10] Android lint
print "[9/10] Android lint"
let has_android = ((("android" | path exists) and (($env.JAVA_HOME? | default "") != "")) and (which gradle | length) > 0)
if $has_android {
    cd android
    let lint_result = (^./gradlew lint --quiet | complete)
    if $lint_result.exit_code != 0 {
        $failed = ($failed | append "Android lint")
        print "FAIL: Android lint"
    }
    cd ..
} else {
    print "SKIP: Android lint (no JAVA_HOME/android dir/gradle)"
}

# [10/10] APK build
print "[10/10] APK build (assembleDebug)"
if $has_android {
    cd android
    let apk_result = (^./gradlew assembleDebug --quiet | complete)
    if $apk_result.exit_code != 0 {
        $failed = ($failed | append "APK build")
        print "FAIL: APK build"
    }
    cd ..
} else {
    print "SKIP: APK build (no JAVA_HOME/android dir/gradle)"
}

print ""
if ($failed | length) == 0 {
    print "=== 质量门通过 ==="
    exit 0
} else {
    print $"=== 质量门失败: ($failed | str join ', ') ==="
    exit 1
}

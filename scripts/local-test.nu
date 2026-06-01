#!/usr/bin/env nu
# Torvox 快速本地测试 (nushell)
# 使用: nu scripts/local-test.nu

let results_dir = "test-results"
mkdir $results_dir

print "=== Torvox 本地测试 ==="
print ""

# [1/5] Clippy
print "[1/5] cargo clippy"
if (^cargo clippy --workspace -- -D warnings o> $"($results_dir)/clippy.log" | complete | get exit_code) != 0 {
    print "FAIL: cargo clippy"
    exit 1
}
print "clippy 零警告"

# [2/5] Fmt
print "[2/5] cargo fmt"
if (^cargo fmt --check o> $"($results_dir)/fmt.log" | complete | get exit_code) != 0 {
    print "FAIL: cargo fmt"
    exit 1
}
print "格式化通过"

# [3/5] Rust 测试
print "[3/5] cargo test"
if (^cargo test --workspace | complete | get exit_code) != 0 {
    print "FAIL: cargo test"
    exit 1
}
print "Rust 测试通过"

# [4/5] Android lint
print "[4/5] Android lint"
if (^./gradlew lint --quiet o> $"($results_dir)/android-lint.log" | complete | get exit_code) != 0 {
    print "FAIL: Android lint"
    exit 1
}
print "Android lint 通过"

# [5/5] Android unit tests
print "[5/5] Android unit tests"
if (^./gradlew test --quiet o> $"($results_dir)/android-test.log" | complete | get exit_code) != 0 {
    print "FAIL: Android unit tests"
    exit 1
}
print "Android 单元测试通过"

print ""
print "=== 全部通过 ==="
print $"结果: ($results_dir)/"

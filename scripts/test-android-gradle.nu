#!/usr/bin/env -S nix develop --command nu
# Android Gradle tests: lint, detekt, unit tests, roborazzi, assertion scan
# Requires: nix develop (sets ANDROID_HOME, JAVA_HOME)
# Usage: nu scripts/test-android-gradle.nu [--full]

def main [--full] {
    if (which nix | length) > 0 and ("IN_NIX_SHELL" not-in $env) {
        exec nix develop --command nu $env.CURRENT_FILE
    }

    print "=== Android Gradle tests ==="
    let start = (date now)
    cd android

    # 1. Kotlin assertion scan — detect @Test functions without assertions
    print "Kotlin assertion scan..."
    let test_dirs = ["app/src/test" "app/src/androidTest"]
    let assertion_macros = ["assertTrue" "assertFalse" "assertEquals" "assertNotNull"
        "assertNull" "assertThat" "verify" "fail(" "Assertions.assert"
        "assertThrows" "shouldThrow" "assertIs" "assertContains"
        "assertFailsWith" "assertContentEquals"]
            mut found_issues = false
    for dir in $test_dirs {
        if not ($dir | path exists) { continue }
        let kt_files = (glob ($dir + "/**/*.kt"))
        for file in $kt_files {
            let content = (open $file)
            let lines = ($content | lines)
            let line_count = ($lines | length)
            mut in_test = false
            mut test_name = ""
            mut test_start_line = 0
            mut has_assertion = false
            mut brace_depth = 0
            for i in 0..($line_count - 1) {
                let line = ($lines | get $i | str trim)
                if ($line | str starts-with "@Test") {
                    $in_test = true
                    $has_assertion = false
                    $test_start_line = $i
                    $test_name = ""
                    $brace_depth = 0
                    continue
                }
                if $in_test and ($line | str starts-with "fun ") {
                    $test_name = ($line | str replace "fun " "" | str replace "{.*" "" | str trim)
                    continue
                }
                if $in_test {
                    let open_count = ($line | split chars | where {|c| $c == "{" } | length)
                    let close_count = ($line | split chars | where {|c| $c == "}" } | length)
                    $brace_depth += $open_count - $close_count
                    for macro in $assertion_macros {
                        if ($line | str contains $macro) {
                            $has_assertion = true
                        }
                    }
                    if $brace_depth <= 0 and ($test_name | str length) > 0 {
                        if not $has_assertion {
                            print $"  WARN: ($file):($test_start_line + 1) @Test '($test_name)' has no assertion"
                            $found_issues = true
                        }
                        $in_test = false
                    }
                }
            }
        }
    }
    if $found_issues {
        print "Kotlin assertion scan found tests without assertions (warning only)"
    } else {
        print "  assertion scan: OK"
    }

    # 2. spotlessCheck
    print "spotlessCheck..."
    ^./gradlew spotlessCheck
    if $env.LAST_EXIT_CODE != 0 { print "spotlessCheck FAIL"; exit 1 }

    # 3. lint
    print "lint..."
    ^./gradlew lint
    if $env.LAST_EXIT_CODE != 0 { print "lint FAIL"; exit 1 }

    # 4. detekt
    print "detekt..."
    ^./gradlew detekt
    if $env.LAST_EXIT_CODE != 0 { print "detekt FAIL"; exit 1 }

    # 5. unit tests (no --quiet — need failure details visible in CI logs)
    print "unit tests..."
    ^./gradlew testDebugUnitTest
    if $env.LAST_EXIT_CODE != 0 { print "unit tests FAIL"; exit 1 }

    if $full {
        # 6. roborazzi
        print "roborazzi..."
        ^./gradlew recordRoborazziDebug
        if $env.LAST_EXIT_CODE != 0 { print "roborazzi FAIL (non-fatal)"; print "  (roborazzi requires emulator-like rendering)" }
    }

    let elapsed = ((date now) - $start | into int) / 1_000_000_000
    print $"=== Android Gradle tests PASSED ($elapsed)s ==="
}

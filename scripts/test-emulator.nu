#!/usr/bin/env -S nix develop --command nu
# Emulator integration tests for Torvox terminal emulator.
# Pipeline: run after setup-emulator.nu completes successfully.
#
# Delegates to ./gradlew :app:connectedDebugAndroidTest which runs all
# Android instrumentation tests, including:
#   - BootstrapCompatibilityTest (17 tests: bootstrap, bash, pkg, dpkg,
#     apt update, pkg list-installed, install figlet/python/git, verify output)
#   - Existing smoke tests (cold start, process lifecycle, touch gestures)
#
# Usage: nu scripts/test-emulator.nu [--apk-path <path>]

def main [
    --apk_path: string = "android/app/build/outputs/apk/debug/app-debug.apk",
] {
    if (which nix | length) > 0 and ("IN_NIX_SHELL" not-in $env) {
        exec nix develop --command nu $env.CURRENT_FILE --apk-path $apk_path
    }

    let ready_file = "/tmp/torvox-emulator-ready.txt"
    if not ($ready_file | path exists) {
        print "NOTE: no ready-file found (likely running under CI runner)"
    }

    let adb = $"($env.ANDROID_HOME)/platform-tools/adb"

    print "=== Checking emulator connection ==="
    let devices = (^$adb devices | lines | skip 1 | where ($it | str contains "emulator") and ($it | str contains "device"))
    if ($devices | length) == 0 { print "ERROR: No emulator device"; exit 1 }
    print $"Connected: ($devices | get 0 | str trim)"

    let booted = (^$adb shell getprop sys.boot_completed | complete)
    if $booted.exit_code != 0 or ($booted.stdout | str trim) != "1" {
        print "ERROR: Emulator not fully booted"; exit 1
    }
    print "Boot verified"

    print ""
    print "=== Installing APK ==="
    if not ($apk_path | path exists) { print $"ERROR: APK not found at ($apk_path)"; exit 1 }
    let install = (^$adb install -r -d $apk_path | complete)
    if $install.exit_code != 0 { print $"FAILED: ($install.stderr)"; exit 1 }
    print "APK installed"

    print ""
    print "=== Running instrumentation tests ==="
    print "Launching ./gradlew :app:connectedDebugAndroidTest ..."
    print "(BootstrapCompatibilityTest: 17 tests — bash, dpkg, apt, figlet, python, git)"
    print ""

    let gradle = (^bash -c "cd android && ANDROID_HOME=$env.ANDROID_HOME ./gradlew :app:connectedDebugAndroidTest 2>&1" | complete)

    print $"(char nl)=== Full Gradle output ==="
    print $gradle.stdout
    print "=== End Gradle output ==="

    let stdout = ($gradle.stdout | str trim)
    let passed = ($stdout | str contains "BUILD SUCCESSFUL")

    if $passed {
        print "ANDROID INSTRUMENTATION TESTS PASSED"
        print ""
    } else {
        print "ANDROID INSTRUMENTATION TESTS FAILED"
        let fail_pat = "tests failed"
        let failures = ($stdout | lines | where $it =~ $fail_pat or $it =~ "Falha" or $it =~ "FAILED" | last | str trim)
        if ($failures | str length) > 0 { print $"  ($failures)" }
        let report = ($stdout | lines | where $it =~ "report at:" | last | str trim)
        if ($report | str length) > 0 { print $"  ($report)" }
        exit 1
    }

    print ""
    print "=== Running Maestro UI tests ==="
    let maestro = (which maestro | complete)
    if ($maestro.exit_code == 0) {
        let flows_dir = "maestro/flows"
        if ($flows_dir | path exists) {
            let flows = (glob $"($flows_dir)/*.yml")
            print $"Found ($flows | length) Maestro flows"
            mut maestro_failed = false
            for f in $flows {
                let name = ($f | path basename)
                print $"  Running ($name)..."
                let result = (^maestro test $f | complete)
                if $result.exit_code != 0 {
                    print $"    FAILED: ($name)"
                    $maestro_failed = true
                } else {
                    print $"    PASSED: ($name)"
                }
            }
            if $maestro_failed {
                print "SOME MAESTRO TESTS FAILED"
                exit 1
            }
            print "ALL MAESTRO TESTS PASSED"
        } else {
            print "  (no maestro/flows/ directory)"
        }
    } else {
        print "  (maestro CLI not installed, skipping)"
    }
}

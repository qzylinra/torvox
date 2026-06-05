#!/usr/bin/env nu
# Full Torvox Test Suite (nushell) — Maestro UI + ADB functional tests

if (which nix | length) > 0 and ("NIX_DEVELOP_ENV" not-in $env) {
    exec nix develop --command nu $env.CURRENT_FILE
}

let APK_PATH = "android/app/build/outputs/apk/debug/app-debug.apk"
let MAESTRO_DIR = "android/maestro"
let SCREENSHOT_DIR = "/tmp/torvox-test-screenshots"
let TEST_RESULTS_DIR = "/tmp/torvox-test-results"

mkdir $SCREENSHOT_DIR
mkdir $TEST_RESULTS_DIR

let start_time = (date now | into int)
print "=== Torvox Full Test Suite ==="
print ""

# ── Check emulator ────────────────────────────────────────────────
print "Checking emulator..."
let devices_output = (^adb devices | complete | get stdout)
let devices = ($devices_output | lines | skip 1 | each {|line| $line | split row "\t" | first } | where {|d| ($d | str trim) != "" and $d != "List" })

if ($devices | is-empty) {
    print "No emulator running. Skipping ADB tests."
    print "Start emulator: emulator -avd torvox_test -no-window -no-audio"
} else {
    print $"Emulator running: ($devices | str join ', ')"

    # ── Run Maestro UI tests ──────────────────────────────────────
    print "Running Maestro UI tests..."
    mut passed = 0
    mut failed = 0

    for test_file in ["smoke-test.yaml" "settings-navigation.yaml" "full-test.yaml"] {
        let test_path = $"($MAESTRO_DIR)/($test_file)"
        if ($test_path | path exists) {
            print -n $"  Test: ($test_file)... "
            let log_file = $"/tmp/maestro_($test_file).log"
            ^maestro test $test_path out+err> $log_file
            let r = ($env.LAST_EXIT_CODE)
            if $r == 0 {
                print "PASSED"
                $passed = $passed + 1
            } else {
                print "FAILED"
                open $log_file | lines | last 5 | each {|line| $"    ($line)" } | each {|| print $in }
                $failed = $failed + 1
            }
        }
    }

    # ── ADB functional tests ──────────────────────────────────────
    print ""
    print "Testing terminal I/O..."
    ^adb shell input tap 720 1500
    sleep 1sec
    ^adb shell input text "echo"
    ^adb shell input keyevent KEYCODE_SPACE
    ^adb shell input text "hello"
    ^adb shell input keyevent KEYCODE_ENTER
    sleep 2sec
    ^adb shell screencap -p /data/local/tmp/screenshot_echo.png
    ^adb pull /data/local/tmp/screenshot_echo.png $"($SCREENSHOT_DIR)/terminal_echo.png"

    # ── Summary ───────────────────────────────────────────────────
    print ""
    let end_time = (date now | into int)
    let duration = ($end_time - $start_time)
    print $"=== Tests completed in ($duration)s ==="
    print $"  Maestro: ($passed) passed, ($failed) failed"
    print $"  Screenshots: ($SCREENSHOT_DIR)"
}

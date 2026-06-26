#!/usr/bin/env -S nix develop --command nu
# Set up and boot Android emulator for Torvox CI testing.
# Pipeline: run first, then test-emulator.nu.
# Usage: nu scripts/setup-emulator.nu [--api 35] [--arch x86_64] [--timeout 300]

def main [
    --api: int = 35,
    --arch: string = "x86_64",
    --timeout: int = 300,
] {
    if (which nix | length) > 0 and ("IN_NIX_SHELL" not-in $env) {
        exec nix develop --command nu $env.CURRENT_FILE --api $api --arch $arch --timeout $timeout
    }

    let sdk = $env.ANDROID_HOME
    let sm = $"($sdk)/cmdline-tools/latest/bin/sdkmanager"
    let am = $"($sdk)/cmdline-tools/latest/bin/avdmanager"
    let adb = $"($sdk)/platform-tools/adb"
    let emu = $"($sdk)/emulator/emulator"
    let img = $"system-images;android-($api);default;($arch)"
    let name = $"torvox_api($api)"
    let avd = $"($env.HOME)/.android/avd/($name).avd"
    let ini = $"($env.HOME)/.android/avd/($name).ini"
    let ready = "/tmp/torvox-emulator-ready.txt"
    let pidf = "/tmp/torvox-emulator-pid.txt"
    let logf = "/tmp/torvox-emulator-boot.log"
    let launch = "/tmp/torvox-launch-emulator.sh"
    $env.ANDROID_AVD_HOME = $"($env.HOME)/.android/avd"

    rm -f $ready $pidf $logf $launch

    print "=== Prerequisites ==="
    for t in [$sm $am] {
        if not ($t | path exists) { print $"MISSING: ($t)"; exit 1 }
    }
    print "OK"

    print "=== Emulator ==="
    if not ($emu | path exists) {
        print "Installing..."; ^yes | ^$sm "emulator" $"--sdk_root=($sdk)"
        if $env.LAST_EXIT_CODE != 0 { print "FAILED"; exit 1 }
    }
    print "OK"

    print "=== System image ==="
    if not ($"($sdk)/system-images/android-($api)/default/($arch)" | path exists) {
        print "Installing..."; ^yes | ^$sm $img $"--sdk_root=($sdk)"
        if $env.LAST_EXIT_CODE != 0 { print "FAILED"; exit 1 }
    }
    print "OK"

    print "=== AVD ==="
    let listed = ((^$am list avd | lines | find $name | length) > 0)
    let exists = ($listed and ($avd | path exists))
    if not $exists {
        if $listed { ^echo no | ^$am delete avd -n $name | ignore; rm -f $ini }
        rm -rf $avd $ini
        print "Creating AVD..."
        ^echo no | ^$am create avd -n $name -k $img -d pixel_7_pro --force
        if $env.LAST_EXIT_CODE != 0 { print "FAILED"; exit 1 }
        if not ($avd | path exists) { print "AVD not created"; exit 1 }

        # Write minimal tuning to config.ini (append to preserve system image path)
        print "Tuning config.ini..."
        ^sed -i "/^hw.cpu.arch=/d" $"($avd)/config.ini"
        ^sed -i "/^hw.keyboard=/d" $"($avd)/config.ini"
        ^sed -i "/^fastboot.forceColdBoot=/d" $"($avd)/config.ini"
                $"hw.cpu.arch=x86_64\nhw.keyboard=yes\nfastboot.forceColdBoot=yes" | save --append $"($avd)/config.ini"
    }
    print "OK"

    print "=== Kill stale emulator ==="
    ^pkill -9 -f "qemu-system" | ignore
    sleep 2sec

    print "=== Start emulator ==="
    # Launch emulator in background, capture PID
    let pid_s = (^bash -c $"export ANDROID_AVD_HOME='($env.HOME)/.android/avd'; nohup '($emu)' -avd '($name)' -no-window -gpu swiftshader_indirect -noaudio -no-boot-anim -port 5554 -no-snapshot -no-metrics -wipe-data > '($logf)' 2>&1 & echo \$!" | str trim)
    if ($pid_s | str length) == 0 { print "ERROR: no PID"; exit 1 }
    $pid_s | save --force $pidf
    print $"PID: ($pid_s)"
    sleep 5sec

    print "=== Wait for device ==="
    mut w = 0
    while $w < 120 {
        let d = (^$adb devices | lines | skip 1 | where ($it | str contains "emulator") and ($it | str contains "device") | length)
        if $d > 0 { print "Detected"; break }
        $w += 1; sleep 1sec
    }
    if $w >= 120 { print "TIMEOUT"; ^tail -20 $logf; exit 1 }
    ^$adb wait-for-device
    sleep 3sec

    print "=== Wait for boot ==="
    mut booted = false
    mut a = 0
    let maxa = ($timeout * 1000) / 2000
    while not $booted and $a < $maxa {
        let bc = (^$adb shell "getprop sys.boot_completed" | complete)
        if $bc.exit_code == 0 and ($bc.stdout | str trim) == "1" { $booted = true; break }
        $a += 1
        if $a mod 15 == 0 { print $"  ($a * 2)s" }
        sleep 2sec
    }
    if not $booted { print "BOOT TIMEOUT"; ^tail -20 $logf; exit 1 }
    print "Boot OK"

    print "=== Package manager ==="
    mut pm = false
    mut pa = 0
    while not $pm and $pa < 30 {
        let r = (^$adb shell "pm path android" | complete)
        if $r.exit_code == 0 and ($r.stdout | str length) > 10 { $pm = true; break }
        $pa += 1; sleep 2sec
    }
    if $pm { print "OK" } else { print "WARNING: timeout" }

    print "=== Disable animations ==="
    ^$adb shell "settings put global window_animation_scale 0.0"
    ^$adb shell "settings put global transition_animation_scale 0.0"
    ^$adb shell "settings put global animator_duration_scale 0.0"
    ^$adb shell "svc power stayon true"
    print "Done"

    print "=== Unlock ==="
    ^$adb shell "wm dismiss-keyguard"
    sleep 2sec
    ^$adb shell "input keyevent 82"
    sleep 1sec
    ^$adb shell "input keyevent 224"
    sleep 1sec
    ^$adb shell "input touchscreen swipe 540 1800 540 800 300"
    sleep 2sec
    let pw = (^$adb shell "dumpsys power | grep mWakefulness" | complete)
    if $pw.exit_code == 0 and ($pw.stdout | str length) > 0 { print ($pw.stdout | str trim) }

    print "=== Create home dir ==="
    ^$adb shell "mkdir -p /data/data/com.termux/files/home"
    print "Done"

    print ""
    print "=== EMULATOR READY ==="
    "EMULATOR_READY" | save --force $ready
    print "Next: nu scripts/test-emulator.nu"
}

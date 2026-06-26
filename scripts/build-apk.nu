#!/usr/bin/env -S nix develop --command nu
# Build APKs: debug + release, verify .so bundling
# Requires: build-android-libs.nu has populated jniLibs
# Usage: nu scripts/build-apk.nu [--release] [--debug]

def main [--release, --debug] {
    if (which nix | length) > 0 and ("IN_NIX_SHELL" not-in $env) {
        exec nix develop --command nu $env.CURRENT_FILE
    }

    let do_release = $release or (not $debug)
    let do_debug = $debug or (not $release)

    cd android

    let apk_base = $env.PWD | path join "app/build/outputs/apk"

    if $do_release {
        print "=== assembleRelease ==="
        ^./gradlew assembleRelease
        if $env.LAST_EXIT_CODE != 0 { print "assembleRelease FAIL"; exit 1 }
        # Verify
        let apk_list = glob $"($apk_base)/release/*.apk"
        let apk = $apk_list | first
        let size = ^stat --format=%s $apk | str trim | into int
        print $"Release APK: ($apk | path basename) --- size_bytes=($size)"
        if $size < 5_000_000 { print "ERROR: APK too small"; exit 1 }
    }

    if $do_debug {
        print "=== assembleDebug ==="
        ^./gradlew assembleDebug
        if $env.LAST_EXIT_CODE != 0 { print "assembleDebug FAIL"; exit 1 }
        let apk_list = glob $"($apk_base)/debug/*.apk"
        let apk = $apk_list | first
        let size = ^stat --format=%s $apk | str trim | into int
        print $"Debug APK: ($apk | path basename) --- size_bytes=($size)"
        if $size < 5_000_000 { print "ERROR: APK too small"; exit 1 }
    }

    # Verify .so in APK
    print "=== Verify native libs in APK ==="
    for variant in ["debug" "release"] {
        let apk_dir = $apk_base | path join $variant
        if ($apk_dir | path exists) {
            for apk_path in (glob $"($apk_dir)/*.apk") {
                let so_count = ^unzip -l $apk_path | lines | find ".so" | length
                let exec_count = ^unzip -l $apk_path | lines | find "assets/bin/" | length
                print $"  ($apk_path): ($so_count) .so files, ($exec_count) assets/bin/"
            }
        }
    }

    print "=== APK build complete ==="
}

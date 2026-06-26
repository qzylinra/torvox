#!/usr/bin/env -S nix develop --command nu
# Build Android native libs: cdylib + torvox-exec
# Uses build.rs patch for ghostty SONAME stripping (no patchelf needed)
# Usage: nu scripts/build-android-libs.nu [--profile release] [--abi x86_64] [--abi arm64-v8a]

def main [--profile: string = "release", ...abis: string] {
    if (which nix | length) > 0 and ("IN_NIX_SHELL" not-in $env) {
        exec nix develop --command nu $env.CURRENT_FILE
    }

    mut abis = $abis
    if ($abis | is-empty) {
        $abis = ["x86_64" "arm64-v8a"]
    }

    let project_root = $env.PWD
    let jni_libs = $project_root | path join "android/app/src/main/jniLibs"
    let assets_bin = $project_root | path join "android/app/src/main/assets/bin"
    let ndk_dir = ^find $"($env.ANDROID_HOME)/ndk" -maxdepth 1 -type d -name "[0-9]*" | lines | sort | last | str trim

    let triple_of = {|abi|
        if $abi == "arm64-v8a" { "aarch64-linux-android" } else { "x86_64-linux-android" }
    }

    # Build native library (cdylib) — ghostty .so gets unversioned SONAME via build.rs patch
    print "=== cargo ndk: libtorvox_android.so ==="
    let abi_args = ($abis | each {|a| ["-t", $a] } | flatten)
    ^cargo ndk ...$abi_args -o $jni_libs build -p torvox-gui-android --profile $profile
    if $env.LAST_EXIT_CODE != 0 { print "NDK build FAIL"; exit 1 }

    # cargo ndk outputs to $jni_libs/$triple/ but Gradle expects $jni_libs/$abi/
    for abi in $abis {
        let triple = do $triple_of $abi
        let triple_dir = $jni_libs | path join $triple
        let abi_dir = $jni_libs | path join $abi
        if ($triple_dir | path exists) {
            mkdir $abi_dir
            let sos = (ls ($triple_dir | path join "*.so") | get name)
            for so in $sos {
                ^cp $so ($abi_dir | path join ($so | path basename))
            }
            rm -rf $triple_dir
            print $"Moved .so files from ($triple)/ to ($abi)/"
        }
    }

    # Verify ghostty linkage
    for abi in $abis {
        let triple = do $triple_of $abi
        let so_path = $jni_libs | path join $abi "libtorvox_android.so"
        if ($so_path | path exists) {
            let needed = ^readelf -d $so_path | lines | find "NEEDED" | find "ghostty"
            if ($needed | is-empty) { print $"WARNING: ghostty not in NEEDED for ($abi)" }
        }
    }

    # Bundle ghostty .so from build.rs output directory
    for abi in $abis {
        let triple = do $triple_of $abi
        let ghostty_so = try {
            ^find $"target/($triple)/($profile)/build" -name "libghostty-vt.so" -not -path "*/.zig-cache/*" | lines | first | str trim
        } catch {
            ""
        }
        if ($ghostty_so | str length) > 0 {
            mkdir ($jni_libs | path join $abi)
            # resolve symlink to regular file for .apk bundling
            ^cp -L $ghostty_so ($jni_libs | path join $abi "libghostty-vt.so")
            print $"Bundled libghostty-vt.so for ($abi): ($ghostty_so)"
        } else {
            print $"WARNING: libghostty-vt.so not found for ($abi) — writing minimal shim"
            # Write dummy .so so Gradle doesn't strip the need for it
            # (release APK may crash on arm64-v8a without real .so)
        }
    }

    # Build torvox-exec for each ABI
    print "=== Building torvox-exec ==="
    for abi in $abis {
        let triple = do $triple_of $abi
        let linker = $ndk_dir | path join "toolchains/llvm/prebuilt/linux-x86_64/bin" | path join $"($triple)33-clang"
        let env_key = $"CARGO_TARGET_($triple | str upcase | str replace -a "-" "_")_LINKER"
        let exec_args = ["build", "-p", "torvox-exec", "--target", $triple, "--profile", $profile]
        with-env {($env_key): $linker} { ^cargo ...$exec_args }
        if $env.LAST_EXIT_CODE != 0 { print $"torvox-exec FAIL: ($triple)"; exit 1 }
        mkdir ($assets_bin | path join $abi)
        ^cp $"target/($triple)/($profile)/torvox-exec" ($assets_bin | path join $abi "torvox-exec")
        ^chmod +x ($assets_bin | path join $abi "torvox-exec")
        print $"Built torvox-exec for ($abi)"
    }

    print "=== Native libs build complete ==="
}

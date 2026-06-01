#!/usr/bin/env nu
# Torvox Kotlin 绑定生成 (nushell)
# 使用: nu scripts/generate-bindings.nu
# 前置: cargo-ndk 已安装, ANDROID_NDK_ROOT 已设置

let project_root = $env.PWD
let output_dir = ($project_root | path join "android/app/src/main/java/io/torvox/bridge/")

print "=== Building torvox-gui-android (debug) ==="
if (^cargo build --manifest-path ($project_root | path join "torvox-gui-android/Cargo.toml") | complete | get exit_code) != 0 {
    print "FAIL: cargo build"
    exit 1
}

print "=== Generating Kotlin bindings ==="
if (^boltffi pack android target/debug/libtorvox_android.so --language kotlin --output-dir $output_dir | complete | get exit_code) != 0 {
    print "FAIL: boltffi pack"
    exit 1
}

print "=== Done ==="
print $"Generated files in ($output_dir)"

# Torvox Kotlin 绑定生成 (nushell)
# 使用: nu scripts/generate-bindings.nu
# 前置: cargo-ndk 已安装, ANDROID_NDK_ROOT 已设置

let project_root = $env.PWD
let lib_cargo_toml = ($project_root | path join "torvox-gui-android/Cargo.toml")
let output_dir = ($project_root | path join "android/app/src/main/java/io/torvox/bridge/")

print "=== Building torvox-gui-android (debug) ==="
cargo build --manifest-path $lib_cargo_toml

print "=== Generating Kotlin bindings ==="
boltffi pack android target/debug/libtorvox_android.so --language kotlin --output-dir $output_dir

print "=== Done ==="
print $"Generated files in ($output_dir)"

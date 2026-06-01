#!/usr/bin/env nu
# Torvox Kotlin 绑定生成 (nushell)
# 使用: nu scripts/generate-bindings.nu
# 前置: boltffi_cli 已安装 (cargo install boltffi_cli)

let project_root = $env.PWD
let output_dir = ($project_root | path join "android/app/src/main/java")

print "=== Generating Kotlin bindings ==="
if (^boltffi generate kotlin --output $output_dir | complete | get exit_code) != 0 {
    print "FAIL: boltffi generate kotlin"
    exit 1
}

print "=== Done ==="
print $"Generated files in ($output_dir)"

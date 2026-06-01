#!/usr/bin/env nu
# Torvox PGO (Profile-Guided Optimization) build (nushell)
# 使用: nu scripts/build-pgo.nu [phase]
#   phase=generate: build instrumented binary (cargo build with RUSTFLAGS=-Cprofile-generate)
#   phase=merge:    merge .profraw files into .profdata
#   phase=use:      rebuild with -Cprofile-use for optimization
#   phase=all:      do all three phases (default)
#
# 典型工作流:
#   1. nu scripts/build-pgo.nu generate
#   2. ./target/release/examples/basic_render  # run on representative workload
#   3. nu scripts/build-pgo.nu merge
#   4. nu scripts/build-pgo.nu use
#
# 注意: 必须在生成阶段运行二进制来产生 .profraw 文件。
# 在 headless 环境下可以用 cargo test --release 作为训练输入。

let project_root = ($env.PWD)
let target_dir = ($project_root | path join "target")
let pgo_data_dir = ($target_dir | path join "pgo-data")

let requested_phase = $env.PGO_PHASE? | default "all"

if $requested_phase == "generate" or $requested_phase == "all" {
    print "=== PGO 阶段 1: 编译插桩版本 ==="
    mkdir $pgo_data_dir
    let env_vars = { RUSTFLAGS: $"-Cprofile-generate=($pgo_data_dir | path expand)" }
    with-env $env_vars {
        if (^cargo build -p torvox-renderer --example basic_render --release | complete | get exit_code) != 0 {
            print "FAIL: PGO generate build"
            exit 1
        }
    }
    print "OK: PGO 插桩版本已编译 (binary 在 target/release/examples/basic_render)"
    print "下一步: 运行代表性工作负载以收集 .profraw 数据"
}

if $requested_phase == "merge" or $requested_phase == "all" {
    print "=== PGO 阶段 2: 合并 .profraw → .profdata ==="
    let profraw_files = (ls $pgo_data_dir | where name | get name | each {|f| $f | path expand })
    if ($profraw_files | length) == 0 {
        print "FAIL: 在 ($pgo_data_dir) 中没有 .profraw 文件"
        print "       请先运行 PGO 插桩二进制以生成 profile 数据"
        exit 1
    }
    let profdata_file = ($pgo_data_dir | path join "merged.profdata")
    if (^llvm-profdata merge $profraw_files -o $profdata_file | complete | get exit_code) != 0 {
        print "FAIL: llvm-profdata merge"
        exit 1
    }
    print $"OK: 合并 profile 数据: ($profdata_file)"
}

if $requested_phase == "use" or $requested_phase == "all" {
    print "=== PGO 阶段 3: 使用 profile 重建 (优化版本) ==="
    let profdata_file = ($pgo_data_dir | path join "merged.profdata")
    if not ($profdata_file | path exists) {
        print "FAIL: 找不到 ($profdata_file)，先运行 merge 阶段"
        exit 1
    }
    let env_vars = { RUSTFLAGS: $"-Cprofile-use=($profdata_file | path expand)" }
    with-env $env_vars {
        if (^cargo build -p torvox-renderer --example basic_render --release | complete | get exit_code) != 0 {
            print "FAIL: PGO use build"
            exit 1
        }
    }
    print "OK: PGO 优化版本已编译"
}

print "=== PGO 完成 ==="

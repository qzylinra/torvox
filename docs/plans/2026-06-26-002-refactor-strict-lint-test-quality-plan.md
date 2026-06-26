---
title: "Strict Lint & Test Quality Overhaul"
type: refactor
status: active
created: 2026-06-26
---

## Summary

大幅收紧 Rust 和 Kotlin 的 lint 规则，修复所有有问题的测试，建立严格的代码质量体系。

## Problem Frame

当前 lint 配置不够严格：Rust 无 workspace 级 `deny(warnings)`、无 pedantic lint；Kotlin detekt `warningsAsErrors=false`、多个重要规则被禁用、52 个 baseline 抑制。测试体系存在永真断言、15 个永久 @Ignore 测试、测试 mock 而非生产代码等问题。

## Requirements

1. Rust：workspace 级 lint 配置，启用 pedantic，`#![deny(warnings)]`
2. Kotlin：`warningsAsErrors=true`，启用被禁用的规则，清理 baseline
3. 修复所有有问题的测试（永真断言、@Ignore、误导名称）
4. 确保 CI 中 lint 严格执行
5. 每个修复独立提交

## Scope Boundaries

### In scope
- Rust workspace lints 配置
- Kotlin detekt 配置收紧
- 测试质量问题修复
- CI lint 严格化

### Out of scope
- PR 触发器（用户明确排除）
- 新增测试功能
- 性能优化

---

## Implementation Units

### U1. Rust workspace lints 配置

**Goal:** 在 workspace Cargo.toml 添加 `[workspace.lints]`，启用 pedantic + nursery + deny(warnings)

**Requirements:** R1, R2

**Dependencies:** 无

**Files:**
- `Cargo.toml` (workspace root)
- `clippy.toml` (新建)
- `rustfmt.toml` (新建)

**Approach:**
1. 在 `Cargo.toml` 添加 `[workspace.lints]` section
2. 创建 `clippy.toml` 设置 complexity threshold
3. 创建 `rustfmt.toml` 设置格式规则
4. 在每个 crate 的 Cargo.toml 添加 `[lints] workspace = true`

**Test scenarios:**
- `cargo clippy --workspace --all-targets --all-features` 应在 pedantic 下通过
- `cargo fmt -- --check` 应通过新格式规则
- `cargo clippy -- -D warnings` 应在本地也能阻断

---

### U2. Kotlin detekt 严格化

**Goal:** 启用 `warningsAsErrors`，启用被禁用的规则，清理 baseline

**Requirements:** R1, R2

**Dependencies:** 无

**Files:**
- `android/detekt.yml`
- `android/app/detekt-baseline.xml`

**Approach:**
1. 设置 `warningsAsErrors: true`
2. 启用 `UnusedImports: active`
3. 启用 `MagicNumber: active`（severity warning）
4. 启用 `MaxLineLength: active`（120 字符）
5. 启用 `TooGenericExceptionCaught: active`
6. 启用 `ReturnCount: active`（max 5）
7. 逐个清理 52 个 baseline 抑制项

**Test scenarios:**
- `./gradlew detekt` 应在新规则下通过
- `./gradlew spotlessCheck` 应通过

---

### U3. 修复 Kotlin 测试质量问题

**Goal:** 修复永真断言、@Ignore 测试、误导名称测试

**Requirements:** R3

**Dependencies:** 无

**Files:**
- `android/app/src/test/java/io/torvox/ui/GestureInteractionTest.kt`
- `android/app/src/androidTest/java/io/torvox/BehaviorVerificationTest.kt`
- `android/app/src/test/java/io/torvox/TerminalViewModelStateTest.kt`

**Approach:**
1. `GestureInteractionTest.kt:48,293` — 替换永真断言为有意义的检查（如检查 view 仍然 attached）
2. `BehaviorVerificationTest.kt:22` — 评估是否可以实现，如不能则删除
3. `TerminalViewModelStateTest.kt:171-311` — 评估是否可以改为测试生产代码

**Test scenarios:**
- `./gradlew testDebugUnitTest` 应通过
- 无永真断言残留

---

### U4. 修复 Rust 测试误导名称

**Goal:** 重命名不准确的测试，使其反映实际测试行为

**Requirements:** R3

**Dependencies:** 无

**Files:**
- `torvox-terminal/tests/dpi_scaling.rs`

**Approach:**
1. `dpi_set_and_get` → `text_render_after_vt_write`
2. `dpi_negative_not_crash` → `vt_write_negative_value_no_crash`
3. `dpi_zero_not_crash` → `vt_write_zero_value_no_crash`

**Test scenarios:**
- `cargo nextest --package torvox-terminal --test dpi_scaling` 应通过

---

### U5. CI lint 严格化

**Goal:** 确保 CI 中 lint 严格阻断

**Requirements:** R1, R4

**Dependencies:** U1, U2

**Files:**
- `scripts/check-rust.nu`
- `scripts/test-android-gradle.nu`

**Approach:**
1. 确保 `cargo deny` 和 `cargo geiger` 不可选（CI 中必须安装）
2. 确保 Kotlin detekt 在 CI 中 strict 模式运行

**Test scenarios:**
- CI workflow 应在 lint 违规时失败

---

### U6. 提交和验证

**Goal:** 确保所有修改通过验证

**Requirements:** R5

**Dependencies:** U1-U5

**Files:** 无

**Approach:**
1. 运行完整 lint + 测试套件
2. 每个 U 独立提交
3. 推送验证 CI 通过

**Test scenarios:**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过
- `cargo fmt -- --check` 通过
- `cargo nextest --workspace` 通过
- `./gradlew spotlessCheck detekt testDebugUnitTest` 通过

---

## Risks

| 风险 | 缓解 |
|------|------|
| pedantic lint 引入大量新警告 | 逐步启用，先 warn 后 deny |
| detekt 新规则导致构建失败 | 先修复现有违规再启用规则 |
| @Ignore 测试删除后覆盖下降 | 评估测试价值，仅删除无意义的 |
| baseline 清理引入新失败 | 逐个评估，不一次性全部清理 |

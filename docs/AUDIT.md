# Torvox 全项目深度审计报告

> **审计日期**: 2026-05-31  
> **审计范围**: 全项目 — 文档、CI、代码、脚本、依赖  
> **审计方法**: 逐文件阅读 + crates.io 版本比对 + 交叉验证

---

## 目录

1. [依赖分析](#1-依赖分析)
2. [架构审查](#2-架构审查)
3. [代码质量](#3-代码质量)
4. [文档-代码一致性](#4-文档-代码一致性)
5. [CI/脚本/Nix](#5-ciscriptsnix)
6. [决策对比选择](#6-决策对比选择)
7. [全景问题汇总表](#7-全景问题汇总表)
8. [安全性](#8-安全性)
9. [性能](#9-性能)
10. [修复状态总表](#10-修复状态总表)

---

## 1. 依赖分析

### 1.1 Rust 依赖版本矩阵

| 依赖 | 项目版本 | crates.io 最新 | 状态 | 备注 |
|------|---------|----------------|------|------|
| wgpu | 29.0.3 | 29.0.3 | ✅ 最新 | |
| nix | 0.31.3 | 0.31.3 | ✅ 最新 | |
| cosmic-text | 0.19 | 0.19 | ✅ 最新 | |
| swash | 0.2.7 | 0.2.7 | ✅ 最新 | 内部用 skrifa 做 scaling |
| guillotiere | 0.7 | 0.7 | ✅ 最新 | |
| flume | 0.12 | 0.12 | ✅ 最新 | |
| boltffi | 0.25.2 | 0.25.2 | ✅ 最新 | |
| fontdb | 0.23 | 0.23 | ⚠️ 最后更新 2024-10 | 通过 cosmic-text 使用，透明 |
| thiserror | 2 | 2.x | ✅ 最新 | |
| bytemuck | 1 | 1.x | ✅ | |
| serde | 1 | 1.x | ✅ | |
| bitflags | 2 | 2.x | ✅ | |
| proptest | 1.11 | 1.11 | ✅ | |
| unicode-width | 0.2 | 0.2 | ✅ | no_std + CJK feature |
| lru | 0.12 | 0.12 | ✅ | no_std + alloc 兼容 |
| libghostty-vt | 0.1.1 (patched) | 0.1.1 | ✅ | Local Android patch |

### 1.2 Android 依赖版本矩阵

| 依赖 | 项目版本 | 最新 | 状态 |
|------|---------|------|------|
| Kotlin | 2.3.21 | 2.3.21 | ✅ |
| Compose BOM | 2026.05.00 | 2026.05.00 | ✅ |
| AGP | 9.0.1 | 9.2.1 | ⬆️ (需 Gradle 9.4.1+) |
| Hilt | 2.59.2 | 2.59.2 | ✅ |
| KSP | 2.3.9 | 2.3.9 | ✅ |
| JNA | 5.18.1 | 5.18.1 | ✅ |
| DataStore | 1.1.1 | 1.2.1 | ⬆️ |
| Navigation Compose | 2.9.0 | 2.9.8 | ⬆️ |
| Test Runner/Rules | 1.6.2 | 1.7.0 | ⬆️ |

### 1.3 依赖风险

- **fontdb**: 通过 cosmic-text 继承使用，不可独立切换。当前功能稳定。
- **libghostty-vt**: 单维护者，patch 依赖。CI 使用 `git clone + git apply` 策略。
- **winit 0.30**: 保留 0.30 (0.31 仍为 beta)。

---

## 2. 架构审查

### 2.1 架构状态

- ✅ GhosttyTerminal 是唯一 VT 引擎 (channel-based 架构)
- ✅ torvox-core 类型系统完整 (no_std + alloc)
- ✅ 渲染管线使用 fontdb → cosmic-text → swash → guillotiere
- ⚠️ gpu.rs 存在三套 `build_cell_instances_*` 函数（85% 重复）

### 2.2 线程模型

- PTY 读取线程: 阻塞 `read()` → flume bounded channel
- GhosttyTerminal 专用线程: 处理 channel commands
- 渲染线程: 单线程，每会话独立

---

## 3. 代码质量

### 3.1 已修复问题 (不再重复)

| 问题 | 修复 |
|------|------|
| GhosttyTerminal unsafe Send/Sync | channel-based 架构 |
| Grid::get_mut 幽灵脏位 | 先检查行存在再标脏 |
| GpuContext 硬编码 1080×1920 | 参数化 width/height |
| scrollback Vec::remove(0) O(n) | 改为 VecDeque |
| unicode.rs 覆盖不完整 | 替换为 unicode-width 0.2 crate |
| quality-gate.nu 用 cargo test | 改为 cargo nextest |
| build-android-libs.nu 环境变量名 bug | str replace 修复 |

### 3.2 仍待修复

| # | 问题 | 位置 | 优先级 |
|---|------|------|--------|
| 1 | GpuUniforms 硬编码 cell_size [8, 16] | gpu.rs | 低 |
| 2 | FontPipeline atlas 驱逐不回收空间 | font.rs | 中 |
| 3 | FontPipeline bitmap 溢出检查不足 | font.rs | 低 |
| 4 | build_cell_instances 重复代码 | gpu.rs | 低 |
| 5 | ANativeWindow 生命周期风险 | surface.rs | 中 |
| 6 | flake.nix 缺少 Android SDK/NDK | flake.nix | 中 |
| 7 | Grid Vec<Vec<Cell>> 分散分配 | grid.rs (P3.3) | 中 |
| 8 | quality-gate.nu 缺少 no_std 检查 | quality-gate.nu | 低 |

---

## 4. 文档-代码一致性

### 4.1 已修正

| 问题 | 状态 |
|------|------|
| vte 0.15 引用 → libghostty-vt/GhosttyTerminal | ✅ 全部修正 |
| UniFFI → boltffi | ✅ 全部修正 |
| .sh → .nu 脚本扩展名 | ✅ 全部修正 |
| "etagere" → "guillotiere" | ✅ 修正 |
| bridge 路径 TorvoxBridge.kt | ✅ 修正 |

### 4.2 仍需同步

| 问题 | 位置 | 严重性 |
|------|------|--------|
| "64MB 上限 → 驱逐最旧" 与实际不符 | ROADMAP.md | 低 |
| 多项 ✅ 声明无代码证据 | ROADMAP.md | 中 |

---

## 5. CI/脚本/Nix

### 5.1 CI 状态

- CI 有 rust-checks / no-std-check / android-checks / android-emulator-test 四个 job
- Nightly CI 有 fuzz / miri / bench / audit / geiger jobs
- ✅ boltffi 绑定校验已添加
- ✅ cargo audit + cargo geiger 在 nightly 运行
- ⚠️ release.yml 缺少 GitHub Release 创建步骤

### 5.2 Nix 状态

- flake.nix 使用 flake-parts (非 flake-utils)
- devShell 提供完整开发环境 (Rust, cago-nextest, cargo-ndk, JDK, Android tools)
- ✅ shfmt 已移除
- ⚠️ 缺少 Android SDK/NDK 配置 (需 `nixpkgs.androidSdk`)
- ⚠️ nixpkgs/fenix/flake-parts 无 commit hash pin

### 5.3 脚本状态

- ✅ quality-gate.nu 已使用 cargo nextest
- ✅ build-android-libs.nu 环境变量 bug 已修复
- ✅ 所有 .sh 已迁移为 .nu

---

## 6. 决策对比选择

### 6.1 torvox-core::Grid 去留

| 方案 | 说明 | 推荐 |
|------|------|------|
| A — 修复 Grid VT 写入 | 需要完整 VT → Grid 写入逻辑 | ❌ 高维护 |
| B — 删除 Grid 拥抱 Ghostty | 删除 ~500 行死代码 | ✅ 当前路径已经是 B |
| C — 现状 (双引擎) | 维护两套系统 | ❌ 技术债 |

**当前路径**: 选项 B — 生产代码使用 `build_cell_instances_from_ghostty`，torvox-core::Grid 主要用于测试和类型检查。

### 6.2 libghostty-vt 供应链风险

| 方案 | 说明 |
|------|------|
| A — Patch 方案 (当前) | git clone + git apply，CI 已验证 |
| B — Vendor 进 repo | 完全控制但体积大 |
| C — 等上游 Android 支持 | 低维护但不可控 |

**当前**: 选项 A，CI 支持。长期目标 C。

---

## 7. 全景问题汇总表

### P0-CRITICAL (无 — 全部已修复)

| # | 问题 | 状态 |
|---|------|------|
| 1 | GhosttyTerminal unsafe Send/Sync | ✅ channel-based 架构 |
| 2 | Grid::get_mut 幽灵脏位 | ✅ 改为先检查再标脏 |
| 3 | GpuContext 硬编码 1080×1920 | ✅ 参数化 |
| 4 | scrollback Vec::remove(0) | ✅ 改为 VecDeque |
| 5 | build-android-libs.nu 环境变量 bug | ✅ 已修复 |

### P1-HIGH

| # | 问题 | 位置 | 说明 |
|---|------|------|------|
| 1 | release.yml 缺少 Release 创建步骤 | release.yml | 不上传到 GitHub Releases |
| 2 | flake.nix 缺少 Android SDK/NDK | flake.nix | nix develop 不能直接 ./gradlew |
| 3 | ANativeWindow 生命周期风险 | surface.rs | 需强制 drop 顺序 |
| 4 | 文档 CRIT-D refs 残留 | 多处 | 部分 `vte` 引用可能仍有残留 |

### P2-MEDIUM

| # | 问题 | 位置 |
|---|------|------|
| 1 | quality-gate.nu 缺少 no_std 检查 | quality-gate.nu |
| 2 | FontPipeline atlas 不回收空间 | font.rs |
| 3 | Grid Vec<Vec<Cell>> 分散分配 | grid.rs (P3.3) |

### P3-LOW

| # | 问题 | 位置 |
|---|------|------|
| 1 | GpuUniforms 硬编码 cell_size | gpu.rs |
| 2 | build_cell_instances 重复代码 | gpu.rs |
| 3 | flake.nix 缺少 commit hash pins | flake.nix |

---

## 8. 安全性

### 8.1 已修复

- ✅ GhosttyTerminal unsafe Send/Sync — 完全消除
- ✅ Grid::get_mut 幽灵脏位 — 先检查再标脏
- ✅ release APK debug 签名 — 添加 release signing config
- ✅ CI boltffi 绑定校验

### 8.2 仍待修复

- ANativeWindow 生命周期风险 (surface.rs) — 中优先级
- quality-gate.nu cargo audit 仅 warn 不 fail — 低优先级

---

## 9. 性能

### 9.1 已优化

- ✅ scrollback O(n) remove(0) → O(1) VecDeque
- ✅ LRU cache O(n log n) sort → O(1) lru::LruCache
- ✅ unicode-width 替换手写 unicode

### 9.2 待优化 (P3.3)

- Grid 分散分配 — flat Vec<Cell> 方案
- render_frame 全量实例构建 — 使用 DirtyMask 过滤
- per-cell FFI 调用 — 批量获取行数据

---

## 10. 修复状态总表

### ✅ 已修复 (19 项)

| # | 问题 | 修复 |
|---|------|------|
| 1 | build-android-libs.nu env var bug | str replace 修复 |
| 2 | GhosttyTerminal unsafe Send/Sync | channel-based |
| 3 | release APK debug 签名 | release signing config |
| 4 | CI 缺少 boltffi 绑定校验 | bindings diff step |
| 5 | Grid::get_mut 幽灵脏位 | 先检查再标脏 |
| 6 | GpuContext 硬编码 1080×1920 | 参数化 |
| 7 | ARCHITECTURE.md bridge 路径 | TorvoxBridge.kt |
| 8 | 双 VT 引擎策略 | GhosttyTerminal 唯一引擎 |
| 9 | CI Zig 步骤无注释 | 添加注释 |
| 10 | libghostty-vt commit hash 记录 | patch + build.rs |
| 11 | scrollback Vec::remove(0) | VecDeque |
| 12 | unicode.rs 覆盖不完整 | unicode-width 0.2 |
| 13 | ROADMAP "etagere" | "guillotiere" |
| 14 | ROADMAP .sh → .nu | 已修正 |
| 15 | quality-gate.nu cargo test | cargo nextest |
| 16 | 渲染线程跨会话共享 | 每会话独立 |
| 17 | unsafe Send/Sync UB | channel-based |
| 18 | LRU cache O(n log n) sort | lru crate O(1) |
| 19 | JNA 5.17.0 → 5.18.1 | 已升级 |

### ⚠️ 部分修复

| # | 问题 | 已修复 | 剩余 |
|---|------|--------|------|
| 1 | render_frame 全量实例构建 | snapshot 支持 dirty_rows | 渲染循环未传入 |
| 2 | release.yml 缺少 Release | signing config | 无 GitHub Release 创建 |

### 🔲 仍待修复

| # | 问题 | 位置 | 优先级 |
|---|------|------|--------|
| 1 | GpuUniforms 硬编码 cell_size [8, 16] | gpu.rs | 低 |
| 2 | FontPipeline atlas 不回收空间 | font.rs | 中 |
| 3 | FontPipeline bitmap 溢出检查 | font.rs | 低 |
| 4 | build_cell_instances 重复代码 | gpu.rs | 低 |
| 5 | ANativeWindow 生命周期风险 | surface.rs | 中 |
| 6 | flake.nix 缺少 Android SDK/NDK | flake.nix | 中 |
| 7 | Grid 分散分配 | grid.rs (P3.3) | 中 |
| 8 | quality-gate.nu no_std 检查 | quality-gate.nu | 低 |
| 9 | quality-gate.nu audit fail | quality-gate.nu | 低 |

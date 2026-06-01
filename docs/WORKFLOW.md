# Torvox 开发工作流 — 规范驱动开发与状态管理

> 本文档定义 Torvox 项目的开发工作流、状态管理和规范驱动开发 (SDD) 实践。
> 基于 GitHub Spec Kit、Alacritty、Warp 等知名项目的最佳实践总结。

---

## 〇、环境要求

**必须使用 nix 管理的开发环境。不要在系统 shell 中直接运行构建命令。**

```bash
# 进入开发环境 (所有工具可用)
nix develop

# 运行命令
nix develop --command cargo build --workspace
nix develop --command cargo nextest run --workspace

# 格式化所有文件
nix fmt

# 运行所有检查
nix flake check --no-build
```

**为什么用 nix？**
- 环境完全可复现，不依赖系统安装
- 所有工具版本锁定，不会因系统更新而破坏
- CI 和本地环境完全一致
- `nix fmt` 格式化所有语言 (Rust, TOML, YAML, Nushell)
- `nix flake check` 运行所有质量检查 (clippy, tests, typos, fmt)

**可用工具 (在 nix develop 中)**:
| 工具 | 用途 |
|------|------|
| `cargo` | Rust 构建 |
| `cargo-nextest` | 测试运行器 |
| `cargo-clippy` | Rust lint |
| `rustfmt` | Rust 格式化 |
| `taplo` | TOML 格式化 |
| `yamlfmt` | YAML 格式化 |
| `typos` | 拼写检查 |
| `ktfmt` | Kotlin 格式化 |
| `ktlint` | Kotlin lint |
| `nushell` | 结构化 shell |
| `gradle` | Android 构建 |
| `kotlin` | Kotlin 编译 |

---

## 一、规范驱动开发 (SDD) 核心原则

### 1.1 核心理念

```
规范是唯一真相来源。代码是规范的衍生物。
```

| 传统开发 | 规范驱动开发 |
|----------|-------------|
| 需求文档 → 代码 → 文档腐烂 | 规范 → 代码 → 规范同步更新 |
| 代码是真相 | 规范是真相 |
| 文档是事后补充 | 规范是事前定义 |
| AI 辅助编码 | AI 按规范生成代码 |

### 1.2 SDD 工作流

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  定义规范    │ ──→ │  生成代码    │ ──→ │  验证对齐    │
│  (Spec)     │     │  (Code)     │     │  (Validate) │
└─────────────┘     └─────────────┘     └─────────────┘
       ↑                                       │
       └───────────── 更新规范 ←──────────────┘
```

**步骤 1: 定义规范**
- 先写规范，再写代码
- 规范包含: 功能描述、验收标准、技术约束、非目标
- 规范存储在代码仓库中 (docs/)

**步骤 2: 实现代码**
- 按规范实现，不添加规范未要求的功能
- 每个实现步骤对应规范中的一个验收标准
- 小步提交，每步验证

**步骤 3: 验证对齐**
- 检查代码是否符合规范
- 检查规范是否需要更新 (发现新问题时)
- 运行测试验证功能正确性

### 1.3 规范文件结构

```
docs/
├── SPECIFICATION.md      # 技术规范 (做什么)
├── ROADMAP.md            # 阶段路线图 (何时做)
├── ARCHITECTURE.md       # 架构设计 (怎么做)
├── ADR/                  # 架构决策记录 (为什么这样做)
│   ├── 001-language-choice.md
│   ├── 002-architecture-pattern.md
│   └── ...
├── AUDIT.md              # 审计报告 (已合并到 AGENTS.md §已知问题与待办)
└── WORKFLOW.md           # 本文件 (工作流)
```

### 1.4 规范维护规则

| 规则 | 说明 |
|------|------|
| **规范先行** | 任何代码变更前，先更新规范 |
| **规范是真相** | 代码与规范冲突时，以规范为准 |
| **规范同步** | 代码变更后，同步更新规范 |
| **规范可验证** | 每个验收标准必须可测试 |
| **规范简洁** | 避免冗余，保持精炼 |

---

## 二、状态管理

### 2.1 项目状态文件

**AGENTS.md §已知问题与待办** — 当前已知问题的唯一真相来源

```markdown
# 已知问题与待办
## ✅ 已修复 (27 项)
## ⚠️ 部分修复
## 🔲 仍待修复
```

**更新时机**:
- 每次代码变更后更新 AGENTS.md §已知问题与待办
- 每次规范变更后更新 AGENTS.md §已知问题与待办
- 每次发现新问题后更新 AGENTS.md §已知问题与待办

### 2.2 状态同步流程

```
代码变更 → 更新 AGENTS.md §已知问题与待办 → 提交 (代码 + 文档)
规范变更 → 更新 AGENTS.md §已知问题与待办 → 提交 (代码 + 文档)
发现 bug → 更新 AGENTS.md §已知问题与待办 → 提交修复
```

**关键原则**: 代码和文档必须在同一个提交中更新，不允许分离。

---

## 三、决策记录 (ADR)

### 3.1 何时需要 ADR

| 场景 | 需要 ADR |
|------|---------|
| 选择技术栈 (语言/库/框架) | ✅ |
| 架构模式变更 | ✅ |
| 依赖版本锁定 | ✅ |
| 安全相关决策 | ✅ |
| 性能 vs 可读性权衡 | ✅ |
| 简单 bug 修复 | ❌ |
| 格式化/命名调整 | ❌ |

### 3.2 ADR 模板

```markdown
# ADR-{编号}: {标题}

## 状态
提议 | 已接受 | 已废弃 | 已取代

## 背景
为什么需要这个决策？

## 决策
我们选择了什么？

## 理由
为什么选择这个方案？考虑了哪些替代方案？

## 后果
这个决策带来什么影响？
```

### 3.3 ADR 编号规则

- 编号: 三位数字，顺序递增 (001, 002, ...)
- 文件名: `docs/ADR/{编号}-{简短描述}.md`
- 不可重编号，不可删除已接受的 ADR

---

## 四、问题管理

### 4.1 问题分类

| 类型 | 标签 | 说明 |
|------|------|------|
| Bug | `bug` | 代码错误 |
| 功能 | `feature` | 新功能 |
| 改进 | `enhancement` | 现有功能改进 |
| 文档 | `documentation` | 文档问题 |
| 性能 | `performance` | 性能问题 |
| 安全 | `security` | 安全问题 |

### 4.2 优先级

| 级别 | 标签 | 说明 |
|------|------|------|
| P0 紧急 | `priority:critical` | 阻塞发布，立即修复 |
| P1 高 | `priority:high` | 重要，尽快修复 |
| P2 中 | `priority:medium` | 正常优先级 |
| P3 低 | `priority:low` | 有空再修 |

### 4.3 问题生命周期

```
发现 → 记录到 AGENTS.md §已知问题与待办 → 分类/定级 → 修复 → 验证 → 关闭
```

---

## 五、提交规范

### 5.1 Conventional Commits

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

| 类型 | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat: implement SGR 256 color support` |
| `fix` | Bug 修复 | `fix: correct scroll_up data corruption` |
| `docs` | 文档更新 | `docs: update SPECIFICATION.md for P1.5` |
| `refactor` | 重构 | `refactor: extract notification helper in session.rs` |
| `test` | 测试 | `test: add proptest for grid scroll invariants` |
| `chore` | 杂务 | `chore: update dependency versions` |

### 5.2 提交规则

| 规则 | 说明 |
|------|------|
| **原子提交** | 每个提交一个逻辑变更 |
| **代码+文档** | 代码变更必须同步更新文档 |
| **测试通过** | 提交前必须通过 clippy + tests |
| **清晰描述** | 描述变更内容和原因 |
| **不混风格** | 不在功能提交中混入格式化变更 |

---

## 六、AI 智能体工作流

### 6.1 会话开始

1. 读取 AGENTS.md (项目上下文)
2. 读取 AGENTS.md §当前状态 + §已知问题与待办
3. 读取 WORKFLOW.md (本文件)
4. 读取 SPECIFICATION.md (规范)
5. 确认当前阶段可达
6. **确保在 nix 环境中运行命令**

### 6.2 实现流程

```
1. 规划: 列出本次会话要完成的具体步骤
2. 类型先行: 先定义类型，再实现行为
3. 小步提交: 每个逻辑步骤提交
4. 验证: 每步 clippy + tests
5. 同步: 代码变更 → 更新 AGENTS.md §已知问题与待办
6. 提交: 代码 + 文档一起提交
```

### 6.3 修改检查清单

```
[ ] 是否需要更新 SPECIFICATION.md?
[ ] 是否需要创建/更新 ADR?
[ ] 是否需要更新 AGENTS.md §已知问题与待办?
[ ] 是否需要更新 ARCHITECTURE.md?
[ ] 是否需要更新 AGENTS.md?
[ ] 是否需要更新 ROADMAP.md?
[ ] 代码变更是否影响 bridge.rs?
[ ] 是否需要重新生成 boltffi 绑定?
```

---

## 七、质量门禁

### 7.1 所有检查 (推荐)

```bash
# 一次性运行所有检查
nix flake check --no-build
```

这会运行:
- clippy (Rust lint)
- fmt (Rust 格式化)
- tests (所有测试)
- typos (拼写检查)
- nixfmt (Nix 格式化)

### 7.2 格式化

```bash
# 格式化所有文件
nix fmt

# 仅检查不修改
nix fmt -- --fail-on-change
```

### 7.3 单独运行

```bash
# Rust 检查
cargo clippy -- -D warnings
cargo test --workspace
cargo fmt --check

# 拼写检查
typos

# Nix 格式化 (所有语言)
nix fmt
```

### 7.4 文档检查

- [ ] AGENTS.md §已知问题与待办 是否反映当前状态?
- [ ] SPECIFICATION.md 是否与代码一致?
- [ ] ARCHITECTURE.md 是否与代码一致?
- [ ] ADR 是否记录了所有重要决策?
- [ ] 版本号是否正确?

---

## 八、与知名项目的对比

| 维度 | Alacritty | Warp | Torvox (目标) |
|------|-----------|------|--------------|
| 规范存储 | 无 (README) | 无 | docs/SPECIFICATION.md ✅ |
| 决策记录 | 无 | 无 | docs/ADR/ ✅ |
| 状态管理 | GitHub Issues | 内部工具 | AGENTS.md §已知问题与待办 ✅ |
| CI/CD | GitHub Actions | 内部 | GitHub Actions ✅ |
| 提交规范 | Conventional | 内部 | Conventional ✅ |
| AI 集成 | 无 | Oz (闭源) | opencode ✅ |

**Torvox 的优势**: 比 Alacritty 和 Warp 更完整的文档体系和决策记录。

---

## 九、编码规范

### 9.1 Shell 脚本

所有 shell 脚本必须使用 **Nushell** (`.nu`)。禁止使用 bash/sh 脚本。

| 规范 | 说明 |
|------|------|
| 首行 | `#!/usr/bin/env nu` |
| 错误处理 | 所有外部命令检查退出码 (`try`/`catch` 或 `| complete`) |
| 管道风格 | 每步一行，`|` 在行尾 |
| 变量命名 | `snake_case`，不缩写 |
| 环境变量 | 用 `$env.VAR = "val"`，不用 `load-env` |
| 外部命令 | 用 `^command` 调用 |

### 9.2 环境管理

所有环境管理必须使用 **Nix**。禁止在系统 shell 中直接运行构建命令。

| 规范 | 说明 |
|------|------|
| 开发 | 始终使用 `nix develop` |
| 构建 | `nix develop --command cargo build` |
| 格式化 | `nix fmt` |
| 检查 | `nix flake check` |

### 9.3 GitHub Actions

| 规范 | 说明 |
|------|------|
| Action 版本 | 所有 action 使用默认分支 (`@main` 或 `@master`)，不用标签或 rev |
| Step `name` | 不设置 — 消息在 run 命令中自解释 |
| 空行 | 不添加无实际作用的空行 |
| `uses` | 紧接 action 路径 + 版本，不换行 |
| `run` 合并 | 相邻 `run:` 步骤合并为一个多行块 |
| `run` 命令 | 多行命令用 `\|` 多行块，避免 `&&` 链过长 |
| `\|\|` 禁止 | 不在任何 run 命令或脚本中使用 `\|\|` |
| Job 命名 | 短横线命名 (`rust-checks`, `no-std-check`) |
| 权限 | 显式声明 `permissions:` |
| `dtolnay/rust-toolchain` | 使用 `@master` + `with: toolchain: "..."` 指定版本 |
| `taiki-e/install-action` | 使用 `@main` + `with: tool: nextest` |

### 9.4 Nix 表达式

| 规范 | 说明 |
|------|------|
| `let in` | 不要使用 — 用 `let body = ...; in body` 或直接 `{ ... }` |
| `rec` | 不要使用 — 用 `self` 参数或拆分定义 |
| `with` | 限制使用 — 仅在顶级 shell 环境引入 |
| 中间变量 | 减少中间变量，不定义只用一次的变量 |
| 命名 | 不缩写单词 (`system` 不写成 `s`) |
| 单行 | 简单映射可单行，复杂逻辑拆多行 |
| flake inputs | 优先用 flake-parts |

### 9.5 通用

| 规范 | 说明 |
|------|------|
| 变量命名 | 不缩写单词 |
| 中间变量 | 尽可能 inline，不定义只用一次的变量 |
| 依赖管理 | shell scripts → Nushell, 环境 → Nix |
| SPA 模型 | 每个主题只有一个权威文档，其他文档交叉引用 |

---

## 十、执行计划

### 10.1 已完成

1. ✅ 本文件 (WORKFLOW.md) — 工作流定义
2. ✅ 更新 AGENTS.md §已知问题与待办 — 反映当前状态
3. ✅ 更新 AGENTS.md — 引用本文件
4. ✅ flake.nix — 添加 formatter/checks
5. ✅ scripts — Nushell (.nu) 格式化

### 10.2 持续执行

1. 每次代码变更同步更新文档
2. 每次发现新问题记录到 AGENTS.md §已知问题与待办
3. 每次重要决策创建 ADR
4. 使用 `nix fmt` 格式化代码
5. 使用 `nix flake check` 验证质量

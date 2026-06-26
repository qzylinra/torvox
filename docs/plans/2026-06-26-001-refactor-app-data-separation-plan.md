---
title: refactor: 将应用数据迁出 Termux 用户文件目录
type: refactor
status: active
date: 2026-06-26
---

# refactor: 将应用数据迁出 Termux 用户文件目录

## Summary

将存放在 `/data/data/com.termux/files/`（`context.filesDir`，与 Termux bootstrap 共享）下的所有应用数据迁移到 `context.getDir()` 创建的隔离目录。迁移后 `files/` 不再被应用读写。添加启动兼容性检查和设置页面手动清除按钮，不依赖任何版本号。

---

## Problem Frame

应用 `applicationId` 为 `com.termux`，导致 `context.filesDir` = `/data/data/com.termux/files/`——正是 Termux bootstrap 用户空间。以下应用数据被错误存放在此：

| 内容 | 当前路径 | 迁移目标 |
|------|---------|---------|
| DataStore 设置 | `files/datastore/settings.preferences_pb` | `getDir("prefs", 0)/settings.preferences_pb` |
| 会话保存 | `files/session_$id.bin` | `getDir("sessions", 0)/session_$id.bin` |
| 日志 | `files/logs/` | `getDir("logs", 0)/` |
| 终端转储 | `files/torvox_terminal.txt` | `cacheDir/torvox_terminal.txt` |
| exec 二进制 | `files/bin/torvox-exec` | `getDir("bin", 0)/torvox-exec` |

旧路径文件在迁移后成为孤立残留，不再被任何代码读写。应用更新时，如果 rkyv 序列化格式变更导致旧会话文件无法反序列化，需有恢复机制。

---

## Requirements

- R1. `/data/data/com.termux/files/` 不能被应用代码读写（不用于保存应用数据）
- R2. 除 bootstrap 安装功能外，应用代码不得读写 `files/` 目录
- R3. `files/` 在应用更新时必须保留且稳定
- R4. 应用更新不要求用户卸载
- R5. 启动时自动检测数据不兼容并删除有问题的设置/临时数据，不碰 `files/`
- R6. 提供手动清除应用数据的选项（仅清除应用数据，不碰 `files/`）
- R7. 不依赖任何版本号做兼容性判断
- R8. 任何情况下应用必须能正常启动和使用

---

## Scope Boundaries

### Deferred to Follow-Up Work

- 旧 `files/` 下的孤立残留文件不做清理（"files 绝对不能动"原则）
- Bootstrap 安装代码不做改动
- 会话保存格式的自描述化
- 日志轮转策略

---

## Context & Research

### 关键文件与当前状态

| 文件 | 问题 | 迁移目标 |
|------|------|---------|
| `android/app/src/main/java/io/torvox/settings/SettingsRepository.kt` | `preferencesDataStore(name = "settings")` 写入 `files/datastore/` | U1 |
| `android/app/src/main/java/io/torvox/runtime/TorvoxRuntime.kt:80` | `sessionSavePath()` 用 `context.filesDir` | U2 |
| `android/app/src/main/java/io/torvox/MainActivity.kt:89` | 日志用 `filesDir/logs/` | U2 |
| `android/app/src/main/java/io/torvox/MainActivity.kt:125` | 终端转储用 `filesDir/torvox_terminal.txt` | U2 |
| `android/app/src/main/java/io/torvox/runtime/LogcatFileWriter.kt:20` | 日志回退用 `context.filesDir` | U2 |
| `android/app/src/main/java/io/torvox/exec/ExecInstaller.kt:12,31` | 二进制用 `filesDir/bin/` | U2 |
| `android/app/src/main/java/io/torvox/ui/TorvoxBridge.kt:621-624` | `restoreSession` 返回 `Unit`，丢弃 FFI `i32` 返回值 | U3 |

### Rust 端路径分析

Rust 侧所有文件 I/O 路径均从 Kotlin 传入：
- `bridge.rs:1077-1086`: `torvox_bridge_restore_session` 接收 `path_ptr`/`path_len` 参数
- `bridge.rs:1055-1064`: `torvox_bridge_save_session` 同上
- `surface.rs:797`: `rkyv::from_bytes` 使用传入路径

Rust 侧不嵌入绝对路径，路径变更后无需修改 Rust 代码。

### 关键约束

- Kotlin 的 `Context.getDir(String, Int)` 返回 `File`（非 `File?`），不返回 null
- `TorvoxBridge.kt:621-624` 的 `restoreSession` wrapper 返回 `Unit`，必须修改为传播错误
- `TorvoxBridge.kt:352-356` JNA 声明 `torvox_bridge_restore_session(...): Int`
- `bridge.rs:1077`: Rust 侧返回 `0` 成功，`-1` 失败

---

## Key Technical Decisions

- **`context.getDir(name, MODE_PRIVATE)` 创建 `/data/data/com.termux/app_<name>/`**：完全脱离 `files/`，Android 保证更新时保留
- **DataStore CorruptionHandler**：仅在 protobuf 结构损坏（`DataCorruptionException`）时重置，不处理瞬时 I/O 错误（DataStore 内部已有重试）
- **修改 `TorvoxBridge.kt` 传播 FFI 错误**：`restoreSession` 检查 i32 返回值，失败时抛出异常。`saveSession` 同理
- **手动清除按钮用协程**：避免 ANR
- **旧 files/ 残留不做清理**：尊重"files 绝对不能动"原则，孤立文件无害

---

## Implementation Units

### U1. 创建 SettingsDataStoreProvider + 更新 SettingsRepository

**Goal:** 将 DataStore 从 `files/datastore/` 迁移到 `getDir("prefs", 0)/settings.preferences_pb`，添加 CorruptionHandler

**Requirements:** R1, R2, R5, R7, R8

**Dependencies:** None

**Files:**
- Create: `android/app/src/main/java/io/torvox/settings/SettingsDataStoreProvider.kt`
- Modify: `android/app/src/main/java/io/torvox/settings/SettingsRepository.kt`

**Approach:**
- `SettingsDataStoreProvider`: `@Singleton` + `@Inject`，注入 `@ApplicationContext`
  - `context.getDir("prefs", Context.MODE_PRIVATE)` 作为存储目录
  - `PreferenceDataStoreFactory.create(produceFile = { File(prefsDir, "settings.preferences_pb") })`
  - `CorruptionHandler<Preferences>`: 仅捕获 `DataCorruptionException`（结构损坏），清空文件后返回 `emptyPreferences()`。不捕获 `IOException`（瞬时 I/O 错误由 DataStore 内部重试处理）
- `SettingsRepository`:
  - 注入 `SettingsDataStoreProvider provider`
  - 将所有 `context.dataStore` 替换为 `provider.dataStore`
  - 移除 `private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "settings")` 及其 import

**Patterns to follow:** `SettingsRepository.kt` 现有注入模式

**Test scenarios:**
- Happy path: 设置读写正确持久化到 `app_prefs/` 下
- Error path: 手动损坏 protobuf 文件，验证 CorruptionHandler 重置为默认值（仅 DataCorruptionException 触发，IOException 不触发）
- Edge case: `app_prefs/` 目录不存在时自动创建

**Verification:** DataStore 文件出现在 `app_prefs/` 下，`files/datastore/` 不再被访问

---

### U2. 迁移所有存储路径

**Goal:** 将 session、日志、终端转储、exec 二进制的路径从 `filesDir` 迁移到 `context.getDir()`

**Requirements:** R1, R2, R3

**Dependencies:** None

**Files:**
- Modify: `android/app/src/main/java/io/torvox/runtime/TorvoxRuntime.kt`
- Modify: `android/app/src/main/java/io/torvox/MainActivity.kt`
- Modify: `android/app/src/main/java/io/torvox/runtime/LogcatFileWriter.kt`
- Modify: `android/app/src/main/java/io/torvox/exec/ExecInstaller.kt`

**Approach:**
- `TorvoxRuntime.sessionSavePath()`:
  ```kotlin
  private fun sessionSavePath(id: Long): String {
      val dir = context.getDir("sessions", Context.MODE_PRIVATE)
      return File(dir, "session_$id.bin").absolutePath
  }
  ```
- `TorvoxRuntime.effectiveHome`: 保持不变——这是只读路径值（shell HOME 回退），不涉及文件写入
- `MainActivity.initFileLogging()`: `File(filesDir, "logs")` → `File(getDir("logs", Context.MODE_PRIVATE), "")`
- `MainActivity.terminalDumpReceiver`: `File(context.filesDir, "torvox_terminal.txt")` → `File(context.cacheDir, "torvox_terminal.txt")`
- `LogcatFileWriter.init()`: `context.filesDir` 回退 → `context.getDir("logs", Context.MODE_PRIVATE)`
- `ExecInstaller`: `File(context.filesDir, "bin")` → `context.getDir("bin", Context.MODE_PRIVATE)`

**Patterns to follow:** `BootstrapDownloader.kt:20` 已使用 `context.cacheDir`

**Test scenarios:**
- Happy path: 会话文件出现在 `app_sessions/`，日志出现在 `app_logs/`，终端转储出现在 `cache/`，二进制出现在 `app_bin/`
- Edge case: 目录不存在时自动创建
- Integration: 完整 session save → restore 流程使用新路径

**Verification:** 无任何代码引用 `context.filesDir`（除 `effectiveHome` 只读路径值外）

---

### U3. 修复桥接层错误传播

**Goal:** 使 `TorvoxBridge.kt` 的 `restoreSession` 和 `saveSession` 正确传播 FFI 返回值错误

**Requirements:** R5, R8

**Dependencies:** None（独立于路径迁移）

**Files:**
- Modify: `android/app/src/main/java/io/torvox/ui/TorvoxBridge.kt`

**Approach:**
- `restoreSession(path: String)`: 检查 `torvox_bridge_restore_session` 返回的 `Int`，非 0 时抛出异常
  ```kotlin
  fun restoreSession(path: String) {
      ensureLib()
      val bytes = path.toByteArray()
      val result = torvox_bridge_restore_session(handle, bytes, bytes.size)
      if (result != 0) throw RuntimeException("Session restore failed (code: $result)")
  }
  ```
- `saveSession(path: String)`: 同理，检查返回值
  ```kotlin
  fun saveSession(path: String): Boolean {
      ensureLib()
      val bytes = path.toByteArray()
      return torvox_bridge_save_session(handle, bytes, bytes.size) == 0
  }
  ```

**Patterns to follow:** JNA 声明已在 `TorvoxBridge.kt:352-356`

**Test scenarios:**
- Happy path: 正常 session 文件 restore 成功
- Error path: 损坏的 rkyv 文件 → `restoreSession` 抛出异常
- Error path: 文件不存在 → `restoreSession` 抛出异常

**Verification:** `restoreSession` 在错误时抛出异常而非静默失败

---

### U4. 启动兼容性检查

**Goal:** 在 `TorvoxRuntime.start()` 中捕获恢复错误，确保应用始终能启动

**Requirements:** R5, R7, R8

**Dependencies:** U2, U3

**Files:**
- Modify: `android/app/src/main/java/io/torvox/runtime/TorvoxRuntime.kt`

**Approach:**
- 在 `start()` 方法中，`restoreSession()` 调用包裹在 try-catch 中：
  - 成功：正常恢复
  - 异常（U3 抛出）：删除会话文件 → 记录日志 → 继续启动
  - `hasSavedSession` 返回 true 但文件不存在（竞态）：跳过恢复，继续启动
- DataStore 的 CorruptionHandler 在 U1 中已添加

**Patterns to follow:** `TorvoxRuntime.kt:311-313` 现有 try-catch 模式

**Test scenarios:**
- Happy path: 正常 session 恢复成功
- Error path: 损坏的 rkyv 文件 → 异常被捕获 → 文件被删除 → 应用正常启动
- Edge case: `hasSavedSession` 返回 true 但文件已不存在 → 优雅跳过
- Edge case: 连续两次恢复失败 → 第二次文件已被删除

**Verification:** 手动放入损坏的 `.bin` 文件到 `app_sessions/`，重启应用应自动删除该文件且正常启动

---

### U5. 添加手动清除应用数据按钮

**Goal:** 设置页面添加「清除应用数据」按钮，清除所有应用数据但不碰 `files/`

**Requirements:** R6, R8

**Dependencies:** U1, U2

**Files:**
- Modify: `android/app/src/main/java/io/torvox/ui/SettingsScreen.kt`

**Approach:**
- 在设置页面添加按钮，位置在「Bootstrap」设置下方或「关于」区域
- 点击后弹出确认对话框，文字说明：「这将清除所有设置、保存的会话、日志、终端转储、缓存数据和 exec 二进制。不影响 Termux 用户文件（home/、usr/）」
- 确认后使用 `viewModelScope.launch(Dispatchers.IO)` 执行：
  - `context.getDir("prefs", 0).deleteRecursively()`
  - `context.getDir("sessions", 0).deleteRecursively()`
  - `context.getDir("logs", 0).deleteRecursively()`
  - `context.cacheDir.deleteRecursively()`
  - `context.getDir("bin", 0).deleteRecursively()`
  - **绝不** `context.filesDir` 及其子目录
- 清除完成后切回主线程，显示"已清除"提示，建议重启应用
- 使用红色按钮以强调破坏性

**Patterns to follow:** 现有 `SettingsScreen.kt` 按钮模式

**Test scenarios:**
- Happy path: 点击按钮 → 确认 → 所有 `app_*` 和 `cache` 被清除 → `files/` 完好
- Edge case: 某个目录不存在，`deleteRecursively` 不抛异常
- Edge case: 删除过程中应用进入后台，不触发 ANR
- Integration: 清除后重启应用，所有设置恢复默认值，bootstrap 功能正常

**Verification:** 清除后 `app_prefs/`, `app_sessions/`, `app_logs/`, `app_bin/` 为空或不存在；`files/usr/`, `files/home/` 完整

---

### U6. 更新测试

**Goal:** 修复因路径变更而断裂的测试

**Requirements:** R1-R8

**Dependencies:** U1, U2, U3

**Files:**
- Modify: `android/app/src/test/java/io/torvox/exec/ExecInstallerTest.kt`
- Modify: `android/app/src/test/java/io/torvox/settings/SettingsRepositoryTest.kt`
- Modify: `android/app/src/test/java/io/torvox/runtime/LogcatFileWriterTest.kt`

**Approach:**
- `ExecInstallerTest.kt:28`: 断言从 `filesDir` 改为 `getDir("bin", 0)`
- `SettingsRepositoryTest.kt:31`: 清理路径从 `filesDir/datastore` 改为 `getDir("prefs", 0)`
- `LogcatFileWriterTest.kt`: 更新 `filesDir/logs/` 断言

**Test scenarios:**
- Happy path: 所有现有测试在新路径下通过

**Verification:** `./gradlew testDebugUnitTest` 全部通过

---

## System-Wide Impact

- **Hilt 依赖:** `SettingsDataStoreProvider` 是新的 `@Singleton`，需确认 Hilt 模块是否需要更新（`@HiltAndroidApp` + `@InstallIn` 通常自动发现 `@Singleton @Inject` 构造函数）
- **Error propagation:** Rust → Kotlin FFI 错误通过 `TorvoxBridge.kt` 修改传播到 `TorvoxRuntime.kt`
- **State lifecycle:** 会话文件路径变化后旧 session 被遗弃。`session_restore` 默认为 false，用户无感知
- **不变量:** Bootstrap 安装代码不变、Rust bridge API 不变、bootstrap 用户数据不受影响

---

## Risks & Dependencies

| Risk | Mitigation |
|------|-----------|
| DataStore 文件搬迁导致设置丢失 | 旧 `files/datastore/` 不会动，只是不再读取。用户需重新配置，bootstrap 和终端功能不受影响 |
| Session 保存路径变化导致自动恢复失效 | 启动时 `hasSavedSession` 检查新路径，旧路径文件被遗弃。`session_restore` 默认为 false，用户不会注意到 |
| SettingsDataStoreProvider Hilt 注入 | `@Singleton @Inject` 在 `@HiltAndroidApp` 应用中自动发现。如有问题可添加 `@Provides` 方法 |
| 旧 files/ 残留文件 | 孤立文件不被任何代码读写，无害。尊重"files 绝对不能动"原则 |
| deleteRecursively 在 UI 线程 | 使用 `viewModelScope.launch(Dispatchers.IO)` 避免 ANR |
| CorruptionHandler 对瞬时 I/O 错误 | 仅捕获 `DataCorruptionException`（结构损坏），不捕获 `IOException`（DataStore 内部重试） |

---

## Documentation / Operational Notes

- AGENTS.md Key Files 表需更新：添加 `SettingsDataStoreProvider.kt`
- 此改动为纯路径重构，不影响用户可见行为（除清除按钮外）
- `effectiveHome`（`TorvoxRuntime.kt:102`）使用 `context.filesDir.absolutePath` 作为 shell HOME 回退值——这是只读路径引用，不涉及文件写入，符合要求

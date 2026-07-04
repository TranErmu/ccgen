# ccgen - 头文件存在性过滤优化

## 项目概述

对 ccgen 工具的 Include 路径解析逻辑进行优化。当前 `include_path.rs` 将用户传入的 `-I` 目录 BFS 展开后全量保留所有子目录，导致不含头文件的空目录也被加入 `compile_commands.json`。本次变更新增头文件存在性过滤，仅保留确实包含 C/C++ 头文件的目录。

### 变更范围

- **仅修改** `src/core/include_path.rs` 一个文件
- 不涉及 CLI 解析、配置文件、源码发现或命令生成等其他模块
- 不新增外部依赖（仅使用 `std::fs`）
- 不增加新的 CLI 参数或配置选项（过滤是默认行为）
- 向后兼容：已有功能不变，仅减少不含头文件的 `-I` 条目

### 核心流水线（变更后）

```
用户传入 -I 目录
    │
    ▼
BFS 递归展开（现有逻辑，不变）
    │
    ▼
--exclude-dir 排除（现有逻辑，不变）
    │
    ▼
头文件过滤 ← 本次新增
    │
    ▼
排序 + 去重（现有逻辑，不变）
    │
    ▼
返回 Vec<PathBuf>
```

---

## 执行方式

使用 `subagent-driven-development` 技能执行。主 Agent 只做统筹调度，不参与代码实现。每个模块由独立的子 Agent 通过 Fork 方式创建并实现。

### 主 Agent 规则

1. **只做统筹**：跟踪整体进度，管理执行顺序，不写任何代码
2. **使用 Fork 创建子 Agent**：每个子 Agent 获得该模块所需的上下文
3. **严格控制上下文**：每个子 Agent 只获得该模块所需的最小上下文
4. **并发控制**：同一时间最多 **4** 个子 Agent 并行执行（本变更仅需 2 个并行）
5. **进度跟踪**：每个子 Agent 完成后，更新任务状态，启动下一批

### 子 Agent 规则

1. 每个子 Agent 实现一个模块，完成后编写 `doc/detail/design-<module-name>.md` 设计文档
2. 子 Agent 之间不共享可变状态，通过函数签名接口交互
3. 必须遵循 tasks.md 中的 checklist，完成后标记 `[x]`
4. 每个子 Agent 同时编写对应的单元测试

---

## 子 Agent 分解

本变更按职责分为 **2 个子 Agent**，可并行执行：

### Sub-agent A：头文件过滤核心算法

| 项 | 说明 |
|---|---|
| 模块名 | `header-filter-algorithm` |
| 修改文件 | `src/core/include_path.rs`（新增函数） |
| 设计文档 | `doc/detail/design-header-filter-algorithm.md` |
| 前置依赖 | 无（纯算法，仅依赖标准库 `Path`/`PathBuf`） |

**需要实现的函数：**

```rust
/// 根据扩展名白名单判断文件是否为 C/C++ 头文件
/// 白名单（不区分大小写）：.h, .hh, .hpp, .hxx, .h++, .ipp, .tcc, .inl
fn is_header_file(entry: &std::fs::DirEntry) -> bool

/// 扫描单层目录，检查是否存在至少一个头文件
/// 遇到权限错误返回 Ok(false)（静默跳过）
/// 其他 I/O 错误同样返回 Ok(false)
fn has_header_files_in_dir(path: &Path) -> Result<bool>

/// 对已展开 + 排除后的目录列表执行头文件过滤
/// 自底向上回溯标记：构建目录树父子关系，从最深目录开始扫描
/// 如果子目录有头文件则父目录也被标记为保留
/// 使用 HashMap<PathBuf, bool> 缓存扫描结果
/// 遇到权限/I/O 错误标记为无头文件，不崩溃
fn filter_by_headers(dirs: &[PathBuf]) -> Vec<PathBuf>
```

**算法描述（自底向上回溯标记）：**

```
输入：已展开 + 排除后的绝对路径列表
输出：仅保留包含头文件的目录

1. 对 dirs 构建目录树，确定父子关系（通过 Path::starts_with 判断）
2. 从最深目录开始（按路径长度降序），自底向上处理每个目录：
   a. 读取目录条目
   b. 检查每个文件扩展名是否在头文件白名单中
   c. 如果找到头文件 → 标记 has_header = true
   d. 如果有子目录已被标记 → 标记 has_header = true
   e. 权限/I/O 错误 → 标记 has_header = false（静默跳过）
3. 返回 has_header = true 的所有目录
```

**测试要求（单元测试）：**
- 头文件白名单匹配：验证所有 8 种扩展名
- 大小写不敏感：`.H`, `.HPP` 等大写也应匹配
- 非头文件排除：`.c`, `.cpp`, `.txt`, `.md` 不应匹配
- 空目录：返回 false
- 无权限目录：静默返回 false，不 panic
- `filter_by_headers` 基本过滤：混合有/无头文件的目录
- 多层嵌套保留：子目录有头文件时父目录保留
- 全空分支：整个分支丢弃

**任务来自 tasks.md：** 任务组 1（1.1 ~ 1.3）+ 任务组 2（2.1 ~ 2.5）

### Sub-agent B：流水线集成与测试

| 项 | 说明 |
|---|---|
| 模块名 | `pipeline-integration` |
| 修改文件 | `src/core/include_path.rs`（修改 `resolve_all`）+ 新建测试目录 |
| 设计文档 | `doc/detail/design-pipeline-integration.md` |
| 前置依赖 | 需参考 Sub-agent A 的函数签名（函数签名由主 Agent 在启动前确认） |

**修改内容：**

```rust
// 修改 resolve_all()，在 collect_dirs 后插入 filter_by_headers()
pub fn resolve_all(config: &CcgenConfig) -> Vec<PathBuf> {
    let mut result = Vec::new();

    for dir in &config.include_dirs {
        let abs_dir = if dir.is_absolute() {
            dir.clone()
        } else {
            config.root.join(dir)
        };
        collect_dirs(&abs_dir, &config.include_exclude_dirs, &mut result);
    }

    // ↓↓↓ 新增：头文件过滤 ↓↓↓
    result = filter_by_headers(&result);

    result.sort();
    result.dedup();
    result
}
```

**执行顺序保证：**
1. BFS 展开（现有 `collect_dirs`）
2. Exclude 排除（现有 `is_excluded_dir`） 
3. **头文件过滤（`filter_by_headers`）** ← 新增
4. 排序去重（现有 `sort()` + `dedup()`）

**测试 fixtures 目录结构**（创建于 `tests/fixtures/`）：

```
tests/fixtures/
├── include/                  # 顶层目录，无头文件
│   ├── sub/                  # 含 a.h
│   │   └── a.h
│   └── empty/                # 空目录
├── lib/
│   └── core/
│       └── internal/         # 含 b.hpp，上层无头文件
│           └── b.hpp
└── docs/                     # 只有 .md 文件，无头文件
    └── readme.md
```

**集成测试要求：**
- 基本过滤：`include/sub/` 有 `a.h` + `include/` 作为父目录 → 两者均保留
- 多层嵌套：`lib/core/internal/` 有 `b.hpp` → `lib/`、`lib/core/`、`lib/core/internal/` 全保留
- Exclude 优先级：`--exclude-dir include/sub` 排除后，即使有头文件也不出现
- 全空目录：`docs/` 全丢，不报错
- CLI 组合：`-I include -I lib` 多目录合并过滤

**任务来自 tasks.md：** 任务组 3（3.1 ~ 3.6）+ 任务组 4（4.1 ~ 4.7）

---

## 现有代码参考

### 当前 `resolve_all()` 实现

```rust
// src/core/include_path.rs - 当前代码
pub fn resolve_all(config: &CcgenConfig) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for dir in &config.include_dirs {
        let abs_dir = if dir.is_absolute() {
            dir.clone()
        } else {
            config.root.join(dir)
        };
        collect_dirs(&abs_dir, &config.include_exclude_dirs, &mut result);
    }
    result.sort();
    result.dedup();
    result
}
```

### 当前关键内部函数

```rust
fn collect_dirs(root: &Path, exclude_dirs: &[PathBuf], result: &mut Vec<PathBuf>) {
    let mut queue = VecDeque::new();
    queue.push_back(root.to_path_buf());
    while let Some(dir) = queue.pop_front() {
        if is_excluded_dir(&dir, exclude_dirs) { continue; }
        result.push(normalize_path(&dir));
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    queue.push_back(entry.path());
                }
            }
        }
    }
}

fn is_excluded_dir(path: &Path, exclude_dirs: &[PathBuf]) -> bool {
    exclude_dirs.iter().any(|ex| path.starts_with(ex) || path == ex)
}

fn normalize_path(path: &Path) -> PathBuf {
    let abs = dunce::simplified(path);
    let s = abs.to_string_lossy().replace('\\', "/");
    PathBuf::from(s)
}
```

### 已有单元测试

当前 `include_path.rs` 文件末尾已有 `#[cfg(test)] mod tests`，包含以下测试：
- `bfs_discovers_all_subdirectories`
- `exclude_dir_omits_directory_and_children`
- `normalize_path_uses_forward_slashes`
- `resolve_all_empty_list_returns_empty`
- `resolve_all_relative_dirs_resolved_against_root`

Sub-agent B 需注意**不删改**这些现有测试，仅新增。

---

## 执行计划

| 阶段 | 子 Agent | 依赖 | 预计产出 |
|------|----------|------|----------|
| **Phase 1**（并行） | **A**: 头文件过滤算法 | 无 | `is_header_file`, `has_header_files_in_dir`, `filter_by_headers` + 单元测试 + 设计文档 |
| | **B**: 流水线集成 + 测试 | 需确认签名 | 修改 `resolve_all` + 测试 fixtures + 集成测试 + 设计文档 |
| **Phase 2** | 验证与修复 | A + B 完成 | `cargo build` + `cargo test` 全部通过 |

---

## 引用 spec 索引

| 模块 | 引用 specs 路径 |
|------|----------------|
| header-filter-algorithm | `specs/include-path/spec.md`（ADDED Requirements 部分） |
| pipeline-integration | `specs/include-path/spec.md`（MODIFIED Requirements + ADDED Requirements） |

---

## 工程规范

1. **错误处理**：权限/I/O 错误静默返回 `Ok(false)`，不 panic，不输出警告
2. **无新增依赖**：仅使用 `std::fs::read_dir`，不引入新的 crate
3. **测试独立性**：使用 `tempfile::TempDir` 创建临时目录（已有依赖）
4. **代码风格**：遵循现有代码风格，与 `collect_dirs`、`normalize_path` 等函数保持一致
5. **设计文档**：每个子 Agent 完成后编写 `doc/detail/design-<module-name>.md`

---

## 完成标准

- [ ] `cargo build` 编译通过，无 warning
- [ ] `cargo test` 全量测试通过（含现有 5 个 + 新增单元测试 + 新增集成测试）
- [ ] `cargo clippy` 无 lint 警告
- [ ] 所有 tasks.md 中的子任务标记为已完成 `[x]`
- [ ] `doc/detail/design-header-filter-algorithm.md` 已生成
- [ ] `doc/detail/design-pipeline-integration.md` 已生成
- [ ] 验收标准：含头文件的目录被保留，无头文件的目录被丢弃，Exclude 优先于过滤

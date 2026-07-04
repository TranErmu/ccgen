# 源码文件发现模块

## 职责

遍历项目目录，发现所有需要加入 `compile_commands.json` 的 C/C++ 源文件。

## 接口

### `discover.rs`

```rust
pub fn find_sources(config: &CcgenConfig) -> Vec<PathBuf>
```

使用 `ignore::WalkBuilder` 遍历 `config.root`，返回匹配的源文件路径列表。

## 实现细节

### 源文件过滤

```rust
fn is_source_file(path: &Path) -> bool
```

只接受 `.c`、`.cpp`、`.cc`、`.cxx` 四种扩展名。头文件（`.h`、`.hpp` 等）被排除。

### Gitignore 支持

```rust
WalkBuilder::new(&config.root)
    .git_ignore(!config.no_gitignore)
    .build()
```

默认启用 `.gitignore` 过滤。`--no-gitignore` 标志可禁用。

### Glob 排除模式

```rust
fn is_excluded(path: &Path, excludes: &[glob::Pattern]) -> bool
```

将 `config.source_excludes` 编译为 `glob::Pattern` 列表，对每个文件路径进行匹配。匹配任意模式即排除。

### 路径规范化

```rust
dunce::simplified(&joined)
```

使用 `dunce` 库将路径简化为标准形式（Windows 下去掉 `\\?\` 前缀），确保所有返回路径为绝对路径。

## 关键实现决策

- **使用 `ignore` crate**：比标准 `walkdir` 多了一层 `.gitignore` 支持，避免重复实现 gitignore 解析逻辑
- **扩展名白名单**：硬编码 `.c/.cpp/.cc/.cxx`，不处理头文件，与 clangd 等工具对 `compile_commands.json` 的预期一致
- **遍历时跳过错误**：`for entry in walk { let entry = match entry { Ok(e) => e, Err(_) => continue } }`，对权限错误等采取静默跳过策略

## 数据结构设计

输出为 `Vec<PathBuf>`，每个元素是归一化的绝对路径。

## 测试策略

单元测试使用 `tests/fixtures/` 目录下的固定文件结构：

```
fixtures/
├── .gitignore             # 包含 *.log
├── src/
│   ├── main.c
│   ├── helper.cc
│   ├── core.cxx
│   ├── utils.cpp
│   ├── header.h           # 应被扩展名过滤排除
│   └── sub/
│       ├── module.c
│       └── temp.log       # 被 gitignore 排除
├── lib/
│   ├── include/
│   │   ├── api.h          # 头文件
│   │   └── detail/
│   │       └── internal.h
│   └── exclude_me/
│       └── excluded.h     # 头文件
└── docs/
    └── readme.md
```

覆盖场景：
- **全量发现**：找到 5 个源文件（main.c, helper.cc, core.cxx, utils.cpp, module.c）
- **排除头文件**：所有 `.h` 文件不被包含
- **gitignore 过滤**：`temp.log` 不被包含
- **`no_gitignore`**：关闭 gitignore 后源文件数量不变（log 文件仍被扩展名过滤）
- **glob 排除**：`**/sub/*` 排除 sub 目录
- **多 glob 排除**：`**/exclude_me/*` + `**/docs/*`
- **绝对路径验证**：所有返回路径为绝对路径
- **空目录**：返回空列表
- **扩展名验证**：所有返回路径确实是 `.c/.cpp/.cc/.cxx`

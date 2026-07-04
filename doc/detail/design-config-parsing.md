# 配置文件解析模块

## 职责

从项目 `.ccgen.toml` 配置文件加载用户配置，转换为 `RawConfig`。

## 接口

### `config.rs`

```rust
pub fn find(root: &Path) -> Option<PathBuf>
```

在指定根目录下查找 `.ccgen.toml` 文件。若存在则返回路径，否则返回 `None`。

```rust
pub fn parse(path: &Path) -> anyhow::Result<RawConfig>
```

读取 TOML 文件，反序列化为 `TomlConfig`，然后转换为 `RawConfig`。

### `TomlConfig` 内部结构

```rust
struct TomlConfig {
    compiler: Option<String>,
    std: Option<String>,
    defines: Option<Vec<String>>,
    undefs: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    exclude_dir: Option<Vec<String>>,
    no_gitignore: Option<bool>,
}
```

**关键设计**：TOML 字段名与 `RawConfig` 字段名不同：
- `include` → `includes`（TOML 的 `include` 映射到 `RawConfig` 的 `includes`）
- `exclude` → `excludes`
- `exclude_dir` → `exclude_dirs`

`into_raw_config()` 使用 `unwrap_or_default()` 处理可选字段，`root` 固定为 `"."`，`output` / `verbose` / `dry_run` 由 CLI 控制、配置文件不设置。

## 关键实现决策

- **字段名映射**：TOML 的 `include` 与 Rust 保留字冲突，使用 Serde 的默认字段名映射（`include` 在结构体中可直接使用）
- **可选反序列化**：所有字段使用 `Option`，允许配置文件省略任意字段
- **文件不存在错误**：使用 `anyhow::Context` 提供文件读取错误的上下文信息；使用 `CcgenError::Config` 包装 TOML 解析错误
- **`find` 轻量探测**：仅检查文件是否存在，不解析内容，用于自动发现配置

## 数据结构设计

无独立外部数据结构，依赖 `types::RawConfig`。

## 测试策略

单元测试覆盖：
- **`find` 存在/不存在**：目录中是否有 `.ccgen.toml`
- **完整配置解析**：所有字段同时设置
- **部分配置解析**：仅设置部分字段，验证其余字段为默认值
- **空配置解析**：空 TOML 全部为默认值
- **非法 TOML**：验证返回错误
- **文件不存在**：验证返回错误

测试工具函数 `write_temp_file` 使用原子计数器生成唯一临时文件名，避免并行测试冲突。

# 参数合并模块

## 职责

将 CLI 参数和配置文件参数合并为一个统一的 `CcgenConfig`。

## 接口

### `merger.rs`

```rust
pub fn merge(cli: RawConfig, file: RawConfig) -> CcgenConfig
```

输入两个 `RawConfig`（分别来自 CLI 和配置文件），输出一个 `CcgenConfig`。

## 合并规则

| 字段 | 优先级规则 |
|------|-----------|
| `root` | CLI 指定则使用 CLI；否则 `current_dir()` |
| `compiler` | CLI 优先，CLI 未指定则用配置文件 |
| `std` | CLI 优先，CLI 未指定则用配置文件 |
| `defines` | 合并：唯一键名，CLI 覆盖同名定义 |
| `undefs` | CLI 非空则完全替换；否则使用配置文件 |
| `include_dirs` | CLI 非空则完全替换；否则使用配置文件 |
| `include_exclude_dirs` | CLI 非空则完全替换；否则使用配置文件 |
| `source_excludes` | CLI 非空则完全替换；否则使用配置文件 |
| `no_gitignore` | CLI || 配置文件（任一为 true 则启用）|
| `output` | CLI 优先，均未指定则默认 `root/compile_commands.json` |
| `verbose` | CLI || 配置文件 |
| `dry_run` | CLI || 配置文件 |

### `root` 特殊处理

```rust
let root = if cli.root.as_os_str() == "." {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
} else {
    cli.root.clone()
};
let root = dunce::canonicalize(&root).unwrap_or(root);
```

将 root 解析为规范化的绝对路径。

### `merge_defines`

```rust
fn merge_defines(cli_defines: &[String], file_defines: &[String]) -> Vec<MacroDef>
```

使用 `HashMap<String, Option<String>>`：
1. 先插入配置文件中的 defines（作为基础）
2. 再插入 CLI 中的 defines（覆盖同名项）
3. 转换为 `Vec<MacroDef>`

### `parse_define`

```rust
fn parse_define(s: &str) -> (String, Option<String>)
```

按第一个 `=` 分割 `NAME=VALUE`，无 `=` 则为 `(NAME, None)`。

## 关键实现决策

- **非覆盖即合并**：布尔值用 `||`（宽松策略），单值用 Option 显式覆盖，列表用 CLI 是否为空决定替换还是保留
- **defines 唯一键合并**：使用 HashMap 保证同一宏名不重复，CLI 优先级更高
- **转换到 `CcgenConfig`**：`RawConfig` 的字符串路径转为 `PathBuf`，宏定义字符串转为 `MacroDef` 结构化表示
- **verbose 日志**：输出每个关键配置项的来源（CLI / config file / default），便于调试

## 数据结构设计

依赖 `types::RawConfig`、`types::CcgenConfig`、`types::MacroDef`、`types::ConfigSource`。

## 测试策略

单元测试覆盖：
- **CLI 覆盖编译器**：验证 `compiler` 的 CLI 优先
- **配置文件兜底**：CLI 未指定时使用配置文件值
- **defines 合并**：CLI 覆盖同名，保留不同名
- **undefs 替换**：CLI 非空时完全替换
- **默认输出路径**：未指定时默认 `compile_commands.json`
- **includes 替换**：CLI 非空时完全替换
- **`parse_define` 单元测试**：有值/无值两种形态
- **verbose 合并**：CLI 或文件任一为 true 则启用

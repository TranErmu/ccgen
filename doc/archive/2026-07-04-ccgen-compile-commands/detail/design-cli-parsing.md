# CLI 参数解析模块

## 职责

从命令行解析用户输入，转换为内部 `RawConfig` 结构体。

## 接口

### `cli.rs`

```rust
pub fn parse_args() -> RawConfig
```

调用了 `CliArgs::parse()`（clap derive），然后通过 `to_raw_config()` 转换为 `RawConfig`。

### `CliArgs` 结构

使用 `clap::Parser` derive 宏定义，共 **14 个参数**：

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `root`（位置参数） | `PathBuf` | `"."` | 项目根目录 |
| `-D` / `--define` | `Vec<String>` | `[]` | 宏定义，支持 `NAME` 或 `NAME=VALUE` |
| `-U` / `--undef` | `Vec<String>` | `[]` | 宏取消定义 |
| `-I` / `--include` | `Vec<String>` | `[]` | Include 搜索路径 |
| `--exclude` | `Vec<String>` | `[]` | 源文件排除 glob 模式 |
| `--exclude-dir` | `Vec<String>` | `[]` | Include 子目录排除 |
| `--compiler` | `Option<String>` | `None` | 覆盖编译器检测 |
| `--std` | `Option<String>` | `None` | 语言标准 |
| `--no-gitignore` | `bool` | `false` | 禁用 .gitignore 过滤 |
| `-o` / `--output` | `Option<PathBuf>` | `None` | 输出文件路径 |
| `--config` | `Option<PathBuf>` | `None` | 配置文件路径 |
| `--dry-run` | `bool` | `false` | 仅输出到 stdout |
| `-v` / `--verbose` | `bool` | `false` | 详细输出 |

重复参数（`-D`、`-U`、`-I`、`--exclude`、`--exclude-dir`）使用 `clap::ArgAction::Append`，支持多次指定。

### `to_raw_config(self) -> RawConfig`

将 `CliArgs` 的字段一对一映射到 `RawConfig`。

## 关键实现决策

- **使用 clap derive**：声明式定义参数，无需手动解析，编译期生成帮助/版本信息
- **CLI 作为唯一入口**：`cli.rs` 不依赖其他模块，`RawConfig` 作为 CLI 与配置文件的统一中间表示
- **位置参数**：`root` 作为唯一的位置参数，允许用户直接指定项目目录

## 数据结构设计

依赖 `types::RawConfig`（见 config-parsing 文档）。

## 测试策略

单元测试覆盖：
- **默认值**：无参数时的所有字段默认值
- **位置参数**：`root` 正确解析
- **`-D` 各种形态**：纯名称、`NAME=VALUE`、含空格值、多次指定
- **`-U`、`-I`、`--exclude`、`--exclude-dir`**：基础赋值与多次追加
- **`--compiler`、`--std`**：可选字符串参数
- **布尔标志**：`--no-gitignore`、`--dry-run`、`--verbose`
- **`-o`、`--config`**：路径参数
- **完整参数组合**：所有参数同时指定
- **短标志组合**：`-D`、`-U`、`-I` 混合使用

所有测试使用 `CliArgs::try_parse_from()` 模拟命令行输入，不依赖真实进程参数。

# 项目初始化模块

## 职责

项目入口，负责初始化全局配置、解析 CLI 参数、加载配置文件、合并配置、触发主流程。

## 接口

### `main.rs`

```rust
fn main() -> anyhow::Result<()>
```

执行顺序：
1. 调用 `CliArgs::parse()` 解析 CLI
2. 判断用户是否指定了 `--config` 路径
   - 指定：直接调用 `config::parse(cfg_path)`
   - 未指定：调用 `config::find(cli_raw.root)` 自动探测 `.ccgen.toml`
3. 若配置文件解析失败，打印警告并使用 `RawConfig::default()` 兜底
4. 调用 `merger::merge(cli_raw, file_raw)` 合并配置
5. 调用 `ccgen::run(merged)` 进入主流程

### `lib.rs`

```rust
pub fn run(config: CcgenConfig) -> anyhow::Result<()>
```

主流程编排：
1. `discover::find_sources(&config)` 发现源文件
2. 若源文件为空，打印警告
3. `include_path::resolve_all(&config)` 解析 include 目录
4. 遍历源文件，对每个调用 `compile_cmd::build_entry()` 构建 `CompileEntry`
5. `dry_run` 模式：调用 `output::print_json()` 输出到 stdout
6. 正常模式：调用 `output::write_to_json()` 写入文件

## 关键实现决策

- **配置加载容错**：配置文件解析失败不会终止程序，改为使用默认配置 `RawConfig::default()` 并打印警告，体现容错设计
- **模块化编排**：`lib.rs::run` 作为纯函数式编排，不处理 CLI/配置文件加载，将初始化与业务逻辑分离
- **`dry_run` 短路**：在 `run` 末尾判断 `config.dry_run`，避免在不需要时执行文件 I/O

## 数据结构设计

无独立数据结构，依赖 `CcgenConfig` 和 `CompileEntry`。

## 测试策略

- 集成测试位于 `tests/integration_test.rs`，覆盖完整端到端流程
- `basic_generation`：验证 5 个源文件全部生成
- `dry_run_does_not_write_file`：验证 dry-run 不写文件
- `empty_sources_warning`：验证空目录生成空 `[]`
- `absolute_paths_forward_slashes`：验证路径规范化

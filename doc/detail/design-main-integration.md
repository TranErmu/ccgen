# 主流程集成模块

## 职责

将所有模块串联为完整的工作流，处理公共错误类型。

## 整体数据流

```
                  ┌─────────────────┐
                  │    main.rs      │
                  │                 │
                  │  CliArgs::parse │
                  │       │         │
                  │       ▼         │
                  │  to_raw_config  │
                  │       │         │
                  │       ├──config │
                  │       │   flag? │
                  │       ▼         │
                  │ config::find 或 │
                  │ config::parse   │
                  │       │         │
                  │       ▼         │
                  │ merger::merge   │
                  │       │         │
                  │       ▼         │
                  │  ccgen::run     │
                  └───────┬─────────┘
                          │
                          ▼
                  ┌─────────────────┐
                  │    lib.rs       │
                  │                 │
                  │discover::       │
                  │  find_sources   │
                  │       │         │
                  │       ▼         │
                  │include_path::   │
                  │  resolve_all    │
                  │       │         │
                  │       ▼         │
                  │  for each src:  │
                  │compile_cmd::    │
                  │  build_entry    │
                  │       │         │
                  │       ▼         │
                  │ output::        │
                  │  write_to_json  │
                  │  or print_json  │
                  └─────────────────┘
```

## 各模块依赖关系

```
main.rs
  ├── cli.rs        — clap derive 解析
  ├── config.rs     — .ccgen.toml 查找与解析
  ├── merger.rs     — 配置合并
  │
lib.rs
  ├── discover.rs   — ignore::WalkBuilder 遍历
  ├── include_path.rs — BFS 展开 include 目录
  ├── compile_cmd.rs  — 编译命令构建
  ├── output.rs     — JSON 序列化与原子写入
  └── types.rs      — 公共数据结构
```

## 错误处理

### `error.rs`

```rust
pub enum CcgenError {
    Io(#[from] std::io::Error),
    Config(String),
}
```

- `Io`：包装标准 I/O 错误，使用 `thiserror` 的 `#[from]` 自动转换
- `Config`：配置文件解析错误的字符串描述

主流程中：
- 配置文件解析失败 → 打印警告 + 使用默认配置（非致命）
- 源文件为空 → 打印警告（非致命，继续生成空 JSON）
- 类型转换使用 `anyhow::Result<?>` 作为统一返回类型

## 关键实现决策

- **配置解析容错**：配置文件损坏或不存在不阻止生成，使用 `RawConfig::default()` 兜底
- **源文件空警告**：空项目输出 `[]`，不报错
- **`lib.rs` 与 `main.rs` 分离**：`lib.rs` 作为纯业务逻辑库，`main.rs` 仅处理 CLI/配置初始化，便于集成测试
- **模块间低耦合**：`lib.rs::run` 接收已合并的 `CcgenConfig`，不关心配置来源

## 测试策略

集成测试位于 `tests/integration_test.rs`，覆盖完整端到端流程：
- `basic_generation`：完整端到端生成
- `macro_defines`：宏定义正确传递到所有条目
- `include_paths`：include 目录递归展开
- `exclude_source`：glob 排除模式生效
- `exclude_include_dir`：include 子目录排除
- `compiler_override`：编译器覆盖
- `language_standard`：语言标准参数
- `no_gitignore`：关闭 gitignore 过滤
- `dry_run_does_not_write_file`：dry-run 行为
- `absolute_paths_forward_slashes`：路径格式一致
- `atomic_write_cleans_temp`：原子写入清理
- `merge_priority`：CLI 优先级验证
- `empty_sources_warning`：空源文件生成空数组

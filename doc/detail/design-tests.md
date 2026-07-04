# 测试策略文档

## 测试架构

项目采用双层测试架构：**单元测试**（模块内 `#[cfg(test)] mod tests`）+ **集成测试**（`tests/integration_test.rs`）。

### 依赖

- `tempfile`（dev-dependency）：提供临时目录创建
- `serde_json::Value`（集成测试）：灵活验证 JSON 结构

## 单元测试覆盖

### `cli.rs`（14 个测试用例）

| 测试 | 覆盖内容 |
|------|---------|
| `test_default_values` | 所有字段的默认值 |
| `test_root_positional_arg` | 位置参数 `root` |
| `test_define_name_only` | `-D NAME` |
| `test_define_name_value` | `-D NAME=VALUE` |
| `test_define_spaced_value` | `-D "NAME=spaced value"` |
| `test_define_multiple` | 多次 `-D` |
| `test_undefines` | 多次 `-U` |
| `test_includes` | 多次 `-I` |
| `test_exclude_and_exclude_dir` | `--exclude` + `--exclude-dir` |
| `test_compiler_and_std` | 可选字符串参数 |
| `test_boolean_flags` | `--no-gitignore` + `--dry-run` + `--verbose` |
| `test_output_and_config` | `-o` + `--config` |
| `test_to_raw_config_complete` | 完整参数组合 |
| `test_short_flag_combos` | 短标志 `-D` `-U` `-I` 混用 |

使用 `CliArgs::try_parse_from` 模拟命令行参数，不依赖真实环境。

### `config.rs`（6 个测试用例）

| 测试 | 覆盖内容 |
|------|---------|
| `test_find_not_found` | 目录不含 `.ccgen.toml` |
| `test_find_found` | 目录含 `.ccgen.toml` |
| `test_parse_complete_config` | 所有字段完整配置 |
| `test_parse_partial_config` | 仅部分字段 |
| `test_parse_empty_config` | 空文件 |
| `test_parse_malformed_toml` | 非法 TOML |
| `test_parse_file_not_found` | 文件不存在 |

使用 `write_temp_file` + 原子计数器隔离临时文件。

### `discover.rs`（9 个测试用例）

| 测试 | 覆盖内容 |
|------|---------|
| `finds_all_source_files` | 5 个源文件全部发现 |
| `excludes_headers` | 4 个头文件被排除 |
| `gitignore_filters_logs` | `*.log` 被 gitignore 过滤 |
| `no_gitignore_disables_gitignore` | 关闭 gitignore |
| `exclude_glob_filters` | `**/sub/*` 排除 |
| `exclude_glob_filters_subtree` | 多 glob 模式 |
| `all_paths_are_absolute` | 路径归一化为绝对路径 |
| `empty_dir_returns_empty` | 空目录 |
| `all_paths_are_source_extensions` | 扩展名验证 |

使用 `tests/fixtures/` 目录下的固定测试文件结构。

### `include_path.rs`（5 个测试用例）

| 测试 | 覆盖内容 |
|------|---------|
| `bfs_discovers_all_subdirectories` | BFS 递归展开 |
| `exclude_dir_omits_directory_and_children` | 排除子目录 |
| `normalize_path_uses_forward_slashes` | `\` → `/` 替换 |
| `resolve_all_empty_list_returns_empty` | 空输入 |
| `resolve_all_relative_dirs_resolved_against_root` | 相对路径 |

使用 `tempfile::TempDir` 创建真实目录结构。

### `merger.rs`（7 个测试用例）

| 测试 | 覆盖内容 |
|------|---------|
| `cli_overrides_file` | compiler 覆盖 |
| `file_used_when_cli_not_set` | 配置文件兜底 |
| `cli_defines_override_file_defines` | defines 合并 |
| `cli_undefs_override_file_undefs` | undefs 替换 |
| `default_output_path` | 默认输出路径 |
| `cli_includes_override_file` | includes 替换 |
| `parse_define_with_value` / `without_value` | 宏解析 |
| `verbose_flag_from_cli` | verbose 合并 |

### `compile_cmd.rs`（12 个测试用例）

| 测试 | 覆盖内容 |
|------|---------|
| `infer_compiler_c` / `_cpp` / `_cc` / `_cxx` | 编译器推断 |
| `infer_compiler_header_defaults_to_gcc` | 头文件默认 |
| `infer_compiler_no_extension_defaults_to_gcc` | 无扩展名默认 |
| `build_entry_full_arguments` | 完整参数 |
| `build_entry_compiler_override` | 编译器覆盖 |
| `build_entry_cpp_language` | C++ 语言标志 |
| `build_entry_no_defines_undefs_std` | 最小参数 |
| `build_entry_with_include_dirs` | 多 include |
| `build_entry_d_define_with_value/without_value` | `-D` 两种形态 |
| `build_entry_u_undef` | `-U` 参数 |
| `build_entry_std` | `-std` 参数 |

使用辅助宏 `assert_args!` 简化断言。

### `output.rs`（6 个测试用例）

| 测试 | 覆盖内容 |
|------|---------|
| `json_format_correct` | JSON 字段正确 |
| `atomic_write_cleans_temp` | 临时文件清理 |
| `dry_run` | 不 panic |
| `auto_create_parent_dir` | 嵌套目录自动创建 |
| `empty_entries` | 空数组 |
| `print_json_does_not_panic` | 不 panic |

## 集成测试覆盖

文件：`tests/integration_test.rs`，共 **13 个测试用例**。

| 测试 | 覆盖内容 |
|------|---------|
| `basic_generation` | 端到端生成 5 个条目 |
| `macro_defines` | 宏定义在所有条目中出现 |
| `include_paths` | include 目录递归展开 |
| `exclude_source` | glob 排除 |
| `exclude_include_dir` | include 子目录排除 |
| `compiler_override` | 编译器覆盖 |
| `language_standard` | 语言标准 |
| `no_gitignore` | 关闭 gitignore |
| `dry_run_does_not_write_file` | dry-run |
| `absolute_paths_forward_slashes` | 路径格式 |
| `atomic_write_cleans_temp` | 原子写入 |
| `merge_priority` | CLI 合并优先级 |
| `empty_sources_warning` | 空源文件生成空数组 |

## 测试运行

```bash
cargo test                    # 运行全部测试（单元 + 集成）
cargo test --lib              # 仅运行单元测试
cargo test --test integration_test  # 仅运行集成测试
cargo test <test_name>        # 运行单个测试
```

## CI 注意事项

- 所有测试使用临时目录，不依赖固定路径
- 文件操作测试使用 `tempfile::TempDir`，测试结束后自动清理
- `cli.rs` 测试完全在内存中解析，无文件 I/O
- 集成测试需要 `tests/fixtures/` 目录存在

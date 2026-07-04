# 输出生成模块

## 职责

将编译条目列表序列化为 `compile_commands.json` 文件，或打印到 stdout。

## 接口

### `output.rs`

```rust
pub fn write_to_json(entries: &[CompileEntry], path: &Path) -> anyhow::Result<()>
```

将编译条目写入 JSON 文件。使用**原子写入**策略：
1. 创建父目录（如不存在）
2. 写入临时文件 `<path>.tmp`
3. 调用 `file.sync_all()` 确保数据落盘
4. 调用 `fs::rename()` 原子替换目标文件

```rust
pub fn print_json(entries: &[CompileEntry]) -> anyhow::Result<()>
```

将编译条目格式化为漂亮 JSON 并打印到 stdout。

## 实现细节

### 原子写入流程

```rust
let tmp_path = path.with_extension("tmp");
let file = fs::File::create(&tmp_path)?;
serde_json::to_writer_pretty(&file, entries)?;
file.sync_all()?;
fs::rename(&tmp_path, path)?;
```

使用临时文件 + `rename` 确保：
- 写入中断不会留下半成品文件
- 读取方始终看到完整内容
- `rename` 在 POSIX 和 Windows 上均为原子操作

### JSON 格式

输出为符合 `compile_commands.json` 规范的 JSON 数组：

```json
[
  {
    "directory": "/project",
    "file": "/project/src/main.c",
    "arguments": ["gcc", "-x", "c", "-c", "src/main.c"]
  }
]
```

## 关键实现决策

- **原子写入**：防止写入过程中进程崩溃导致输出文件损坏
- **`sync_all()`**：确保数据从操作系统缓冲区写入磁盘，防止意外断电
- **`serde_json::to_writer_pretty`**：输出易读的格式化的 JSON
- **父目录自动创建**：用户指定的输出路径可能包含不存在的目录层次

## 数据结构设计

依赖 `types::CompileEntry`（实现了 `Serialize`）。

## 测试策略

单元测试覆盖：
- **JSON 格式正确性**：验证 `directory`、`file`、`arguments` 字段
- **原子写入清理**：验证 `.tmp` 临时文件在完成后被清理
- **dry_run 字符串**：验证 `print_json` 生成的 JSON 字符串
- **自动创建父目录**：嵌套路径自动创建
- **空条目**：空数组输出 `[]`
- **`print_json` 不 panic**：验证函数正确执行

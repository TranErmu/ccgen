# Include 路径处理模块

## 职责

将用户指定的 include 目录递归展开为平坦的目录列表，用于构建编译命令的 `-I` 参数。

## 接口

### `include_path.rs`

```rust
pub fn resolve_all(config: &CcgenConfig) -> Vec<PathBuf>
```

输入 `CcgenConfig`，输出排序去重后的目录路径列表。

## 实现细节

### BFS 递归展开

```rust
fn collect_dirs(root: &Path, exclude_dirs: &[PathBuf], result: &mut Vec<PathBuf>)
```

使用 `VecDeque` 作为 BFS 队列，对每个目录：
1. 检查是否被排除（`is_excluded_dir`）
2. 若未排除，将归一化路径加入结果
3. 读取子目录并加入队列继续遍历

### 目录排除

```rust
fn is_excluded_dir(path: &Path, exclude_dirs: &[PathBuf]) -> bool
```

使用 `path.starts_with(exclude)` 或 `path == ex` 判断。若父目录被排除，子目录也会被 `collect_dirs` 的 BFS 自然跳过。

### 路径归一化

```rust
fn normalize_path(path: &Path) -> PathBuf
```

1. `dunce::simplified(path)` 简化 Windows 路径
2. 将所有 `\` 替换为 `/`，确保输出一致的前向斜杠格式

### 总体流程

```rust
pub fn resolve_all(config: &CcgenConfig) -> Vec<PathBuf>
```

1. 对 `config.include_dirs` 中每个目录：
   - 绝对路径直接使用
   - 相对路径相对于 `config.root` 解析
2. 调用 `collect_dirs` 递归展开
3. 最终 `sort()` + `dedup()` 去重

## 关键实现决策

- **BFS 而非递归**：防止深层目录结构导致栈溢出
- **前向斜杠强制替换**：Windows 路径 `\` 替换为 `/`，与 `compile_commands.json` 的跨平台兼容性要求一致
- **排除检查前置**：在加入 BFS 队列前检查排除，被排除目录的子目录不会被遍历
- **排序去重**：多个 include 目录可能重复或重叠，`sort + dedup` 确保输出稳定

## 数据结构设计

- 输入输出均为 `Vec<PathBuf>`
- BFS 使用 `std::collections::VecDeque<PathBuf>`

## 测试策略

单元测试使用 `tempfile::TempDir` 创建临时目录结构：

```
dir_a/
├── sub_1/
│   └── deep/
└── sub_2/
dir_b/
dir_c/
```

覆盖场景：
- **BFS 递归发现**：`dir_a` 展开为 4 个目录（包含自身）
- **排除子目录**：排除 `sub_1`，其子 `deep` 也被跳过
- **路径归一化**：验证 `\` 全部替换为 `/`
- **空 include 列表**：返回空
- **相对路径解析**：相对目录相对 root 解析后返回绝对路径

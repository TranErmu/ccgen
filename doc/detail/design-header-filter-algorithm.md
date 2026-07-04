# 设计文档：头文件过滤核心算法

## 模块概述

为 `include_path.rs` 新增三个内部函数，实现基于扩展名白名单的头文件存在性检测与目录过滤。

## 函数设计

### `is_header_file(entry: &DirEntry) -> bool`

- 检查 `DirEntry` 是否为文件（非目录）
- 获取文件扩展名，转小写后匹配白名单
- 白名单：`h`, `hh`, `hpp`, `hxx`, `h++`, `ipp`, `tcc`, `inl`

### `has_header_files_in_dir(path: &Path) -> io::Result<bool>`

- 调用 `std::fs::read_dir` 扫描单层目录
- 对每个条目调用 `is_header_file` 检测
- 遇到 I/O 错误（权限不足等）返回 `Ok(false)`，静默跳过

### `filter_by_headers(dirs: &[PathBuf]) -> Vec<PathBuf>`

- 对输入目录按路径深度降序排序（最深优先）
- 使用 `HashMap<PathBuf, bool>` 缓存每个目录的扫描结果
- 自底向上回溯：如果目录自身有头文件，或任意子目录（通过 `Path::starts_with` 判断）已被标记为有头文件，则该目录保留
- 返回所有被标记为有头文件的目录

## 算法复杂度

- 时间：O(N × M)，N 为目录数，M 为平均每目录文件数（缓存避免重复扫描）
- 空间：O(N)，缓存存储

## 错误处理

- 所有 I/O 错误静默跳过，不 panic，不输出警告
- 权限不足的目录被标记为无头文件

## 测试覆盖

- 9 个单元测试：白名单匹配、大小写不敏感、非头文件排除、目录类型排除、空目录、有头文件目录、基本过滤、多层嵌套、全空分支

# 设计文档：流水线集成

## 模块概述

将头文件过滤步骤集成到 `resolve_all()` 的路径处理流水线中，并创建测试 fixtures 和集成测试。

## 流水线变更

```
BFS 展开 → Exclude 排除 → 头文件过滤 ← 新增 → 排序去重
```

修改 `resolve_all()` 函数，在 `collect_dirs` 循环后、`sort()`/`dedup()` 前插入 `filter_by_headers()` 调用。

## 测试目录结构

```
tests/fixtures/include_filter/
├── has_headers/
│   ├── sub/
│   │   └── a.h
│   └── empty/
├── lib/
│   └── core/
│       └── internal/
│           └── b.hpp
└── no_headers/
    └── readme.md
```

## 集成测试

| 测试名 | 场景 |
|--------|------|
| `include_filter_basic` | 基本过滤：有头文件的目录保留，空目录丢弃 |
| `include_filter_nested` | 多层嵌套：子目录有头文件，父目录链全部保留 |
| `include_filter_no_headers_discarded` | 全空目录：无头文件的目录完全丢弃 |
| `include_filter_exclude_priority` | Exclude 优先级：被排除的目录不参与过滤 |
| `include_filter_multiple_dirs` | 多目录合并：`-I a -I b` 各自过滤后合并 |

## 向后兼容

- 已有功能不变，仅减少不含头文件的 `-I` 条目
- 无新增 CLI 参数或配置选项
- 无新增外部依赖

# ccgen

C/C++ compile_commands.json 生成器

## 简介

ccgen 是一个 Rust CLI 工具，用于为无法立即编译的 C/C++ 项目生成 `compile_commands.json`，使 clangd 等工具能正常提供代码补全、跳转和诊断。

工具接收用户提供的宏定义、Include 路径和编译器选项，结合项目源码文件列表，生成符合 LLVM JSON Compilation Database 规范的编译命令数据库。

## 安装

```bash
cargo install --path .
```

## 使用方法

### 基本用法

```bash
# 在项目目录下生成 compile_commands.json
ccgen

# 指定项目根目录
ccgen /path/to/project
```

### 宏定义

```bash
# 空值宏
ccgen -D DEBUG

# 带值宏
ccgen -D VERSION=2

# 带空格值的宏
ccgen -D NAME="spaced value"

# 多个宏
ccgen -D DEBUG -D VERSION=2 -D EXTRA
```

### Include 路径

```bash
# 递归发现 Include 目录的所有子目录
ccgen -I /usr/include -I /usr/local/include

# 排除特定 Include 子目录
ccgen -I ./lib --exclude-dir test
```

### 源码排除

```bash
# 排除 glob 匹配的源文件
ccgen --exclude "*.test.*" --exclude "build/*"
```

### 配置文件

创建 `.ccgen.toml` 在项目根目录：

```toml
compiler = "gcc"
std = "c11,c++17"
defines = ["DEBUG", "VERSION=2"]
undefs = ["OLD"]
include = ["src", "include"]
exclude = ["test", "build"]
exclude_dir = [".git", "target"]
no_gitignore = false
```

CLI 参数优先级高于配置文件。

### 输出控制

```bash
# 指定输出路径
ccgen -o /tmp/compile_commands.json

# 仅输出到 stdout（不写文件）
ccgen --dry-run

# 启用详细输出
ccgen -v
```

## 完整选项

| 选项 | 说明 |
|------|------|
| `[ROOT]` | 项目根目录，默认当前目录 |
| `-D, --define <KEY=VALUE>` | 宏定义，可重复 |
| `-U, --undef <NAME>` | 取消宏定义，可重复 |
| `-I, --include <PATH>` | Include 路径，可重复 |
| `--exclude <PATTERN>` | 源码排除 glob，可重复 |
| `--exclude-dir <DIR>` | Include 子目录排除，可重复 |
| `--compiler <NAME>` | 覆盖编译器 |
| `--std <STD>` | 语言标准（如 c11, c17, c++20） |
| `--no-gitignore` | 禁用 .gitignore 过滤 |
| `-o, --output <PATH>` | 输出路径 |
| `--config <PATH>` | 配置文件路径 |
| `--dry-run` | 仅输出到 stdout |
| `-v, --verbose` | 详细输出 |

## 输出格式

生成的 JSON 符合 LLVM Compilation Database 规范：

```json
[
  {
    "directory": "/path/to/project",
    "file": "/path/to/project/src/main.c",
    "arguments": ["gcc", "-x", "c", "-c", "/path/to/project/src/main.c"]
  }
]
```

## 项目结构

```
ccgen/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs               # 库入口 + run()
│   ├── types.rs             # 共享类型 (RawConfig, CcgenConfig, CompileEntry)
│   ├── error.rs             # 错误类型
│   ├── input/               # 输入解析
│   │   ├── mod.rs
│   │   ├── cli.rs           # CLI 参数解析 (clap)
│   │   └── config.rs        # .ccgen.toml 解析
│   ├── core/                # 处理流水线
│   │   ├── mod.rs
│   │   ├── merger.rs        # 参数合并
│   │   ├── discover.rs      # 源码文件发现
│   │   ├── include_path.rs  # Include 路径处理
│   │   └── compile_cmd.rs   # 编译命令构建
│   └── output/              # 输出
│       ├── mod.rs
│       └── writer.rs        # JSON 输出 + 原子写入
├── tests/
│   ├── integration_test.rs
│   └── fixtures/
└── doc/
    └── detail/
        └── design-*.md      # 模块设计文档
```

## 开发

```bash
# 构建
cargo build

# 测试
cargo test

# 格式化
cargo fmt

# Clippy
cargo clippy
```

## 许可证

MIT

---
name: ccgen
description: >
  C/C++ compile_commands.json 生成器。为无法立即编译的 C/C++ 项目生成
  符合 LLVM Compilation Database 规范的 compile_commands.json，使 clangd
  等工具能正常提供代码补全、跳转和诊断。支持宏定义注入、Include 路径递归
  发现、源码排除、.gitignore 过滤和配置文件。当用户提到 compile_commands.json、
  clangd 配置、C/C++ 项目代码补全、或需要为没有构建系统的项目生成编译数据库时触发。
compatibility: Requires ccgen binary in PATH or project root
metadata:
  author: Havie
  version: "0.1.0"
---

# ccgen - C/C++ compile_commands.json 生成器

为 C/C++ 项目生成 compile_commands.json，使 clangd 能正常工作。

## Instructions

### 判断是否安装

```bash
# windows
command -v ccgen >/dev/null 2>&1 || test -x ./ccgen.exe >/dev/null 2>&1 || echo "ccgen未安装"
# or linux
command -v ccgen >/dev/null 2>&1 || test -x ./ccgen >/dev/null 2>&1 || echo "ccgen未安装"
```

如果未安装则退出执行SKILL

### 基本用法

```bash
# 在项目目录下生成 compile_commands.json
ccgen .

# 指定项目路径
ccgen /path/to/project
```

### 常用场景

**场景 1：为项目生成 compile_commands.json**

```bash
ccgen /path/to/project
```

**场景 2：注入宏定义**

```bash
# 单个宏
ccgen . -D DEBUG

# 带值宏
ccgen . -D VERSION=2

# 多个宏
ccgen . -D DEBUG -D VERSION=2 -D EXTRA
```

**场景 3：Include 路径发现**

```bash
# 递归发现 Include 目录
ccgen . -I /usr/include -I /usr/local/include

# 排除特定子目录
ccgen . -I ./lib --exclude-dir test
```

**场景 4：使用配置文件**

在项目根目录创建 `.ccgen.toml`：

```toml
compiler = "gcc"
std = "c11"
defines = ["DEBUG", "VERSION=2"]
include = ["src", "include"]
exclude_dir = [".git", "target"]
```

然后直接运行：

```bash
ccgen .
```

**场景 5：dry-run 预览**

```bash
ccgen . --dry-run --verbose
```

**场景 6：指定配置文件**

```bash
ccgen . --config /path/to/custom-config.toml
```

### 参数优先级

CLI 参数 > `.ccgen.toml` > 默认行为

### 输出格式

生成的 JSON 格式：

```json
[
  {
    "directory": "/path/to/project",
    "file": "/path/to/project/src/main.c",
    "arguments": ["gcc", "-x", "c", "-c", "/path/to/project/src/main.c"]
  }
]
```

### 完整选项

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

## Examples

**输入：**

```bash
ccgen /path/to/project -D DEBUG -I ./include --std c11
```

**输出：**

```json
[
  {
    "directory": "/path/to/project",
    "file": "/path/to/project/src/main.c",
    "arguments": [
      "gcc", "-x", "c", "-c", "/path/to/project/src/main.c",
      "-I", "/path/to/project/include",
      "-D", "DEBUG",
      "-std=c11"
    ]
  }
]
```

## Common edge cases

- 无源文件 → 输出警告但正常退出（exit 0）
- Include 路径不存在 → 跳过该路径，继续处理其他路径
- .ccgen.toml 格式错误 → 返回人类可读错误信息
- Windows 路径 → 自动转换为正斜杠 `/` + 使用 dunce 去除 `\\?\` 前缀
- 输出目录不存在 → 自动递归创建
- 写入中断 → 使用临时文件 + rename 原子写入，避免输出文件损坏

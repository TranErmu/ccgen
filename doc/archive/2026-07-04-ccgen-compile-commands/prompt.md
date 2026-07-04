# ccgen - compile_commands.json 生成器

## 项目概述

ccgen 是一个 Rust CLI 工具，用于为无法立即编译的 C/C++ 项目生成 `compile_commands.json`。
工具接收用户提供的宏定义、Include 路径和编译器选项，结合项目源码文件列表，生成符合 LLVM JSON Compilation Database 规范的编译命令数据库。

### 核心处理流程

```
CLI 参数 ─┐
          ├─→ 参数合并 → 源码发现 → 编译命令生成 → JSON 输出
配置文件 ─┘
```

### 优先级规则
CLI 参数 > `.ccgen.toml` > 默认行为

---

## 执行方式

使用 `subagent-driven-development` 技能执行。主 Agent 只做统筹调度，不参与代码实现。
每个模块由独立的子 Agent 通过 Fork 方式创建并实现。

### 主 Agent 规则

1. **只做统筹**：跟踪整体进度，管理执行顺序，不写任何代码
2. **使用 subagent-driven-development 技能**：通过 Fork 创建子 Agent
3. **严格控制上下文**：每个子 Agent 只获得该模块所需的上下文，不加载全文
4. **并发控制**：同一时间最多 4 个子 Agent 并行执行
5. **进度跟踪**：每个子 Agent 完成后，更新任务状态，启动下一批

### 子 Agent 规则

1. 每个子 Agent 实现一个模块，完成后编写 `doc/detail/design-<module-name>.md` 设计文档
2. 子 Agent 之间不共享可变状态，通过已确定的接口文件（types.rs）交互
3. 必须遵循 tasks.md 中的 checklist，完成后标记 `[x]`
4. 每个子 Agent 也需要在模块目录编写单元测试

---

## 模块结构

```
ccgen/
├── Cargo.toml
├── .ccgen.toml                          # 示例配置文件（测试用）
├── src/
│   ├── main.rs                          # 入口：组装完整处理流程
│   ├── cli.rs                           # CLI 参数定义（clap derive）
│   ├── config.rs                        # 配置文件 (.ccgen.toml) 解析
│   ├── merger.rs                        # 合并 CLI 和配置文件的参数
│   ├── discover.rs                      # 源码文件发现和过滤
│   ├── include_path.rs                  # Include 路径递归发现和排除
│   ├── compile_cmd.rs                   # 为每个源文件构建编译命令
│   ├── output.rs                        # JSON 生成和文件写入
│   ├── types.rs                         # 核心数据结构定义
│   └── error.rs                         # 错误类型定义
├── tests/
│   └── integration_test.rs              # 集成测试
└── doc/
    └── detail/
        ├── design-setup.md
        ├── design-cli-parsing.md
        ├── design-config-parsing.md
        ├── design-source-discovery.md
        ├── design-include-path.md
        ├── design-merger.md
        ├── design-compile-cmd.md
        ├── design-output.md
        ├── design-main-integration.md
        └── design-tests.md
```

---

## 核心数据结构（types.rs）

每个子 Agent 都依赖此接口，在 Phase 0 中先行定义：

```rust
// 原始配置（来自 CLI 或配置文件）
pub struct RawConfig {
    pub compiler: Option<String>,
    pub std: Option<String>,
    pub defines: Vec<String>,
    pub undefs: Vec<String>,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub no_gitignore: bool,
    pub root: PathBuf,
    pub output: Option<PathBuf>,
}

// 合并后的最终配置
pub struct CcgenConfig {
    pub root: PathBuf,
    pub compiler: Option<String>,
    pub std: Option<String>,
    pub defines: Vec<MacroDef>,
    pub undefs: Vec<String>,
    pub include_dirs: Vec<PathBuf>,
    pub include_exclude_dirs: Vec<PathBuf>,
    pub source_excludes: Vec<String>,
    pub no_gitignore: bool,
    pub output: PathBuf,
    pub verbose: bool,
    pub dry_run: bool,
}

pub struct MacroDef {
    pub name: String,
    pub value: Option<String>,
}

pub struct CompileEntry {
    pub directory: PathBuf,
    pub file: PathBuf,
    pub arguments: Vec<String>,
}

// 来源标记，用于 verbose 日志
pub enum ConfigSource {
    Cli,
    ConfigFile,
}
```

---

## 执行阶段

### Phase 0：项目初始化（1 个子 Agent）

**必须最先执行，无并行依赖。**

| 项 | 说明 |
|---|---|
| 模块名 | `setup` |
| 子 Agent 数 | 1 |
| 输出文件 | `Cargo.toml`, `src/lib.rs`, `src/types.rs`, `src/error.rs`, `src/main.rs`（骨架）|
| 设计文档 | `doc/detail/design-setup.md` |
| 引用 spec | `cli-parsing/spec.md`, `config-parsing/spec.md` 等中涉及的 error 场景 |

**子 Agent 任务：**
1. 运行 `cargo init --name ccgen`，设置 `edition = "2021"`
2. 配置 `Cargo.toml` 依赖：
   - `clap` (feature: `derive`)
   - `serde` (feature: `derive`)
   - `toml`
   - `ignore`
   - `walkdir`
   - `serde_json`
   - `anyhow`
   - `glob`
   - `dunce`
3. 创建 `src/lib.rs` 作为库入口，声明所有模块（仅骨架，函数体留空）
4. 实现 `src/types.rs`：定义上述所有数据结构（RawConfig, CcgenConfig, MacroDef, CompileEntry, ConfigSource）
5. 实现 `src/error.rs`：定义 `CcgenError` 枚举及 `From` trait 实现
6. 初始化 `src/main.rs` 为仅有 `fn main() {}` 的空壳
7. 创建测试项目目录 `tests/fixtures/`（包含多级目录、源文件、.gitignore）

---

### Phase 1：数据源模块（最多 4 个子 Agent 并行）

依赖 Phase 0 完成后启动。四个模块之间**无相互依赖**，可完全并行。

#### Module 1.1：CLI 参数解析

| 项 | 说明 |
|---|---|
| 模块名 | `cli-parsing` |
| 输出文件 | `src/cli.rs` |
| 设计文档 | `doc/detail/design-cli-parsing.md` |
| 引用 spec | `specs/cli-parsing/spec.md` |

**接口定义：**
```rust
pub fn parse_args() -> RawConfig
```

**实现要求：**
- 使用 clap derive macro 定义 `CliArgs` 结构体
- 支持所有命令行参数：`[ROOT]`, `-D`, `-U`, `-I`, `--exclude`, `--exclude-dir`, `--compiler`, `--std`, `--no-gitignore`, `--output`, `--config`, `--dry-run`, `--verbose`
- 实现 `CliArgs::to_raw_config() -> RawConfig` 方法
- -D 格式支持：`-D NAME`（空值）, `-D NAME=VALUE`, `-D NAME="spaced value"`
- -U 格式：`-U NAME`
- -I 可重复
- 自动生成 `--help`
- **编写单元测试**覆盖所有参数解析场景

**任务来自 tasks.md：** 任务组 3（3.1 ~ 3.14）

#### Module 1.2：配置文件解析

| 项 | 说明 |
|---|---|
| 模块名 | `config-parsing` |
| 输出文件 | `src/config.rs` |
| 设计文档 | `doc/detail/design-config-parsing.md` |
| 引用 spec | `specs/config-parsing/spec.md` |

**接口定义：**
```rust
pub fn find(root: &Path) -> Option<PathBuf>
pub fn parse(path: &Path) -> Result<RawConfig>
```

**实现要求：**
- 定义 `TomlConfig` 结构体（通过 serde Deserialize）
- `find()` 自动从项目根目录查找 `.ccgen.toml`
- `parse()` 读取并解析 TOML 文件，转换为 `RawConfig`
- 配置文件不存在时优雅降级
- 格式错误时返回人类可读错误
- **编写单元测试**覆盖完整配置、部分字段、格式错误场景

**任务来自 tasks.md：** 任务组 4（4.1 ~ 4.5）

#### Module 1.3：源码文件发现

| 项 | 说明 |
|---|---|
| 模块名 | `source-discovery` |
| 输出文件 | `src/discover.rs` |
| 设计文档 | `doc/detail/design-source-discovery.md` |
| 引用 spec | `specs/source-discovery/spec.md` |

**接口定义：**
```rust
pub fn find_sources(config: &CcgenConfig) -> Vec<PathBuf>
```

**实现要求：**
- 递归扫描项目目录下的 `.c/.cpp/.cc/.cxx` 文件
- 默认遵守 `.gitignore`（使用 `ignore` crate）
- 支持 `--no-gitignore` 禁用
- 支持 `--exclude` glob 模式（使用 `glob` crate 匹配）
- 排除 `.h/.hpp` 头文件
- 返回绝对路径
- **编写单元测试**覆盖 gitignore 过滤、exclude 排除、no-gitignore 场景

**任务来自 tasks.md：** 任务组 7（7.1 ~ 7.7）

#### Module 1.4：Include 路径处理

| 项 | 说明 |
|---|---|
| 模块名 | `include-path` |
| 输出文件 | `src/include_path.rs` |
| 设计文档 | `doc/detail/design-include-path.md` |
| 引用 spec | `specs/include-path/spec.md` |

**接口定义：**
```rust
pub fn resolve_all(config: &CcgenConfig) -> Vec<PathBuf>
```

**实现要求：**
- 对每个用户传入的 Include 目录执行 BFS 递归遍历，收集所有子目录
- 支持 `--exclude-dir` 排除指定目录及其所有子目录
- 所有路径转换为绝对路径（使用 `dunce` 处理 Windows）
- 路径分隔符统一为正斜杠 `/`
- **编写单元测试**覆盖递归发现、排除、路径归一化场景

**任务来自 tasks.md：** 任务组 8（8.1 ~ 8.5）

---

### Phase 2：处理与输出模块（最多 3 个子 Agent 并行）

需要 Phase 1 完成后启动，但三个模块之间**无相互依赖**。

#### Module 2.1：参数合并

| 项 | 说明 |
|---|---|
| 模块名 | `merger` |
| 输出文件 | `src/merger.rs` |
| 设计文档 | `doc/detail/design-merger.md` |
| 引用 spec | `specs/macro-config/spec.md` |
| 前置依赖 | 需读取 `cli.rs` 和 `config.rs` 的函数签名 |

**接口定义：**
```rust
pub fn merge(cli: RawConfig, file: RawConfig) -> CcgenConfig
```

**实现要求：**
- CLI 优先级高于配置文件
- defines 合并：同名宏以 CLI 为准
- undefs、include、exclude、exclude_dir 以 CLI 完全覆盖配置文件
- compiler、std、no_gitignore、output 以 CLI 优先
- ROOT 路径转换为绝对路径
- `--verbose` 模式下标注每个配置项的来源
- **编写单元测试**覆盖优先级冲突、合并逻辑

**任务来自 tasks.md：** 任务组 5（5.1 ~ 5.6）

#### Module 2.2：编译命令构建

| 项 | 说明 |
|---|---|
| 模块名 | `compile-cmd` |
| 输出文件 | `src/compile_cmd.rs` |
| 设计文档 | `doc/detail/design-compile-cmd.md` |
| 引用 spec | `specs/compiler-inference/spec.md`, `specs/language-standard/spec.md` |

**接口定义：**
```rust
pub fn infer_compiler(file_path: &Path) -> &str
pub fn build_entry(config: &CcgenConfig, source: &Path, include_dirs: &[PathBuf]) -> CompileEntry
```

**实现要求：**
- `infer_compiler`: `.c` → `gcc`, `.cpp/.cc/.cxx` → `g++`；config 中指定时覆盖
- `build_entry` arguments 数组格式：`[compiler, "-x", <lang>, "-c", <file>, "-I", <path>, ..., "-D", <macro>, ...]`
- 根据扩展名确定语言参数：`c` 或 `c++`
- 展开 Include 路径为 `-I` 对（每个路径一个 `-I` + 路径）
- 展开宏定义为 `-D` 对（有值：`-D` + `NAME=VALUE`；无值：`-D` + `NAME`）
- 展开取消定义为 `-UNAME` 元素
- 如果指定了 `--std`，添加 `-std=<STD>` 元素
- 编译器仅作为字符串，不校验系统存在
- `directory` 字段为项目根绝对路径
- `--verbose` 模式输出构建日志
- **编写单元测试**覆盖编译器推断、arguments 格式、语言标准

**任务来自 tasks.md：** 任务组 9（9.1 ~ 9.4）+ 任务组 10（10.1 ~ 10.9）

#### Module 2.3：输出生成

| 项 | 说明 |
|---|---|
| 模块名 | `output` |
| 输出文件 | `src/output.rs` |
| 设计文档 | `doc/detail/design-output.md` |
| 引用 spec | `specs/output-generation/spec.md` |

**接口定义：**
```rust
pub fn write_to_json(entries: &[CompileEntry], path: &Path) -> Result<()>
pub fn print_json(entries: &[CompileEntry]) -> Result<()>
```

**实现要求：**
- JSON 序列化为符合 LLVM Compilation Database 规范的数组
- 每条包含 `directory`、`file`、`arguments` 三个字段，无 `output`
- 写入时先写临时文件，再通过 `std::fs::rename` 原子替换
- 支持 dry-run 模式输出到 stdout
- 如果输出目录不存在，递归创建
- **编写单元测试**覆盖 JSON 格式、原子写入、dry-run

**任务来自 tasks.md：** 任务组 11（11.1 ~ 11.5）

---

### Phase 3：主流程集成（1 个子 Agent）

| 项 | 说明 |
|---|---|
| 模块名 | `main-integration` |
| 输出文件 | `src/main.rs`（完整实现） |
| 设计文档 | `doc/detail/design-main-integration.md` |
| 前置依赖 | 所有 Phase 0-2 模块完成 |

**接口定义（src/lib.rs）：**
```rust
pub fn run(config: CcgenConfig) -> Result<()>
```

**实现要求：**
- 组装完整处理流程：解析 CLI → 解析 Config → 合并 → 源码发现 → Include 路径发现 → 为每个源文件构建编译命令 → JSON 输出
- 使用 anyhow 传播错误
- `--verbose` 输出各阶段处理摘要（发现的源文件数、Include 路径数、生成的命令数）
- 无源文件时输出警告但正常退出
- `src/lib.rs` 中暴露 `run(config)` 供测试调用
- `main.rs` 仅作为薄的 CLI 入口：parse_args → run

**任务来自 tasks.md：** 任务组 12（12.1 ~ 12.4）

---

### Phase 4：集成测试（1 个子 Agent）

| 项 | 说明 |
|---|---|
| 模块名 | `tests` |
| 输出文件 | `tests/integration_test.rs` |
| 设计文档 | `doc/detail/design-tests.md` |
| 前置依赖 | 所有 Phase 0-3 模块完成 |

**实现要求：**
- 使用 `tests/fixtures/` 下的测试项目目录
- 通过调用 `ccgen::run()` 进行集成测试
- 覆盖以下场景：
  1. 基本用法——扫描目录生成 JSON
  2. `-D` 宏定义注入
  3. `-I` Include 路径递归发现
  4. `--exclude` 排除源文件
  5. `--exclude-dir` 排除 Include 子目录
  6. `--compiler` 覆盖编译器
  7. `--std` 语言标准
  8. `--no-gitignore` 禁用 gitignore 过滤
  9. `--dry-run` 输出到 stdout
  10. 参数合并优先级（CLI > 配置文件）
  11. 所有路径为绝对路径且使用正斜杠
  12. 原子写入（验证临时文件被清理）
  13. `--config` 指定配置文件路径

**任务来自 tasks.md：** 任务组 13（13.1 ~ 13.14）

---

## 引用 spec 索引

| 模块名 | 引用 specs 路径 |
|--------|----------------|
| setup | 全局类型需求 |
| cli-parsing | `specs/cli-parsing/spec.md` |
| config-parsing | `specs/config-parsing/spec.md` |
| source-discovery | `specs/source-discovery/spec.md` |
| include-path | `specs/include-path/spec.md` |
| merger | `specs/macro-config/spec.md` |
| compile-cmd | `specs/compiler-inference/spec.md`, `specs/language-standard/spec.md` |
| output | `specs/output-generation/spec.md` |
| main-integration | 全局流程需求 |
| tests | 全局验收标准 |

---

## 工程规范

1. **错误处理**：使用 `anyhow::Result` 作为函数返回值，`CcgenError` 枚举用于模块内部
2. **路径处理**：所有对外输出的路径必须是绝对路径 + 正斜杠 `/`
3. **编码风格**：遵循 Rust 2021 edition 标准，使用 `cargo fmt` 格式化
4. **代码质量**：每个子 Agent 交付前确保 `cargo build` 通过
5. **测试**：每个模块至少包含 3 个有意义的单元测试，集成测试覆盖 13+ 场景
6. **设计文档**：每个子 Agent 完成后编写 `doc/detail/design-<module-name>.md`，包含模块接口、关键实现决策、测试策略

---

## 完成标准

- [ ] `cargo build` 编译通过，无 warning
- [ ] `cargo test` 所有单元测试 + 集成测试通过
- [ ] `cargo clippy` 无 lint 警告
- [ ] 所有 tasks.md 中的子任务标记为已完成 `[x]`
- [ ] 所有 `doc/detail/design-*.md` 设计文档已生成
- [ ] 测试 fixtures 中的测试项目能正确使用 `ccgen` 生成 `compile_commands.json`

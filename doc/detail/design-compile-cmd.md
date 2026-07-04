# 编译命令构建模块

## 职责

为每个源文件生成对应的编译命令，构造 `CompileEntry`。

## 接口

### `compile_cmd.rs`

```rust
pub fn infer_compiler(file_path: &Path) -> &str
```

根据源文件扩展名推断编译器：
- `.c` → `"gcc"`
- `.cpp` / `.cc` / `.cxx` → `"g++"`
- 其他（含无扩展名）→ `"gcc"`

```rust
pub fn build_entry(
    config: &CcgenConfig,
    source: &Path,
    include_dirs: &[PathBuf],
) -> CompileEntry
```

构建单个源文件的编译条目。

## `build_entry` 参数构建顺序

1. **编译器**：`config.compiler`（若指定）→ `infer_compiler(source)`（默认）
2. **语言标志**：`-x c`（`.c` 文件）或 `-x c++`（其余扩展名）
3. **编译模式**：`-c`
4. **源文件路径**：`source.to_string_lossy()`
5. **Include 目录**：每个目录对生成 `-I <dir>`
6. **宏定义**：每个 `MacroDef` 生成 `-D <NAME>` 或 `-D <NAME>=<VALUE>`
7. **宏取消定义**：每个 undef 生成 `-U <NAME>`
8. **语言标准**：若指定则生成 `-std=<std>`

### 参数结构示例

```rust
CompileEntry {
    directory: config.root.clone(),         // 工作目录
    file: source.to_path_buf(),             // 源文件路径
    arguments: vec![
        "gcc",                              // 编译器
        "-x", "c",                          // 语言
        "-c",                               // 编译模式
        "/project/src/main.c",              // 源文件
        "-I", "/project/include",           // include 路径
        "-D", "FOO",                        // 宏定义
        "-D", "BAR=1",
        "-U", "OLD",                        // 宏取消
        "-std=gnu11",                       // 语言标准
    ],
}
```

## 关键实现决策

- **默认编译器按扩展名推断**：减少用户配置负担，常用即默认
- **`-x` 语言标志**：显式指定语言避免编译器根据扩展名推导的不确定性
- **宏定义值的字符串化**：`format!("{}={}", name, value)`，保证编译命令的跨平台可读性
- **verbose 日志**：输出每个源的编译命令详情，便于调试

## 数据结构设计

```rust
pub struct CompileEntry {
    pub directory: PathBuf,     // 工作目录（root）
    pub file: PathBuf,          // 源文件绝对路径
    pub arguments: Vec<String>, // 编译命令参数列表
}
```

实现了 `Serialize`（通过 derive），用于序列化为 JSON。

## 测试策略

单元测试使用辅助宏 `assert_args!` 简化参数列表比较：
- **`infer_compiler`**：覆盖 `.c`、`.cpp`、`.cc`、`.cxx`、头文件、无扩展名
- **完整参数**：defines、undefs、std、include_dirs 全部指定
- **编译器覆盖**：指定 `clang` 覆盖默认推断
- **C++ 语言**：`.cpp` 文件自动使用 `-x c++` 和 `g++`
- **最小化参数**：无任何额外配置时的默认命令
- **多个 include 目录**：验证 `-I` 参数顺序
- **宏定义有值/无值**：两种形态的 `-D` 输出
- **`-U` 参数**：取消宏定义
- **`-std` 参数**：语言标准
- **verbose 日志**：验证不 panic

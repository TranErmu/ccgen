---
# ============================================================
# 必需字段（缺一不可）
# ============================================================

# 技能的唯一标识符。约束：
#   - 长度 1-64 个字符
#   - 只能包含小写字母（a-z）、数字（0-9）和连字符（-）
#   - 不能以连字符开头或结尾，也不能出现连续连字符（--）
#   - 必须和这个 SKILL.md 所在的父目录名完全一致
#     （比如目录是 pdf-processing/，这里就必须是 pdf-processing）
name: skill-name

# 描述这个技能"做什么"以及"什么时候该用它"。
# 约束：长度 1-1024 个字符。
# agent 在"发现阶段"只看得到 name + description，所以这段话的
# 质量直接决定技能会不会被正确触发 —— 尽量写具体关键词，
# 别只写功能，也要写触发场景。
#
#   反例（信息量太低）：
#     description: Helps with PDFs.
#   正例：
#     description: >
#       Extracts text and tables from PDF files, fills PDF forms, and
#       merges multiple PDFs. Use when working with PDF documents or
#       when the user mentions PDFs, forms, or document extraction.
description: >
  <在这里写清楚：①这个技能具体能做什么；②什么场景/关键词下应该
  触发它。两者都要覆盖，越具体越好。>

# ============================================================
# 可选字段
# ============================================================

# 这个技能适用的许可证。建议简洁：写许可证名称，
# 或指向技能目录里打包的许可证文件。
# 没有特殊许可要求可以整行删除。
license: Apache-2.0

# 仅当该技能对运行环境有特殊要求时才填写 —— 多数技能不需要这个字段。
# 长度 1-500 个字符，可以说明：适配的目标产品/客户端、依赖的系统包
# 或命令行工具、是否需要网络访问等。
# 例如：
#   compatibility: Designed for Claude Code (or similar products)
#   compatibility: Requires git, docker, jq, and access to the internet
#   compatibility: Requires Python 3.14+ and uv
# 没有特殊要求可以整行删除。
compatibility: <可选：说明环境/依赖要求>

# 任意的 key-value 附加信息，规范本身不限定具体字段。
# 建议 key 取名尽量"唯一"，避免和其他工具/平台的内部约定撞车。
# 不需要可以整段删除。
metadata:
  author: <你的名字或组织名>
  version: "1.0"

# 实验性字段：预先批准该技能可以使用的工具，空格分隔。
# 各家 agent 实现对这个字段的支持程度不一致，不要完全依赖它。
# 例如：allowed-tools: Bash(git:*) Bash(jq:*) Read
# 不需要可以整行删除。
allowed-tools: <可选：空格分隔的工具白名单>
---

# <技能的人类可读标题，比如 "PDF Processing">

<一两句话简介：这个技能解决什么问题，呼应上面 description 的内容>

## Instructions

<这是 agent 真正会读到、并照做的步骤说明。写清楚"怎么做"，
建议按步骤拆解，越具体越好，例如：

1. 第一步要做什么、用什么工具/命令
2. 第二步要做什么
3. 遇到什么情况要怎么判断分支
4. 最后如何收尾/校验结果>

## Examples

<给出 1-2 组典型的输入 -> 输出示例，帮助 agent 理解期望的格式与效果>

**输入：**
<示例输入>

**输出：**
<示例输出>

## Common edge cases

<列出容易出错、需要特殊处理的边界情况，以及对应处理方式，例如：>

- <边界情况 1> → <应该怎么处理>
- <边界情况 2> → <应该怎么处理>

## Reference materials

<如果有更详细的文档，放进同目录下的 references/ 文件夹，
再在这里用相对路径引用 —— agent 只有读到这一行才会按需加载该文件，
不引用就不会消耗上下文。例如：>

See [the reference guide](references/REFERENCE.md) for details.

## Scripts

<如果有可执行脚本，放进同目录下的 scripts/ 文件夹，
再在这里说明什么情况下应该调用、怎么调用。例如：>

Run the extraction script: `scripts/extract.py`
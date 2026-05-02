# Codex MCP Text Contract 设计文档

## 1. 文档身份

本文定义 Codex MCP Text Contract。该 contract 是 Codex 在调用显式 opt-in MCP server 时使用的模型可见输入/输出适配规则。它不修改 MCP 协议，不修改 OpenAI Responses API，不把 MCP server 编译进 Codex，也不把所有 MCP 工具自动转换为 Codex 内置工具。它只规定 Codex 的 MCP adapter 如何在模型交互面去除不必要的协议壳。

本文使用两种语气：

- indicative mood：陈述已经存在的事实、当前 flow、协议形状、源码观察、黑盒验证结果和现实约束。
- desiderative mood：陈述本 artifact 希望达成的效果、希望减少的摩擦、希望建立的稳定契约和希望排除的非目标。

本文的核心主张是：MCP 的协议对象和模型应阅读的工具结果不是同一个对象。MCP result 的完整结构应由宿主、UI、日志、hook、trace 和自动化消费；模型上下文应接收 MCP server 明确放入 `content[0].text` 的文本。对于原始大块文本输入，MCP tool 可以通过唯一 `freeform: string` 字段声明它是 freeform 工具；Codex 可以把该工具映射为 Responses custom/freeform tool。

## 2. Indicative Mood：当前事实

### 2.1 MCP 已经区分内容通道

MCP tool result 包含 `content`、`structuredContent`、`isError` 和 `_meta` 等字段。`content` 是内容数组，常见元素为 text content。`structuredContent` 是结构化 JSON 结果。`isError` 是机器可读状态。`_meta` 是元信息。

这四个字段不是同一类对象。`content` 承载可呈现内容。`structuredContent` 承载机器可读对象。`isError` 承载状态。`_meta` 承载协议或宿主相关元数据。Codex 当前模型输出路径仍会向模型暴露 structuredContent 优先序列化结果、content wrapper JSON 或运行时 header，而不只是工具作者准备给模型看的文本。

### 2.2 `rmcp 1.6.0` 的黑盒行为

在 `/home/t103o/workbench/examples/temporary/rust-mcp-counter` 中，已经构造并运行了一个 Rust MCP server 和 client。server 暴露以下工具：

- `counter`：返回 pretty Markdown text。
- `get_count`：返回普通 text。
- `set_count`：返回普通 text。
- `counter_report`：返回 `Json<CounterReport>`，即 structured output。
- `counter_pretty_report`：手动返回 pretty Markdown `content` 和 JSON `structuredContent`。

黑盒 inspect client 调用这些工具后观察到：

```text
content.len = 1
content[0].type = text
```

`counter_report` 的结果同时包含：

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"count\":1,\"message\":\"当前计数值是 1\",\"parity\":\"odd\"}"
    }
  ],
  "structuredContent": {
    "count": 1,
    "message": "当前计数值是 1",
    "parity": "odd"
  },
  "isError": false
}
```

`counter_pretty_report` 的结果同时包含：

```json
{
  "content": [
    {
      "type": "text",
      "text": "# 计数器漂亮报告\n\n- 当前计数值: **1**\n- 奇偶性: `odd`\n\n> 当前计数值是 1"
    }
  ],
  "structuredContent": {
    "count": 1,
    "message": "当前计数值是 1",
    "parity": "odd"
  },
  "isError": false
}
```

该结果说明，在目标工具形态中，MCP server 可以把模型应阅读的文本稳定地放在 `content[0].text`，同时把机器可读数据放在 `structuredContent`。Codex 不需要把完整 envelope 展示给模型，也不需要设计多个 content item 的复杂线性化规则。

### 2.3 Codex 内置工具已经有低壳模型体验

Codex 内置终端工具和内置 `apply_patch` 工具并非完全没有壳，但它们向模型呈现的是 text-first 报告，而不是 JSON envelope。

终端工具输出形状类似：

```text
Chunk ID: ...
Wall time: ... seconds
Process exited with code 0
Output:
...
```

内置 `apply_patch` 输出形状类似：

```text
Exit code: 0
Wall time: 0 seconds
Output:
Success. Updated the following files:
A path/to/file
```

这些输出不是协议裸值，但它们是模型可读文本，不要求模型穿透 `[{"type":"text","text":"..."}]` 这样的协议壳。

### 2.4 Codex 源码中 MCP 输出仍未执行 content-only 投影

Codex 本地源码中存在两条 MCP 输出形态。`ToolOutput for CallToolResult` 将 MCP result 转为 `ResponseInputItem::McpToolCallOutput`，保留完整 `CallToolResult`。`McpToolOutput` 将 MCP result 转为 `ResponseInputItem::FunctionCallOutput`，并通过 `CallToolResult::as_function_call_output_payload()` 生成模型可见 payload。

`as_function_call_output_payload()` 当前优先序列化非 null `structuredContent`。没有 `structuredContent` 时，纯 text content 不会直接投影为 text，而是把 `content` 数组序列化为 JSON；只有包含 image content 时才生成 content items。`McpToolOutput::response_payload()` 还会在模型可见输出前添加 `Wall time: ... seconds\nOutput:` header，并执行函数输出截断。

该实现保留了协议信息和运行时信息，但没有执行本文定义的 `content[0].text` 投影。因此，content-only contract 的实现目标不是简单删除旧 `McpToolCallOutput`，而是在当前 `McpToolOutput` 模型输出路径中为 opt-in server 增加一个更窄的投影分支。

### 2.5 OpenAI Responses 已经支持 custom/freeform tool

OpenAI Responses 工具体系中存在 function tool 和 custom/freeform tool 的区分。function tool 适合 JSON schema 参数。custom/freeform tool 适合原始文本输入。Codex 内置 `apply_patch` 已经利用了 freeform/custom 路径，使模型可以直接输出 patch program，而不必把 patch 放进 JSON string 字段。

### 2.6 MCP 输入目前通常走 JSON arguments

MCP `tools/call` 使用 arguments object。普通 MCP tool input schema 是 JSON schema。Codex 将 MCP tool 暴露给模型时，通常要求模型生成 JSON arguments。对于 `read_file(path, line_range, show_line_numbers)` 这类多字段结构化工具，JSON arguments 是合适的。对于 patch、rewrite、splice 这类大块文法文本工具，JSON arguments 会引入额外转义和语法嵌套。

### 2.7 Codex MCP tool 声明和路由的当前形状

Codex 当前在 `codex-rs/tools/src/tool_registry_plan.rs` 中把直接暴露的 MCP tools 组织为 Responses namespace tool。每个 MCP tool 经 `mcp_tool_to_responses_api_tool()` 转成 function tool，handler 注册为 `ToolHandlerKind::Mcp`。模型发出 `ResponseItem::FunctionCall` 时，`ToolRouter::build_tool_call()` 通过 namespace/name 解析 `ToolInfo`，再构造 `ToolPayload::Mcp { server, tool, raw_arguments }`。

Responses custom/freeform tool 当前由 `ToolSpec::Freeform` 表达。模型发出 `ResponseItem::CustomToolCall` 时，Codex 只得到顶层 `name` 和 raw `input`，现有路由会构造 `ToolPayload::Custom { input }`。该路径不携带 MCP server name、原始 MCP tool name 或 namespace。因此，MCP freeform 不能只把 MCP 声明改成 custom tool；实现必须保留 custom tool name 到 `ToolInfo` 的可逆映射，并把 custom input 路由回 `ToolPayload::Mcp` 或等价的新 payload。

### 2.8 Codex config 的当前接入点

`[mcp_servers.<name>]` 当前由 `codex_config::McpServerConfig` 表示，由 `RawMcpServerConfig` 反序列化。`RawMcpServerConfig` 使用 `deny_unknown_fields`，未声明字段会被拒绝。配置 JSON schema 来自 `RawMcpServerConfig`。因此，`model_content_only` 和 `mcp_freeform` 必须同时进入 raw config、typed config、deserialization mapping 和 generated schema。

Plugin-provided MCP server 的 policy overlay 使用 `PluginMcpServerConfig`，当前只覆盖 enablement、approval 和 tool allow/deny policy。首版 contract 不要求 plugin overlay 暴露 `model_content_only` 或 `mcp_freeform`；plugin MCP 若需要该 contract，应另行设计 plugin policy surface。

## 3. Desiderative Mood：期望改变

Codex 应当能够把显式 opt-in MCP server 的模型可见输出投影为 `content[0].text`。模型应当看到 MCP server 作者为模型准备的文本，而不是 protocol-shaped result wrapper。

Codex 应当能够把显式 opt-in MCP server 中 schema 精确等于唯一 `freeform: string` 字段的 tool 暴露为 Responses custom/freeform tool。模型应当直接写原始 freeform 文本，而不是手动构造 JSON arguments。

MCP server 作者应当能够通过两个稳定动作表达意图：

1. 把模型应读内容放进 `content[0].text`。
2. 把 freeform 输入工具设计成唯一 `freeform: string` 字段。

Codex config 应当能够通过两个稳定开关启用该 contract：

```toml
[mcp_servers.docutouch]
model_content_only = true
mcp_freeform = true
```

启用后，Codex 不应再引入额外展示模式、字段名配置、工具 allowlist、空结果诊断或 envelope fallback。该 artifact 的目的不是修复未知工具，而是执行明确契约。

## 4. ACF Critical Questions 总表

### 4.1 问题世界

#### 4.1.1 谁在什么场景下遇到摩擦？

Codex 模型在使用 MCP 工具时遇到摩擦。MCP server 已经返回 text content，但 Codex 当前模型输出路径仍可能把 `structuredContent`、`content` wrapper 或运行时 header 放到模型可见输出中。模型要从这些非正文对象中提取正文。

MCP 工具作者在实现 patch、rewrite、splice 等大块文本工具时也遇到摩擦。工具本身需要接收一段原始文法文本，但普通 MCP/JSON arguments 迫使模型把该文本包进 JSON string 字段，导致换行、引号、反斜杠和代码片段需要额外转义。

#### 4.1.2 当前 flow 是什么？

返回 flow：MCP server 返回 `CallToolResult`。Codex 保存完整 result。模型上下文中可能出现 `structuredContent` JSON、`content` wrapper JSON 或带 wall-time header 的文本。

输入 flow：MCP server 在 `tools/list` 中声明 input schema。Codex 将该 schema 暴露为模型需要生成的 JSON arguments。模型调用工具时生成 JSON 字符串参数。Codex 将参数转给 MCP `tools/call`。

#### 4.1.3 摩擦发生在哪个动作或转折点？

返回摩擦发生在 MCP result 进入模型上下文时。MCP server 已经把正文放入 `content[0].text`，但 Codex 没有按该字段投影正文。

输入摩擦发生在模型生成工具调用参数时。大块文本工具的自然输入是 raw text，但模型必须生成 JSON arguments。

#### 4.1.4 当前替代方案是什么？

替代方案包括：

- 把工具注册为 Codex 内置工具。
- 使用 Codex 内置 `apply_patch`。
- 继续通过 MCP JSON arguments 调用工具。
- 在 MCP server 结果中尽量写更漂亮的 text，但 Codex 仍可能优先显示 structuredContent、content wrapper 或运行时 header。

这些方案不能同时满足独立进程插件、freeform 输入和 content-only 模型输出。

#### 4.1.5 现实约束有哪些？

MCP server 仍然通过标准 MCP 协议通信。Codex 是 Rust 实现，直接注册内置工具成本高。MCP 是自然插件机制。Responses API 已经支持 custom/freeform tool。MCP result 已经区分 `content` 和 `structuredContent`。Codex host 仍需保留结构化数据用于 UI、日志、hook、trace 和审计。

### 4.2 期望改变

#### 4.2.1 希望减少什么？

希望减少模型上下文中的协议壳。希望减少大块文本输入中的 JSON 字符串转义。希望减少模型从结构化 envelope 中提取正文的认知成本。希望减少上下文 token 噪声。

#### 4.2.2 希望增强什么？

希望增强 MCP 工具接近 Codex 内置工具的交互质量。希望增强 MCP server 作者对模型可见内容的控制。希望增强 Codex 对 Responses custom/freeform 能力的利用。希望增强 `content` 与 `structuredContent` 的职责分离。

#### 4.2.3 成功状态是什么？

当 `model_content_only = true` 时，模型只看到 `content[0].text`。当 `mcp_freeform = true` 且 tool schema 为唯一 `freeform: string` 字段时，模型以 custom/freeform 方式调用该工具。Codex 执行时仍向 MCP server 发送标准 `{ "freeform": raw_input }` arguments。

#### 4.2.4 是否可观察？

可观察。可以检查下一轮 Responses input 中是否只包含 text output，而不包含 MCP envelope。可以检查工具声明是否从 function JSON schema 变为 custom/freeform tool。可以检查 MCP server 实际收到的 arguments 是否为 `{ "freeform": raw_input }`。

#### 4.2.5 是否可验证？

可验证。可以构造 mock MCP server 和 mock Responses request，断言模型上下文、工具声明和 MCP call arguments 的精确形状。

### 4.3 人工制品依据

#### 4.3.1 为什么需要新 artifact？

现有 Codex MCP adapter 把 MCP 协议对象暴露给模型，未利用 MCP `content` 与 `structuredContent` 的消费分离。现有 adapter 也未把唯一 freeform 字符串工具映射为 Responses custom/freeform tool。需要一个 adapter contract 将已有协议能力对齐。

#### 4.3.2 替代方案是什么？

替代方案一：保持现状。模型继续看到 MCP envelope，freeform 文本继续包进 JSON arguments。该方案保留摩擦。

替代方案二：把 docutouch 等工具写成 Codex 内置工具。该方案破坏独立进程插件边界，增加编译和分发成本。

替代方案三：在 MCP server 中返回更漂亮的文本，但 Codex 仍暴露 envelope。该方案只能优化 text 内容，不能解决外层壳。

替代方案四：让模型自己学习忽略 envelope。该方案把 adapter 责任推给模型，增加 token 和错误面。

#### 4.3.3 关键假设是什么？

MCP server 作者在启用 contract 时会把模型应读内容放入 `content[0].text`。freeform tool 会使用唯一 `freeform: string` schema。Codex config 开关由知道该 server 行为的用户或开发者启用。

`rmcp-counter-reference.md` 记录了本地 `rmcp 1.6.0` counter experiment。该参考证明 pretty text content 与 `structuredContent` 可以在同一个 MCP result 中共存，并且 `content[0].text` 可以作为模型可见文本的稳定来源。该参考不证明 freeform input；freeform input 仍由 Codex adapter tests 证明。

#### 4.3.4 拒绝其他方案的理由是什么？

拒绝输出模式矩阵，因为 contract 已经规定模型只读 content。拒绝 allowlist，因为 schema 与 config 双重声明已经足够。拒绝字段名配置，因为字段名 `freeform` 是契约身份。拒绝空结果诊断，因为它污染忠实输出。

#### 4.3.5 权衡是什么？

该设计牺牲通用自动兜底，换取契约清晰和模型上下文纯净。启用 contract 的 server 必须遵守输出和输入规范。未遵守的 server 不应启用该 contract。

### 4.4 本体

#### 4.4.1 实体

- Codex MCP adapter：执行契约投影和 freeform 映射的宿主部件。
- MCP server：外部工具进程。
- MCP tool：由 server 暴露的工具。
- Model：消费工具结果并生成工具调用的语言模型。
- Host consumers：UI、日志、hook、trace、审计、自动化。

#### 4.4.2 值对象

- `model_content_only: bool`
- `mcp_freeform: bool`
- MCP `CallToolResult`
- MCP `content[0].text`
- MCP `structuredContent`
- MCP `isError`
- MCP input schema
- Responses custom/freeform input

#### 4.4.3 资源

- Codex config TOML
- MCP tool list
- MCP tool call result
- Responses tool declaration
- Responses turn input
- Codex trace/log/hook payload

#### 4.4.4 状态

- Server contract disabled
- Server content-only enabled
- Server freeform enabled
- Tool recognized as freeform
- Tool treated as ordinary MCP function tool

#### 4.4.5 事件

- MCP server initialized
- MCP tools listed
- Tool schema inspected
- Tool exposed to model
- Model emits tool call
- Codex invokes MCP `tools/call`
- MCP returns `CallToolResult`
- Codex projects result into model context

#### 4.4.6 不变量

- `model_content_only` 不改变 MCP wire result。
- `model_content_only` 不向模型输出 `structuredContent`、`isError`、`_meta`、MCP `content` wrapper 或 Codex wall-time header。
- `mcp_freeform` 不改变 MCP server 接收标准 arguments object 的事实。
- freeform recognition requires exact single required string field named `freeform`。
- Empty `content[0].text` projects to empty model output。
- MCP freeform tool 的 model-visible custom name 必须能解析回唯一 `ToolInfo`。

### 4.5 公共契约

#### 4.5.1 Consumer

模型是 `content[0].text` 的 consumer。Codex host 是 `structuredContent`、`isError`、`_meta` 和运行时信息的 consumer。MCP server 是 wrapped `{ freeform: raw_input }` arguments 的 consumer。

#### 4.5.2 输入是什么？

普通 MCP tools 的输入仍是 JSON arguments。freeform MCP tools 的模型可见输入是 raw string。Codex 执行时将 raw string 包装为 `{ "freeform": raw_string }`。

freeform MCP tool 的 Responses custom tool name 使用该 MCP tool 的 canonical display name，即 `ToolInfo::canonical_tool_name().display()`。该名称必须与现有 MCP name qualification 结果一致，避免 custom call 丢失 namespace 后无法路由。

#### 4.5.3 输出是什么？

模型可见输出是 `content[0].text` 原文。host-visible 输出是完整 `CallToolResult` 与宿主运行时数据。

#### 4.5.4 错误如何表达？

对模型，错误由 `content[0].text` 表达。对宿主，错误状态由 `isError` 和完整 result 表达。Codex 不为模型生成额外错误解释。

#### 4.5.5 哪些承诺稳定？

字段名 `freeform` 稳定。配置名 `model_content_only` 和 `mcp_freeform` 稳定。返回投影 `content[0].text` 稳定。structuredContent 不进入模型上下文的规则稳定。

#### 4.5.6 哪些不承诺？

不承诺为非 opt-in server 优化输出。不承诺自动修复违反 contract 的工具。不承诺把多 content item 线性化为特殊格式。不承诺将 host runtime metadata 插入模型上下文。

### 4.6 内部结构

#### 4.6.1 部件职责

MCP tool discovery 部件负责读取 tool schema。Freeform recognizer 负责判断 schema 是否精确匹配 `freeform: string`。Tool declaration builder 负责把匹配工具暴露为 Responses custom/freeform tool。MCP execution adapter 负责把 custom input 包回 MCP arguments。Result projector 负责把 `content[0].text` 写入模型上下文。Trace/log/hook 部件负责保存完整结果。

Config parser 负责接收 server-level `model_content_only` 和 `mcp_freeform`。MCP tool exposure 部件负责把每个 `ToolInfo` 与所属 server config 关联。Custom call router 负责在 `ResponseItem::CustomToolCall` 中识别 MCP freeform tool name，并生成 MCP payload，而不是落入普通 `ToolPayload::Custom`。Code-mode nested tool path 若不能生成 Responses custom call，则首版不要求将 MCP freeform 暴露给 code-mode nested tool list。

#### 4.6.2 谁拥有状态？

Codex config 拥有 server-level 开关。MCP server 拥有工具 schema 和结果内容。Codex host 拥有运行时 trace 与完整 result。模型不拥有协议状态。

#### 4.6.3 谁依赖谁？

Freeform recognizer 依赖 MCP tool input schema 和 config。Result projector 依赖 MCP `CallToolResult` 和 config。MCP server 不依赖 Codex internal types。

#### 4.6.4 哪些实现可替换？

内部 response item 形状可替换，只要模型上下文等价于 `content[0].text`。内部 trace 存储可替换，只要完整 MCP result 保留给 host consumers。

#### 4.6.5 哪些复杂度可以降低？

删除输出模式矩阵。删除 per-tool freeform allowlist。删除字段名配置。删除空结果诊断。删除 fallback envelope。删除多个 content item 线性化策略。

### 4.7 动态语义

#### 4.7.1 初始状态

Codex 读取 config。若 server 未启用 `model_content_only`，MCP 输出保持默认行为。若 server 未启用 `mcp_freeform`，MCP 输入保持默认 JSON function behavior。

#### 4.7.2 事件发生

启动时，Codex 初始化 MCP server 并列出 tools。对于每个直接暴露的 MCP tool，Codex 在该 server 的 `mcp_freeform` 开启时检查 input schema。匹配 freeform schema 的 tool 被声明为 Responses custom/freeform tool，并注册同名 MCP handler 和 custom-name-to-`ToolInfo` 映射。其他 tool 保持普通 MCP function tool。

调用时，模型对 ordinary MCP tool 生成 JSON arguments；对 freeform MCP tool 生成 raw custom input。Codex 将 freeform input 包装为 `{ "freeform": input }` 后调用 MCP server。

返回时，若该 server 的 `model_content_only` 开启，Codex 从 result 中取 `content[0].text` 写入模型上下文。该输出不包含 `structuredContent` 优先序列化结果，不包含 `content` 数组 JSON，不包含 wall-time header。完整 result 保留给 host consumers。

#### 4.7.3 状态转移

Tool 从 raw MCP definition 转为 model-visible function tool 或 model-visible custom tool。Tool call 从 model-visible custom input 转为 MCP arguments object。Tool result 从 MCP protocol result 转为 model-visible text projection。

#### 4.7.4 异常如何处理

工具执行错误由 MCP server 返回。模型看到 `content[0].text`。Codex host 看到 `isError` 和完整 result。Codex 不向模型插入额外解释。

#### 4.7.5 是否存在非法状态？

存在。启用 `mcp_freeform` 后，某 tool 声称 freeform 但 schema 不精确匹配唯一 `freeform: string`，则该 tool 不被识别为 freeform。启用 `model_content_only` 后，server 返回非 text content，则模型投影为空或无模型可见文本；这是工具与 contract 不一致，属于 server 设计问题，不由模型输出层修复。

存在另一个实现非法状态：freeform tool 声明为 Responses custom tool，但 custom call name 不能解析回唯一 MCP `ToolInfo`。该状态必须在 tool declaration 构建阶段被排除；不能等到工具调用时猜测 server 或 tool。

#### 4.7.6 是否有恢复路径？

恢复路径是关闭 config 开关或修正 MCP server schema/result。Codex 不通过插入伪文本恢复。

### 4.8 投影

#### 4.8.1 用户看到什么？

用户在 Codex 对话中看到工具返回的纯文本，不看到 MCP envelope。若工具返回空文本，用户看到空工具输出或无正文。

#### 4.8.2 API consumer 看到什么？

MCP server 仍看到标准 MCP arguments。Host-side consumers 仍可看到完整 `CallToolResult`。

#### 4.8.3 测试看什么？

测试看 Responses request 中模型上下文是否为纯 text。测试看 MCP server 收到的 arguments 是否为 `{ "freeform": raw_input }`。测试看 structuredContent 是否不进入模型文本。

#### 4.8.4 监控看什么？

监控看完整 result、isError、tool name、call id、wall time、trace metadata。监控不依赖模型文本。

#### 4.8.5 演示看什么？

演示展示同一个 MCP result 的两种消费：模型看到 `content[0].text`，host trace 看到完整 `structuredContent` 和 metadata。

### 4.9 黑盒规格

#### 4.9.1 给定输入，期望输出是什么？

给定 MCP result：

```json
{
  "content": [{ "type": "text", "text": "abc" }],
  "structuredContent": { "x": 1 },
  "isError": false
}
```

当 `model_content_only = true`，模型可见输出为：

```text
abc
```

给定 MCP result：

```json
{
  "content": [{ "type": "text", "text": "" }]
}
```

模型可见输出为空字符串。

给定 freeform custom input：

```text
hello
```

Codex 发给 MCP server：

```json
{ "freeform": "hello" }
```

#### 4.9.2 哪些场景必须通过？

- 普通 text result 去壳。
- structured result 保留 host-side，但不进入模型。
- pretty text + structuredContent result 只把 pretty text 给模型。
- 空 text result 给模型空。
- isError true result 给模型 content text。
- exact freeform schema 映射为 custom/freeform tool。
- 非 exact schema 保持普通 MCP tool。

#### 4.9.3 边界条件是什么？

空字符串是有效输出。JSON 字符串内容如果位于 `content[0].text`，模型就看到 JSON 字符串原文。structuredContent 可以存在，也可以不存在。isError 可以存在，也可以不存在。它们不改变模型投影。

#### 4.9.4 失败表现是什么？

如果工具 schema 不符合 freeform contract，它不被映射成 freeform。若 server 在 content-only contract 下返回非 text content，模型投影为空或没有模型正文。该失败应在开发者测试或 host trace 中发现，而不是通过模型上下文伪文本修复。

#### 4.9.5 何时判定完成？

当黑盒测试能证明：模型上下文只含 `content[0].text`，freeform tool 不要求模型生成 JSON arguments，MCP server 仍收到标准 `{ freeform }` arguments，完整 result 仍可由宿主消费，即判定完成。

### 4.10 实现

#### 4.10.1 当前实现满足哪些契约？

当前 MCP server 可以返回 `content[0].text` 和 `structuredContent`。`rmcp 1.6.0` 支持 `Json<T>` structured output，也允许手动构造 pretty text content + structuredContent。Codex 内部已有 `ToolSpec::Freeform`、`ResponseItem::CustomToolCall`、`ToolPayload::Custom`、`ToolPayload::Mcp`、`McpToolOutput`、`ToolInfo` 和 MCP handler 概念。

#### 4.10.2 当前实现违反哪些契约？

当前 Codex MCP adapter 没有在模型上下文中忠实投影 `content[0].text`。在 `McpToolOutput` 路径中，模型可见输出会优先使用 `structuredContent`，或把 `content` 数组序列化为 JSON，并添加 wall-time header。当前 Codex MCP adapter 也没有把 exact `freeform: string` MCP tool 映射为 Responses custom/freeform tool。

#### 4.10.3 技术限制是真实约束还是偶然限制？

MCP JSON-RPC 是真实约束。MCP server 收到 arguments object 是真实约束。模型必须看到 protocol-shaped output 是偶然限制。MCP freeform tool 必须由模型生成 JSON arguments 是偶然限制。Codex 可以在 adapter 层改变模型可见形状，而不改变 MCP wire protocol。

#### 4.10.4 首选实现落点是什么？

首选实现落点如下：

- `codex-rs/config/src/mcp_types.rs`：在 `RawMcpServerConfig` 和 `McpServerConfig` 增加 `model_content_only: bool` 与 `mcp_freeform: bool`，默认 `false`，并在 `TryFrom<RawMcpServerConfig>` 中显式传递。
- `codex-rs/core/config.schema.json` 的生成输入：使两个字段出现在 `[mcp_servers.<name>]` schema 中。
- `codex-rs/tools/src`：增加 freeform schema recognizer。recognizer 接收已解析的 `JsonSchema`，只接受 object schema、唯一 property `freeform`、该 property 为 string、`required` 精确等于 `["freeform"]`、`additionalProperties` 精确为 `false`。
- `codex-rs/tools/src/tool_registry_plan.rs`：直接暴露 MCP tools 时，若所属 server 开启 `mcp_freeform` 且 schema 匹配，则生成 `ToolSpec::Freeform`，tool name 使用 canonical display name，并注册 MCP handler。未匹配工具仍生成 function tool。
- `codex-rs/core/src/tools/router.rs`：在 `ResponseItem::CustomToolCall` 分支先尝试用 custom tool name 解析 MCP freeform `ToolInfo`。命中时生成 `ToolPayload::Mcp { raw_arguments: {"freeform": input} }`；未命中时保持普通 custom tool 行为。
- `codex-rs/core/src/tools/context.rs`：`McpToolOutput::to_response_item()` 在 server 开启 `model_content_only` 时返回只含 `content[0].text` 的 model output，不添加 wall-time header，不序列化 `structuredContent`，不序列化 `content` wrapper。日志、hook、code-mode result 仍保留完整 result。

#### 4.10.5 需要传递哪些新状态？

`McpToolOutput` 当前只有 result、tool input、wall time、image detail support 和 truncation policy，不知道 server config。实现需要把 `model_content_only` 放进 `McpToolOutput`，或把 equivalent policy 放进 `ToolPayload::Mcp`。`mcp_freeform` 需要在 tool registry plan 阶段可见，因此 `ToolRegistryPlanMcpTool` 或其上游输入需要携带 server-level freeform policy。

#### 4.10.6 哪些实现不进入首版？

首版不要求 deferred MCP tools 通过 `tool_search` 暴露为 freeform。当前 deferred/loadable spec 只表达 function/namespace 工具；把 custom/freeform 加入 deferred discovery 是另一个工具发现 surface。首版不要求 plugin MCP policy overlay 支持这两个字段。首版不要求 code-mode nested tool prompt 把 MCP freeform 呈现为 raw custom input。

#### 4.10.7 新发现是否已回到正确层位？

黑盒发现 `rmcp 1.6.0` 目标工具返回单一 text content。该发现属于 MCP server result 形状事实。设计已将它转化为 contract：content-only server 返回 `content[0].text` 作为模型文本。没有再引入多 text 拼接策略。

### 4.11 验证

#### 4.11.1 如何证明 artifact 做对了？

使用 mock MCP server 和 captured Responses requests。验证工具声明、工具调用、MCP arguments、模型上下文和 host trace。

#### 4.11.2 是否有可重复验证方式？

有。Rust MCP experiment 中的 inspect client 可重复观察 MCP result；本设计目录的 `rmcp-counter-reference.md` 记录了命令、工具表面和关键输出形状。Codex 测试可 mock Responses 和 MCP server，断言 request body。

#### 4.11.3 是否覆盖正常流、异常流和边界？

正常流：text result、pretty result、freeform input。异常流：isError true with text content。边界：empty text content、non-freeform single string schema、structuredContent present。

#### 4.11.3.1 ATDD 验收矩阵

Acceptance tests 应先表达外部可观察行为，再允许内部单元测试补足 recognizer 和 config 细节：

- Config acceptance：`[mcp_servers.docutouch] model_content_only = true` 与 `mcp_freeform = true` 能成功反序列化，默认值为 `false`，schema 包含两个字段，未知拼写仍被拒绝。
- Tool declaration acceptance：direct MCP tool 的 input schema 精确为唯一 required `freeform: string` 且 server 开启 `mcp_freeform` 时，Responses request 的 `tools` 中出现 `type: "custom"` 工具；同 server 中非 exact schema 工具仍是 function tool。
- Custom call routing acceptance：模型发出该 custom tool call 后，MCP server 收到 `arguments = { "freeform": raw_input }`，而不是 raw string，也不是 `{ "input": raw_input }`。
- Content-only output acceptance：server 返回 `{ content: [{ type: "text", text: "abc" }], structuredContent: { x: 1 }, isError: false }` 且开启 `model_content_only` 时，下一轮 Responses input 中对应 tool output 的文本为 `abc`，不包含 `structuredContent`、`content`、`isError`、`Wall time` 或 `Output:`。
- Error text acceptance：server 返回 `isError: true` 且 `content[0].text = "failed"` 时，模型可见输出为 `failed`，host-side result 仍保留 `isError: true`。
- Empty text acceptance：server 返回 `content[0].text = ""` 时，模型可见输出为空字符串，不生成诊断文本。
- Default behavior acceptance：server 未开启 `model_content_only` 或 `mcp_freeform` 时，现有 MCP output formatting、function declaration、routing 和 truncation tests 保持通过。

#### 4.11.3.2 推荐单元测试位置

- `codex-rs/config/src/mcp_types_tests.rs`：新增字段反序列化、默认值和 unknown-field 拒绝测试。
- `codex-rs/tools/src/mcp_tool_tests.rs` 或新文件：测试 exact freeform recognizer，覆盖 required 顺序、额外字段、nullable string、missing `additionalProperties`、`additionalProperties: true`。
- `codex-rs/tools/src/tool_registry_plan_tests.rs`：测试 opt-in server 的 freeform MCP tool 声明为 `ToolSpec::Freeform`，普通 MCP tool 仍为 function。
- `codex-rs/core/src/tools/router_tests.rs`：测试 `ResponseItem::CustomToolCall` 能路由到 MCP payload，并正确包装 raw input。
- `codex-rs/core/src/tools/context_tests.rs`：测试 `McpToolOutput` 的 content-only response item 不带 wall-time header，不使用 structured content，不丢失 code-mode raw result。
- `codex-rs/core/tests/suite/rmcp_client.rs` 或同级 integration test：用 stdio MCP test server 和 captured Responses request 验证端到端 request shape。

#### 4.11.4 验证是否对应 Desired Change？

对应。Desired Change 是减少 envelope 和 JSON arguments 摩擦。验证直接检查 envelope 是否不进入模型上下文，以及 freeform input 是否不要求模型生成 JSON。

#### 4.11.5 验证是否依赖内部细节？

黑盒验证不依赖内部细节。实现测试可以依赖 internal types，但 acceptance tests 应检查外部可观察的 Responses request 和 MCP call。

### 4.12 维护

#### 4.12.1 事实变了怎么办？

如果未来 MCP 或 rmcp 改变 result 形状，contract 仍以 `content[0].text` 为准。server 若启用 contract，应继续提供该字段。若不能提供，不应启用 contract。

#### 4.12.2 目标变了怎么办？

如果目标变为通用多模态 MCP rendering，应设计另一个 artifact。不得把该需求塞进 Codex MCP Text Contract。

#### 4.12.3 契约变了怎么办？

字段名 `freeform` 和 `content[0].text` 投影是稳定核心。改变它们等于新版本 contract。需要显式版本化。

#### 4.12.4 旧对象如何退出？

旧 MCP JSON function tools 可继续存在。新 freeform tools 使用 `freeform` schema。旧字段如 `patch` 不进入 freeform contract。

#### 4.12.5 测试和文档如何同步？

任何改变配置名、schema contract、投影规则都必须同步更新 tests 和文档。黑盒 inspect 示例应保留为验证材料。

#### 4.12.6 版本如何兼容？

默认关闭两个开关，保持现有行为。开启后按 contract 执行。旧配置不受影响。

### 4.13 发表视图

#### 4.13.1 面向谁？

面向 Codex 维护者、MCP server 作者、工具生态开发者和希望把外部工具做成原生体验的高级用户。

#### 4.13.2 省略了什么？

省略通用 MCP 多模态 rendering。省略坏 server 自动修复。省略 UI 展示细节。省略 hook payload 具体字段设计。

#### 4.13.3 是否夸大了承诺？

没有。该 contract 只承诺模型可见输入/输出去壳，不承诺所有 MCP 工具都适合 freeform，不承诺所有 result 都能无损变成文本。

#### 4.13.4 是否与当前维护体一致？

一致。Codex 已有 custom tool、MCP tool、function output 和 host trace 概念。该设计只把现有能力重新连接。

#### 4.13.5 是否帮助读者正确理解 artifact？

帮助。读者可以用一句话理解：启用后，模型写 freeform，模型读 content；MCP 和 host 仍然保留结构化协议。

### 4.14 递归使用与当前工作上下文

#### 4.14.1 当前主要 artifact 是哪个？

当前主要 artifact 是 Codex MCP Text Contract。

#### 4.14.2 当前操作是什么？

当前操作是设计和发表，不是实现补丁。

#### 4.14.3 当前操作需要展开哪些构成性位置？

需要展开问题世界、期望改变、本体、公共契约、动态语义、黑盒规格、实现落点和验证方式。

#### 4.14.4 Parent、child 或 sibling artifact 是否影响当前工作？

Parent artifact 是 Codex tool system。Child artifact 是 `model_content_only` 投影器和 `mcp_freeform` recognizer。Sibling artifacts 包括普通 MCP adapter、Codex 内置 apply_patch、Codex terminal tool。它们提供对照，但不扩大 contract 边界。

#### 4.14.5 当前工作上下文是否引入无关上游、内部细节或相邻对象？

本文只在必要处引用内部实现落点，不把内部类型作为公共契约。本文不把 UI 多模态 rendering、坏 server 防御或 host runtime annotation 纳入目标。

#### 4.14.6 Boundary 的“不负责”项是否只包含相邻、易混淆且误配代价较高的职责？

是。不负责项包括通用 MCP rendering、空结果解释、多模式输出、字段名配置和工具 allowlist。这些职责容易与本 contract 混淆，但纳入后会破坏最小对象。

## 5. 最终规范摘要

### 5.1 配置

```toml
[mcp_servers.<name>]
model_content_only = true
mcp_freeform = true
```

### 5.2 返回投影

```text
model_visible_output = call_tool_result.content[0].text
```

空字符串保持空。错误文本由 MCP server 写入 content。Codex 不向模型添加解释。

### 5.3 Freeform schema

```json
{
  "type": "object",
  "properties": {
    "freeform": {
      "type": "string"
    }
  },
  "required": ["freeform"],
  "additionalProperties": false
}
```

匹配该 schema 的工具在 `mcp_freeform = true` 时暴露为 Responses custom/freeform tool。

### 5.4 执行映射

```text
Responses custom input -> MCP arguments { "freeform": input }
```

custom tool name 必须可解析回唯一 MCP `ToolInfo`。首选 name 是 canonical display name，例如 `mcp__docutouch__apply_patch`。

### 5.5 首版实现边界

首版覆盖直接暴露的用户配置 MCP servers。首版不覆盖 deferred/tool_search MCP freeform、plugin MCP policy overlay freeform、code-mode nested freeform prompt。未启用配置的 MCP server 保持现有行为。

### 5.6 实现入口

主要改动入口是：

- config：`codex-rs/config/src/mcp_types.rs`
- tool declaration：`codex-rs/tools/src/tool_registry_plan.rs` 和 MCP schema conversion helpers
- custom routing：`codex-rs/core/src/tools/router.rs`
- MCP output projection：`codex-rs/core/src/tools/context.rs`
- integration tests：`codex-rs/core/tests/suite/rmcp_client.rs` 或同级 MCP suite

### 5.7 宿主消费

`structuredContent`、`isError`、`_meta`、wall time、call id、trace data 由 Codex host 消费，不进入模型可见输出。

## 6. 结论

Codex MCP Text Contract 把 MCP 和 Responses 中已经存在的分离关系对齐。MCP `content` 成为模型输出通道。MCP `structuredContent` 成为宿主和程序通道。MCP 唯一 `freeform: string` schema 成为 Responses custom/freeform 输入通道。Codex adapter 执行确定性转换，不猜测、不修补、不附加伪文本。

该设计的价值来自去除不必要对象。它不增加新的内容语义，只移除模型不该消费的协议壳。它不改变 MCP，只让 Codex 更正确地使用 MCP。它不把外部工具变成内置工具，却让外部工具获得接近内置工具的模型交互体验。

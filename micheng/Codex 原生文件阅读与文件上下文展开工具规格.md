# Codex 原生文件阅读与文件上下文展开工具规格

## 1. 名称

本文定义两个 Codex core 内置模型工具：

- `read_file`
- `load_files_context`

`read_file` 是单文件文本阅读工具。

`load_files_context` 是文件路径序列到原生 `read_file` 工具轨迹的上下文展开触发器。

## 2. 问题空间

### 2.1 当前问题

Codex 模型在完成代码理解、文档精读、计划审查、规格整合等任务时，经常需要读取一组已经确定的文件。

现有方式存在三个结构性问题。

第一，使用 shell 或 `bat` 读取文件时，输出来自命令 stdout。单文件读取尚可接受；多文件参数会把多个文件正文拼进一个输出流。文件边界不再由工具协议表达，而是只能依赖文本分隔、命令配置或模型猜测。上下文被截断、文件内容相似、文件末尾缺少明显标志时，边界不可靠。

第二，使用普通工具多次读取文件时，模型需要逐个生成工具调用参数。若目标文件数量很大，模型输出大量重复 JSON 参数和路径字符串。这些输出不包含推理价值，只是机械调度成本。若不能在一轮内完成，还会增加请求轮次。

第三，MCP 工具作为外挂工具，只能把自身调用结果返回给 Codex。MCP server 不能直接向 Codex core 的 model-visible history 注入多个 sibling Responses items。因此 MCP 无法实现“把一组文件恢复为多组原生 `read_file` call/output 历史项”的语义。

### 2.2 期望改变

Codex 应提供内置文件阅读工具，使模型可以直接以结构化工具调用读取文件文本，而不是通过 shell stdout 间接阅读。

Codex 还应提供一个文件上下文展开触发器，使模型在已经确定目标文件路径序列时，只需提交路径清单，runtime 即可按清单顺序生成等价的普通 `read_file` 调用轨迹，并将这些轨迹写入 model-visible Responses history。

### 2.3 成功状态

实现完成后，模型可以：

1. 调用 `read_file` 读取一个文件的全文或连续行片段。
2. 控制 `read_file` 是否显示 1-based 行号。
3. 调用 `load_files_context` 提交一个路径清单。
4. 在下一次模型输入中看到按清单顺序生成的原生 `read_file` call/output 对。
5. 看到每个文件读取的独立成功或失败结果。
6. 不需要解析一个 batch 工具返回的大型聚合文本。
7. 不需要模型手动输出大量重复 `read_file` 调用参数。

## 3. 术语

### 3.1 Responses history

Codex 发送给 OpenAI Responses API 的模型可见历史。源码中主要表现为 `Vec<ResponseItem>`。

### 3.2 `ResponseItem::FunctionCall`

表示一次模型工具调用的 Responses history item。相关字段包括：

- `name`
- `namespace`
- `arguments`
- `call_id`

### 3.3 `ResponseItem::FunctionCallOutput`

表示一次工具调用结果的 Responses history item。它通过 `call_id` 与对应 `FunctionCall` 绑定。

### 3.4 原生读取轨迹

由一组交错的 `FunctionCall(read_file)` 与 `FunctionCallOutput` 构成的 Responses history 片段。

形态为：

```text
FunctionCall(read_file, args_0, call_id_0)
FunctionCallOutput(call_id_0, output_0)
FunctionCall(read_file, args_1, call_id_1)
FunctionCallOutput(call_id_1, output_1)
...
```

### 3.5 文件路径序列

由文本清单表示的有序路径列表。每一行对应一个路径字符串。顺序即展开顺序。

## 4. 设计纪律

### 4.1 对象边界

`read_file` 只负责单文件文本阅读。

`load_files_context` 只负责把路径序列展开成普通 `read_file` 调用轨迹。

二者不得互相吞并职责。

### 4.2 不使用 stdout 聚合边界

任何多文件阅读结果不得通过单个 stdout、单个 Markdown 文本块、单个 XML 容器或单个 JSON 数组作为主要模型上下文呈现。

多文件边界必须由 Responses history 中的独立 `FunctionCall` / `FunctionCallOutput` pair 表达。

### 4.3 不设计文件清单小语言

`load_files_context` 的清单不是 glob、脚本、配置文件或查询语言。

清单每一行只表示一个路径字符串。

工具不得解释：

- glob
- 注释
- include
- command substitution
- inline metadata
- range annotation
- dedup instruction

### 4.4 不定义批次成功率

`load_files_context` 不拥有“批量文件读取成功”这个对象。

它的成功含义是：路径序列已经被展开为普通 `read_file` 调用轨迹。

单个目标文件读取失败属于对应 `read_file` 调用的结果，不属于 `load_files_context` 的批次失败。

### 4.5 不使用版本叙事

本文只定义当前人工制品对象。

不得使用“第一版”“第二版”“以后可以扩展”等过程语言定义当前对象。

若出现其他能力需求，应作为另一个对象另行定义，不得污染本文对象。

## 5. Codex 源码事实依据

### 5.1 Responses 请求结构

`codex-rs/codex-api/src/common.rs` 中 `ResponsesApiRequest` 包含：

```rust
pub input: Vec<ResponseItem>
pub tools: Vec<serde_json::Value>
```

这说明 Codex 发送给模型的是结构化 `ResponseItem` 序列。

### 5.2 Responses item 模型

`codex-rs/protocol/src/models.rs` 中 `ResponseItem` 包含：

```rust
ResponseItem::FunctionCall { ... }
ResponseItem::FunctionCallOutput { ... }
```

这些变体正好表达工具调用与工具结果。

### 5.3 请求构造路径

`codex-rs/core/src/session/turn.rs` 中，采样前通过：

```rust
sess.clone_history().await.for_prompt(...)
```

生成模型请求输入。

`build_prompt` 将该输入设置为：

```rust
Prompt { input, tools: router.model_visible_specs(), ... }
```

`codex-rs/core/src/client.rs` 中 `build_responses_request` 将 `Prompt.input` 放入 `ResponsesApiRequest.input`。

### 5.4 普通工具执行路径

普通工具执行遵循以下源码路径：

1. 模型输出 `ResponseItem::FunctionCall`。
2. `handle_output_item_done` 识别工具调用。
3. `ToolRouter::build_tool_call` 转为内部 `ToolCall`。
4. Codex 记录原始 `FunctionCall`。
5. `ToolCallRuntime` 分发到具体 `ToolHandler`。
6. handler 返回工具输出。
7. Codex 将输出转成 `ResponseItem::FunctionCallOutput`。
8. Codex 记录工具输出。
9. 下一次采样请求携带 call/output 历史。

### 5.5 History normalization

Codex history normalization 要求：

- 每个 function call 有对应 output。
- 每个 output 有对应 call。

缺失 output 的 call 会被补 `aborted`。

orphan output 会被移除或触发 debug 错误。

因此 `load_files_context` 必须注入完整 call/output 对，不能只注入 output。

### 5.6 MCP 边界

MCP handler 处理一次 MCP tool call，返回一个 MCP result。Codex 将该 result 转为一个 `FunctionCallOutput` 或 `CustomToolCallOutput`。

MCP server 不能直接写入 Codex core 的 `Vec<ResponseItem>` history。

因此 MCP 不能实现 `load_files_context` 的原生 Responses history 展开语义。

## 6. `read_file` 规格

### 6.1 工具身份

`read_file` 是 Codex core 内置单文件文本阅读工具。

它读取一个文件的 UTF-8 文本，并返回请求的全文或连续行片段。

### 6.2 工具名称

```text
read_file
```

### 6.3 工具类型

Responses function tool。

### 6.4 参数

#### 6.4.1 `path`

类型：`string`

必填。

含义：目标文件路径。

路径解析遵循 Codex 当前 turn environment 的文件系统规则。相对路径相对于当前 turn environment 的 `cwd` 解析。

#### 6.4.2 `line_range`

类型：`string`

可选。

含义：连续行范围。

未提供时读取整个文件。

语法：

```text
start:end
```

`start` 和 `end` 均为可选端点，但冒号必须存在。

有效形态包括：

```text
1:100
1:
:100
-100:
:-1
```

正数端点按 1-based 行号解释。

负数端点按文件尾部相对定位。`-1` 表示最后一行，`-2` 表示倒数第二行。

端点解析后，范围包含起始行和结束行。

若 `start` 缺省，起始行为第 1 行。

若 `end` 缺省，结束行为最后一行。

若解析后起始行小于 1，则起始行取第 1 行。

若解析后结束行大于最后一行，则结束行取最后一行。

若解析后起始行大于结束行，返回空文本。

若 `line_range` 不是合法范围表达式，`read_file` 返回工具错误。

#### 6.4.3 `show_line_numbers`

类型：`boolean`

可选。

默认值：`false`

含义：是否在输出中渲染 1-based 行号。

`false` 时，输出为所选文件文本本身。

`true` 时，每一行渲染为：

```text
<line_number> | <line_text>
```

示例：

```text
1 | # Title
2 |
3 | content
```

#### 6.4.4 `environment_id`

类型：`string`

条件性可选。

当 Codex session 暴露多 environment 工具参数时，`read_file` 遵循现有 `view_image` 的 environment 选择模式，允许指定目标 environment。

这不是阅读能力的一部分，而是 Codex environment plumbing 的一部分。

### 6.5 输出

#### 6.5.1 默认输出

默认输出是纯文本。

工具不得在正文中添加：

- 文件路径标题
- shell 命令信息
- 执行时间
- JSON wrapper
- Markdown fence
- 文件边界分隔符

路径身份由 `FunctionCall.arguments.path` 表达。

#### 6.5.2 带行号输出

当 `show_line_numbers = true` 时，输出为带 1-based 行号的文本渲染。

带行号输出是定位模式，不是默认阅读模式。

### 6.6 错误

`read_file` 的错误以普通工具失败输出返回。

错误来源包括：

- 参数 JSON 无法解析。
- `path` 缺失或类型错误。
- `line_range` 语法非法。
- 路径不存在。
- 路径不是文件。
- 文件无法读取。
- 文件无法按 UTF-8 解码。
- sandbox 或权限拒绝。

### 6.7 实现落点

新增文件：

```text
codex-rs/core/src/tools/handlers/read_file.rs
codex-rs/core/src/tools/handlers/read_file_spec.rs
```

在：

```text
codex-rs/core/src/tools/handlers/mod.rs
```

导出 handler。

在：

```text
codex-rs/core/src/tools/spec_plan.rs
```

按 environment 工具注册模式注册。

`read_file` 应复用 Codex filesystem abstraction：

```text
ExecutorFileSystem::read_file_text
```

路径解析、environment 选择、sandbox context 应与 `view_image` 保持一致。

### 6.8 共享执行函数

实现中应提取共享执行函数，供 `ReadFileHandler` 与 `LoadFilesContextHandler` 使用。

推荐对象：

```text
ReadFileRequest
ReadFileExecutionResult
execute_read_file_text(...)
```

共享函数职责：

1. 根据 environment 和 cwd 解析路径。
2. 检查 metadata。
3. 读取 UTF-8 文本。
4. 应用 `line_range`。
5. 应用 `show_line_numbers`。
6. 返回文本或可转换为工具失败输出的错误。

共享函数不得记录 history。

history 记录由调用者决定。

## 7. `load_files_context` 规格

### 7.1 工具身份

`load_files_context` 是 Codex core 内置文件上下文展开触发器。

它接收一个文件路径序列，并按序列生成原生 `read_file` 工具调用轨迹。

### 7.2 工具名称

```text
load_files_context
```

### 7.3 工具类型

Responses function tool，带 Codex runtime history injection side effect。

### 7.4 参数

#### 7.4.1 `list_path`

类型：`string`

可选，但必须与 `list_text` 二选一。

含义：指向一个纯文本文件清单。

该清单文件本身通过 Codex filesystem 读取。

#### 7.4.2 `list_text`

类型：`string`

可选，但必须与 `list_path` 二选一。

含义：内联纯文本文件清单。

#### 7.4.3 `environment_id`

类型：`string`

条件性可选。

当 Codex session 暴露多 environment 工具参数时，`load_files_context` 可指定 environment。

该 environment 同时用于读取 `list_path` 指向的清单文件和清单中的目标文件。

### 7.5 参数互斥

`list_path` 与 `list_text` 必须且只能提供一个。

两者都未提供时，`load_files_context` 返回自身工具错误。

两者同时提供时，`load_files_context` 返回自身工具错误。

### 7.6 清单格式

清单是 UTF-8 文本。

清单按行分割。

每一行表示一个路径字符串。

行终止符不属于路径字符串。

CRLF 输入中，每行末尾的 `\r` 不属于路径字符串。

中间空行不是有效路径行；出现空行时，`load_files_context` 返回自身工具错误。

工具不解释 glob。包含 `*`、`?`、`[` 等字符的行仍然只是路径字符串。

工具不解释注释。以 `#` 开头的行仍然只是路径字符串。

工具不执行命令替换。

工具不支持 include。

工具不去重。

工具不重排。

### 7.7 展开语义

设清单解析得到路径序列：

```text
P = [p0, p1, ..., pn]
```

`load_files_context` 必须按序列顺序为每个 `pi` 生成一次默认 `read_file` 调用。

默认 `read_file` 调用参数为：

```json
{"path":"pi"}
```

若 `load_files_context` 自身带有 `environment_id`，synthetic `read_file` arguments 应包含同一 `environment_id`。

`load_files_context` 不为 synthetic `read_file` 提供 `line_range`。

`load_files_context` 不为 synthetic `read_file` 提供 `show_line_numbers`。

因此 synthetic `read_file` 默认读取完整文件，输出纯正文。

### 7.8 History 注入形态

对于每个路径 `pi`，runtime 必须生成一对 Responses history items：

```text
ResponseItem::FunctionCall {
  id: None,
  name: "read_file",
  namespace: None,
  arguments: serde_json({"path": pi, ...}),
  call_id: ci
}

ResponseItem::FunctionCallOutput {
  call_id: ci,
  output: read_file_result_i
}
```

这些 pair 必须按清单顺序交错排列：

```text
FunctionCall(read_file, p0, c0)
FunctionCallOutput(c0, output0)
FunctionCall(read_file, p1, c1)
FunctionCallOutput(c1, output1)
...
```

不得只注入 output。

不得把多个文件正文合并到 `load_files_context` 自己的 output。

不得把多个文件正文合并到一个 synthetic output。

### 7.9 `call_id` 生成

synthetic `read_file` call id 必须与触发器 call id 关联，并保证在当前 history 中唯一。

推荐形态：

```text
{load_call_id}__read_file__{index}
```

示例：

```text
call_abc__read_file__0
call_abc__read_file__1
```

`ResponseItem::FunctionCall.id` 可为 `None`。

配对依赖 `call_id`。

### 7.10 `load_files_context` 自身输出

`load_files_context` 自身仍然有一个普通 `FunctionCallOutput`，用于闭合它自己的工具调用。

该输出必须是短状态文本。

该输出不得包含文件正文。

推荐内容：

```text
expanded N read_file calls
```

如果清单为空，`load_files_context` 返回自身工具错误。

### 7.11 单个文件读取失败

清单解析成功后，每个路径都必须展开成 `read_file` 调用。

如果某个路径对应的 `read_file` 失败，runtime 必须为该 synthetic call 记录失败 output。

失败 output 的形态应与普通 `read_file` 工具失败一致。

后续路径继续展开。

`load_files_context` 不因单个 `read_file` 失败而停止，不撤销已生成轨迹，不定义批次失败。

### 7.12 `load_files_context` 自身失败

以下错误属于 `load_files_context` 自身失败：

- 参数 JSON 无法解析。
- `list_path` 和 `list_text` 都未提供。
- `list_path` 和 `list_text` 同时提供。
- `list_path` 指向的清单文件无法读取。
- 清单文件无法按 UTF-8 解码。
- 清单为空。
- 清单包含空行。

这些错误导致 `load_files_context` 返回自身失败 output。

这些错误不会生成 synthetic `read_file` 调用。

## 8. 离散事件仿真

### 8.1 当前普通工具调用仿真

#### 状态定义

```text
H: Codex model-visible Responses history, 类型为 Vec<ResponseItem>
T: 当前模型可见工具集合, 类型为 Vec<ToolSpec>
R: Responses API request
M: 模型
C: Codex runtime
```

#### 初始状态

```text
H0 = [UserMessage(...)]
T0 = 当前工具 specs
```

#### 事件序列

```text
E1: C 构造 R0
    R0.input = H0
    R0.tools = T0

E2: C 发送 R0 给 M

E3: M 输出 FunctionCall
    fc = ResponseItem::FunctionCall {
      name,
      arguments,
      call_id
    }

E4: C 记录 fc 到 H
    H1 = H0 + [fc]

E5: C 将 fc 转换为内部 ToolCall

E6: C 调度对应 ToolHandler

E7: ToolHandler 执行并返回工具结果
    out = ResponseInputItem::FunctionCallOutput {
      call_id,
      output
    }

E8: C 将 out 转换为 ResponseItem::FunctionCallOutput
    fo = ResponseItem::FunctionCallOutput { call_id, output }

E9: C 记录 fo 到 H
    H2 = H1 + [fo]

E10: 因为工具调用需要 follow-up，C 构造 R1
     R1.input = normalize(H2)
     R1.tools = T0

E11: M 在 R1 中观察到 fc 与 fo，并继续推理
```

### 8.2 `load_files_context` 仿真

#### 初始状态

```text
H0 = 当前 history
T0 包含 read_file 与 load_files_context
```

#### 模型触发

模型输出：

```text
lc = ResponseItem::FunctionCall {
  name: "load_files_context",
  arguments: {"list_text":"a.md\nb.md\nmissing.md\nc.md"},
  call_id: "call_load"
}
```

#### 展开事件

```text
E1: C 记录 lc
    H1 = H0 + [lc]

E2: C 调度 LoadFilesContextHandler

E3: Handler 解析 arguments，得到路径序列
    P = ["a.md", "b.md", "missing.md", "c.md"]

E4: Handler 为 P[0] 构造 synthetic read_file call
    fc0 = FunctionCall(read_file, {"path":"a.md"}, "call_load__read_file__0")

E5: Handler 执行 read_file 语义，读取 a.md
    result0 = 成功正文

E6: Handler 构造 output
    fo0 = FunctionCallOutput("call_load__read_file__0", result0)

E7: Handler 为 P[1] 构造 fc1

E8: Handler 执行 read_file 语义，读取 b.md

E9: Handler 构造 fo1

E10: Handler 为 P[2] 构造 fc2

E11: Handler 执行 read_file 语义，missing.md 不存在

E12: Handler 构造失败 output
     fo2 = FunctionCallOutput("call_load__read_file__2", read_file_error_output)

E13: Handler 为 P[3] 构造 fc3

E14: Handler 执行 read_file 语义，读取 c.md

E15: Handler 构造 fo3

E16: Handler 记录 synthetic items 到 history
     H2 = H1 + [fc0, fo0, fc1, fo1, fc2, fo2, fc3, fo3]

E17: Handler 返回 load_files_context 自身短状态
     lco = FunctionCallOutput("call_load", "expanded 4 read_file calls")

E18: C 记录 lco
     H3 = H2 + [lco]

E19: C 构造下一次 Responses request
     R1.input = normalize(H3)

E20: M 观察到：
     - a.md 成功读取
     - b.md 成功读取
     - missing.md 的 read_file 失败
     - c.md 成功读取
     然后 M 自行决定后续动作
```

#### 关键性质

`load_files_context` 不把 `missing.md` 的失败提升为自身批次失败。

`load_files_context` 不阻止 `c.md` 的展开。

模型看到的失败是普通 `read_file` 失败。

## 9. 实现架构

### 9.1 新增模块

```text
codex-rs/core/src/tools/handlers/read_file.rs
codex-rs/core/src/tools/handlers/read_file_spec.rs
codex-rs/core/src/tools/handlers/load_files_context.rs
codex-rs/core/src/tools/handlers/load_files_context_spec.rs
```

可增加共享模块：

```text
codex-rs/core/src/tools/handlers/file_reading.rs
```

共享模块只承载读取执行逻辑，不记录 history。

### 9.2 Handler 注册

在：

```text
codex-rs/core/src/tools/spec_plan.rs
```

当 Codex session 有 environment 时注册：

```text
ReadFileHandler
LoadFilesContextHandler
```

注册方式遵循现有 built-in handler 模式。

### 9.3 Tool spec

`read_file_spec.rs` 生成 Responses function tool spec。

`load_files_context_spec.rs` 生成 Responses function tool spec。

工具 spec 必须进入 `router.model_visible_specs()`，从而出现在 Responses request 的 `tools` 字段中。

### 9.4 `ReadFileHandler`

`ReadFileHandler` 实现普通 `ToolHandler`。

`tool_name()` 返回：

```text
read_file
```

`spec()` 返回 `read_file` tool spec。

`supports_parallel_tool_calls()` 返回 `true`。

`handle()`：

1. 解析参数。
2. 调用共享 `execute_read_file`。
3. 成功时返回文本 output。
4. 失败时返回 `FunctionCallError::RespondToModel`。

### 9.5 `LoadFilesContextHandler`

`LoadFilesContextHandler` 实现模型可见 `ToolHandler`。

`tool_name()` 返回：

```text
load_files_context
```

`spec()` 返回 `load_files_context` tool spec。

`supports_parallel_tool_calls()` 返回 `false`。

理由：该 handler 具有按序 history injection side effect。它的展开顺序必须稳定。

`handle()`：

1. 解析参数。
2. 校验 `list_path` / `list_text` exactly one。
3. 得到清单文本。
4. 解析路径序列。
5. 按顺序为每个路径构造 synthetic `read_file` call。
6. 对每个路径执行共享 `execute_read_file`。
7. 将成功或失败结果都构造成 synthetic `FunctionCallOutput`。
8. 将交错 synthetic items 写入 history。
9. 返回 `load_files_context` 自身短状态 output。

### 9.6 Synthetic item builder

定义 helper：

```text
build_synthetic_read_file_pair(parent_call_id, index, read_file_args, output_payload)
```

输出：

```text
[ResponseItem::FunctionCall, ResponseItem::FunctionCallOutput]
```

`arguments` 必须使用 `serde_json::to_string` 序列化。

不得手写 JSON 字符串。

### 9.7 History 写入

`LoadFilesContextHandler` 使用：

```text
Session::record_conversation_items(turn_context, synthetic_items)
```

或等价 core 内部 API。

写入对象必须是 `ResponseItem`，因为 synthetic trace 包含 `FunctionCall`。

不得只使用 `ResponseInputItem` 注入 synthetic output。

### 9.8 输出顺序

最终 history 顺序必须为：

```text
FunctionCall(load_files_context)
FunctionCall(read_file p0)
FunctionCallOutput(p0)
FunctionCall(read_file p1)
FunctionCallOutput(p1)
...
FunctionCallOutput(load_files_context)
```

该顺序来自现有普通工具流：模型 call 先被记录，handler side effect 在 handler 执行期间记录 synthetic pairs，handler 返回后普通 drain 记录触发器自己的 output。

## 10. 验收标准

### 10.1 `read_file` 验收

- 模型可见工具列表包含 `read_file`。
- `read_file` 能读取 UTF-8 文件全文。
- 默认输出不包含路径标题、命令信息、JSON wrapper、Markdown fence。
- `line_range` 能读取文件开头片段。
- `line_range` 能读取文件中间片段。
- `line_range` 能读取文件末尾片段。
- `line_range` 支持负数端点。
- `show_line_numbers = true` 输出 1-based 行号。
- `show_line_numbers = false` 输出纯正文。
- 路径不存在时返回普通工具失败 output。
- 路径是目录时返回普通工具失败 output。
- 非 UTF-8 文件返回普通工具失败 output。

### 10.2 `load_files_context` 验收

- 模型可见工具列表包含 `load_files_context`。
- `list_text` 可触发路径序列展开。
- `list_path` 可触发路径序列展开。
- `list_path` 和 `list_text` 同时出现时，`load_files_context` 自身失败。
- 两者都缺失时，`load_files_context` 自身失败。
- 清单为空时，`load_files_context` 自身失败。
- 清单中出现空行时，`load_files_context` 自身失败。
- 清单顺序被保留。
- 重复路径不被去重。
- glob-like 字符不被解释。
- 以 `#` 开头的行不被解释为注释。
- 每个路径生成一组 synthetic `read_file` call/output。
- synthetic pairs 在 history 中交错排列。
- synthetic output 与 synthetic call 的 `call_id` 匹配。
- `load_files_context` 自身 output 不包含任何文件正文。
- 某个目标文件读取失败时，对应 synthetic `read_file` output 为失败 output。
- 某个目标文件读取失败时，后续路径仍然展开。
- 下一次 Responses request 的 `input` 中包含 synthetic `read_file` call/output pairs。
- history normalization 后 synthetic pairs 不被当作 orphan output 移除。

### 10.3 MCP 边界验收

- 实现不依赖 MCP server。
- 实现不通过 MCP result 承载多文件正文。
- 实现不把 `load_files_context` 做成外部 MCP 工具。

### 10.4 stdout 边界验收

- 实现不调用 shell、`cat`、`sed`、`bat` 来读取文件正文。
- 多文件内容不经过单一 stdout 聚合。

## 11. 测试计划

### 11.1 Tool spec 测试

位置：

```text
codex-rs/core/src/tools/spec_plan_tests.rs
```

断言：

- `read_file` 在 environment tools enabled 时出现在 model-visible specs。
- `load_files_context` 在 environment tools enabled 时出现在 model-visible specs。
- 参数 schema 包含预期字段。

### 11.2 `read_file` handler 测试

新增测试模块：

```text
codex-rs/core/src/tools/handlers/read_file_tests.rs
```

覆盖：

- 全文读取。
- 行范围读取。
- 负数范围读取。
- 行号显示。
- 路径不存在。
- 目录路径。
- 非 UTF-8。
- invalid `line_range`。

### 11.3 `load_files_context` handler 测试

新增测试模块：

```text
codex-rs/core/src/tools/handlers/load_files_context_tests.rs
```

覆盖：

- `list_text` 展开。
- `list_path` 展开。
- exactly-one 参数验证。
- 空清单。
- 空行。
- 顺序保留。
- 重复路径保留。
- 目标文件读取失败但后续继续。
- 自身 output 不含文件正文。

### 11.4 History normalization 测试

位置：

```text
codex-rs/core/src/context_manager/history_tests.rs
```

覆盖：

- 完整 synthetic `read_file` call/output pairs 经过 `for_prompt` 后保留。
- orphan output 仍按既有规则处理。
- 缺 output 的 synthetic call 仍按既有规则补 `aborted`。

### 11.5 Responses request 集成测试

使用现有 core test support 中的 Responses mock。

覆盖：

1. mock 模型先输出 `load_files_context` FunctionCall。
2. Codex 执行 handler。
3. 下一次 mock Responses request 的 `input` 包含：
   - `load_files_context` call。
   - 交错 synthetic `read_file` call/output pairs。
   - `load_files_context` 自身 output。
4. 文件正文只出现在 synthetic `read_file` output 中。
5. `load_files_context` 自身 output 不包含文件正文。

## 12. 非目标

本文不定义：

- glob 扫描工具。
- 目录批量发现工具。
- 文件清单注释语法。
- 清单 include 语法。
- 每行携带 range 的批量读取计划。
- 批量读取事务。
- 批量读取摘要。
- MCP 实现。
- stdout 聚合读取。
- 文件内容缓存策略。
- UI 展示样式。

这些对象不属于当前问题。

## 13. 完成定义

当以下条件全部成立时，工作完成：

1. Codex core 中存在模型可见 `read_file` 内置工具。
2. Codex core 中存在模型可见 `load_files_context` 内置工具。
3. `read_file` 能按规格读取文本、范围和行号。
4. `load_files_context` 能按清单顺序生成 synthetic `read_file` call/output pairs。
5. synthetic pairs 作为原生 `ResponseItem` 写入 model-visible history。
6. 下一次 Responses request 的 `input` 中能观察到这些 pairs。
7. 单个目标文件读取失败表现为对应 `read_file` output 失败，且后续路径继续展开。
8. `load_files_context` 自身 output 不承载文件正文。
9. 多文件正文没有通过 stdout、JSON 数组、Markdown/XML 容器聚合。
10. 所有验收测试通过。

## 14. 总结

`read_file`：

```text
给定路径和可选行范围，返回一个文件的文本。
```

`load_files_context`：

```text
给定路径行序列，按序生成普通 read_file 工具调用轨迹，并写入 Responses history。
```

这两个对象共同解决的问题是：让 Codex 模型以原生 Responses 工具边界阅读确定文件集合，避免 shell stdout 聚合和模型机械生成大量读取调用。


# 先聊清楚：这件事的核心不是“改 prompt”，而是把 Codex runtime 和 Codex 默认 prompt 解耦

我现在对你的目标已经收敛到一个非常明确的对象：**让 Codex runtime 和 Codex 默认 prompt 解耦**。你想复用 Codex 的运行时能力，但不想被 Codex 作为“coding agent 产品”时默认注入的那套模型可见世界绑定住。

这个目标和“自己写 Python 调 Responses API”之间的关系很清楚。你自己写 Python 的时候，模型输入完全由你决定：`instructions` 是你写的，`input` 是你放进去的，`tools` 是你传进去的。你可以让它只有一句 `You are a helpful assistant.` 加当前用户消息。这个输入边界非常干净。问题是 Python 脚本只解决了模型请求本身，没有 Codex 已经实现好的 runtime：没有 TUI/CLI 交互，没有现成的登录和 provider 配置，没有 Codex 的流式渲染，没有 thread 管理，没有工具调用循环，没有已有的错误处理、模型选择、rollout 记录、MCP/工具接入能力。你不想重新造这些轮子。

所以正确方向不是“放弃 Codex 写一个新客户端”，而是把 Codex 拆成两层看：第一层是 runtime shell，负责请求生命周期、流式、会话、工具执行、记录、UI；第二层是 model-visible default world，也就是 Codex 默认给模型看的身份、规则、工具说明、环境说明、权限说明、协作模式、各种内置提示词。你要复用第一层，控制第二层。

源码事实也支持这个切分。最终发给 Responses API 的请求里，`instructions` 和 `input`、`tools` 是分开的。`Prompt.base_instructions` 进入请求的顶层 `instructions`；`build_initial_context` 聚合出 developer/user context items；`ToolRouter::model_visible_specs()` 进入 `Prompt.tools`。这三个点共同构成模型能看到的初始世界。Codex runtime 本身的 thread id、rollout path、token usage、transport state、previous_response_id 等不是你要控制的对象，因为模型不直接看到它们。你要控制的是模型可见内容。

你现在进一步把范围限定得很合理：先不管外部来源。AGENTS.md 是外部文件，MCP 是外部 server，skills 是外部目录或插件提供的内容，用户 config 也已经有自己的配置入口。它们虽然也可能产生噪声，但它们不是“内置不可改”的核心痛点。真正不可控的是编译进 Rust 二进制或由 Rust 固定渲染的内置内容：model catalog 里的 base instructions、fallback prompt、permissions/sandbox 模板、collaboration mode 模板、ContextualUserFragment 里写死的说明、内置工具的 description 和参数 schema 描述、apply_patch grammar 等。你要先把这些不可控的内置内容变成项目级可控数据。

因此这份设计文档的对象不是“全部上下文控制平台”，而是 **Builtin Context Lock**：一个项目级文件，用来覆盖 Codex 内置的模型可见内容。配置只需要指定这个文件路径；命令只需要生成这个文件；UI 不是 Codex 必需能力，只是以后可以消费和编辑这个文件的普通外部工具。

这个设计的关键纪律是：只对当前限定对象负责。不要把未来 UI 的 diff 体验塞进 lock schema，不要把 MCP/skills/AGENTS 的原生控制重新发明一遍，不要支持工具改名和 handler 映射，不要引入一堆 mode。文件里出现的内置条目由 lock 接管；enabled 就使用 lock 内容；disabled 就压制这个内置条目；未出现的内置条目保持原行为。外部来源保持原机制。

这样它能直接支持你要的最纯净场景：项目配置 builtin lock，lock 里把 base instructions 改成 `You are a helpful assistant.`，把所有内置 fragments 禁用，把所有内置 tools 禁用，同时你自己不启用外部 AGENTS/MCP/skills。此时 Codex runtime 仍然是 Codex，但模型看到的请求可以接近你手写 Python Responses 脚本的形态：一个 instructions，加当前 user message，tools 为空。

下面是正式设计文档。我把它写成后续开发者可以直接依据实现的规格，而不是讨论稿。

````markdown
# Builtin Context Lock 设计文档

## 1. 目标定义

Builtin Context Lock 是 Codex 的项目级内置上下文覆盖机制。它允许一个项目通过配置文件指定一份 lock 文件，用该文件接管 Codex 自身内置的模型可见内容。被接管的内容包括 Codex 编译进二进制或由 Rust 固定渲染的 base instructions、内置 context fragments、内置工具的模型可见规格，以及其他由 Codex 内置代码产生的模型可见静态文本。

Builtin Context Lock 的核心目标是让 Codex runtime 和 Codex 默认 prompt 解耦。Codex runtime 继续负责会话、流式响应、工具执行、thread 记录、provider 配置和 CLI/TUI 交互；模型可见的内置默认内容由项目 lock 文件控制。项目可以把 Codex 降噪成一个接近原始 Responses API 客户端的纯净运行环境，也可以只禁用或修改少量内置提示词和工具说明。

该机制只接管 Codex 内置来源。AGENTS.md、MCP server、skills 文件、plugins、用户显式 config、用户消息、模型回复、工具执行结果、thread id、rollout metadata、transport state 不属于 Builtin Context Lock 的控制对象。

## 2. 设计动机

Codex 默认运行时面向 coding-agent 工作流。它会向模型提供大量有助于代码任务的默认上下文和工具说明，例如 coding-agent 身份、编辑约束、sandbox/approval 说明、collaboration mode instructions、apply_patch 工具说明、shell 工具说明、多 agent 工具说明、environment context、skills/plugins/apps 的内置说明框架。这些默认内容在编程任务里是有效能力，在科研、prompt 行为研究、纯对话实验、工具隔离实验中可能是噪声。

用户可以自己写 Python 脚本直接调用 OpenAI Responses API，从而得到完全干净的输入边界：指定 `instructions`，指定 `input`，指定 `tools`。但是这种做法会绕开 Codex 已经实现好的 runtime 能力。Builtin Context Lock 的作用是在不重写 runtime 的前提下，让模型可见的内置默认内容变成项目可控数据。

设计原则是只接管“Codex 内置且模型可见”的内容。外部来源已有原生控制方式，不在该机制中重新建模。该机制不追求运行时审批、不追求最终请求拦截、不追求 UI 状态管理、不追求修改工具调用名和 handler 映射。

## 3. 当前源码事实

Codex 请求构造路径中有三个与模型可见内置内容相关的主要入口。

第一，base instructions。Session 初始化阶段解析 `base_instructions`，优先使用用户配置或历史里的 base instructions，最终退回到当前 model info 的 model instructions。该文本进入 `Prompt.base_instructions`，在 Responses 请求构造阶段作为顶层 `instructions` 字段发送。源码中相关对象包括 `BaseInstructions`、model catalog 的 `base_instructions` 和 `ModelInfo::get_model_instructions()`。

第二，initial context fragments。`Session::build_initial_context` 聚合多个 developer/user context sections，并把它们转换成 request input items。许多片段通过 `ContextualUserFragment` 渲染。该 trait 定义 role、start marker、end marker 和 body。实现者包括 permissions instructions、collaboration mode instructions、environment context、available skills instructions、available plugins instructions、apps instructions、personality spec、model switch、realtime start/end、image generation instructions 等。这里既有外部来源内容，也有 Codex 内置固定文本。Builtin Context Lock 只接管其中内置文本和内置片段开关，不接管外部文件或外部 server 的内容。

第三，tool specs。`ToolRouter::from_config` 调用工具 spec 构建逻辑，得到 `specs` 和 `model_visible_specs`。`build_prompt` 把 `router.model_visible_specs()` 放入 `Prompt.tools`。内置工具的 description、parameters schema description、freeform grammar 等由 Rust 代码或 `include_str!` 固定生成。Builtin Context Lock 需要在内置 tool specs 进入 tool router/model-visible specs 前提供 override 和 disable 能力。

这些入口共同决定模型第一次看到的 Codex 内置世界。Builtin Context Lock 应当在这些入口处工作，而不是在网络层抓包，也不是只修改最终 JSON。

## 4. 非目标

Builtin Context Lock 不控制 AGENTS.md。AGENTS.md 是项目文件，用户可以直接修改、删除或通过现有配置影响其使用。

Builtin Context Lock 不控制 MCP server 的动态工具列表。MCP 是外部来源，用户可以通过 MCP 配置控制 server。后续如果需要锁定 MCP list_tools，可以作为独立机制设计，不属于本功能。

Builtin Context Lock 不控制 skills 文件和 plugins 的外部内容。skills/plugins 的安装、删除、启用和内容修改已有外部文件或配置入口。本功能只可接管 Codex 内置的 skills/plugins 说明框架文本，例如“如何使用 skills/plugins”的内置说明，而不接管 skill 条目本身。

Builtin Context Lock 不控制用户后续输入、模型回复、工具返回、对话历史自然 append、compaction 结果、thread id、request id、rollout path、auth token、transport header、previous_response_id、token usage。

Builtin Context Lock 不提供运行时手动审批，不拦截每次网络请求，不提供 pending request UI。

Builtin Context Lock 不支持修改工具调用名。工具名是模型返回 tool call 和本地 handler dispatch 的绑定点。第一版只支持禁用内置工具，以及修改工具模型可见说明文本和 schema description。工具名和 handler mapping 保持 Codex 原始语义。

Builtin Context Lock 不把 UI 编辑状态写进 lock schema。UI 可以作为独立工具读取和编辑 lock 文件，但 lock 文件本身只表达 Codex runtime 需要读取的当前真值。

## 5. 配置表面

项目配置只需要声明 lock 文件路径。

推荐 TOML 表面：

```toml
[builtin_context_lock]
path = ".codex/builtin-context.lock.json"
```

当 `builtin_context_lock.path` 存在且对应文件可读取时，Codex 对该文件中出现的内置条目启用 lock 接管。当配置不存在时，Codex 行为完全保持原样。

不引入 `enabled` 字段。路径配置本身就是启用声明。

不引入 mode 字段。该机制只有一种语义：文件中出现的内置条目由 lock 管理；文件中未出现的内置条目保持原行为。

路径相对解析规则应与其他项目路径配置保持一致。建议相对当前 effective cwd 或配置文件所在目录解析；实现时必须选择一个现有项目配置路径规则并在文档中固定，不允许在不同调用点使用不同解析基准。

## 6. 命令表面

新增一个生成命令，用于导出当前 Codex 二进制和当前配置下可见的内置 catalog。

推荐命令：

```bash
codex builtin-context-lock generate
```

命令行为：

- 如果项目配置中存在 `builtin_context_lock.path`，默认输出到该路径。
- 如果项目配置中不存在该路径，命令必须要求显式 `--output <path>`，或提示用户先配置路径。第一版可选择其中一种，但行为必须固定。
- 命令只生成 lock 文件，不启用配置，不启动 UI，不写 sidecar，不保存 diff 状态。
- 命令只采集 Codex 内置模型可见内容。它不扫描 AGENTS.md，不调用 MCP list_tools，不读取外部 skill 文件，不导出用户历史。
- 命令可以受当前 feature flags、平台、模型、tool config 影响，因为某些内置工具和内置片段只有在当前配置下才会出现。第一版以“当前配置会产生的内置 catalog”为导出对象。

生成命令的输出是一个普通 JSON 文件。后续 UI 可以读取该 JSON 并提供编辑体验，但 UI 不是该命令的组成部分。

## 7. Lock 文件语义

Lock 文件由若干内置条目组成。每个条目通过稳定 id 绑定到一个 Codex 内置模型可见对象。

条目运行时语义：

- 条目存在且 `enabled = true`：Codex 使用 lock 中的内容或 spec 代替原内置内容。
- 条目存在且 `enabled = false`：Codex 压制该内置条目，使其不进入模型可见输入或模型可见工具列表。
- 条目不存在：Codex 不接管该内置条目，原内置逻辑照常运行。

删除条目不是禁用。删除条目表示放弃管理该内置条目，Codex 恢复原行为。禁用必须保留条目并设置 `enabled = false`。

Lock 文件不表达外部来源。外部来源未出现在 lock 中不是“允许外部 fallback”，而是因为外部来源不属于本机制。

## 8. Lock 文件结构

推荐 JSON 顶层结构：

```json
{
  "schema_version": 1,
  "base_instructions": [],
  "fragments": [],
  "tools": [],
  "templates": []
}
```

`schema_version` 是必填整数。第一版值为 `1`。

`base_instructions` 保存 Codex 内置 base/model instructions 条目。

`fragments` 保存 Codex 内置 context fragments 或 fragment-level 开关。

`tools` 保存 Codex 内置工具的模型可见 specs。

`templates` 保存由 `include_str!` 或 Rust 常量提供、会被其他内置渲染逻辑引用的模板。第一版可以把模板并入 fragments；如果实现需要细粒度替换 permissions/sandbox/collaboration 模板，则使用该组。

每个条目必须包含：

```json
{
  "id": "stable.builtin.id",
  "enabled": true
}
```

不同 kind 的条目可包含不同 payload。

Base instructions 条目示例：

```json
{
  "id": "builtin.base_instructions.default_model",
  "enabled": true,
  "content": "You are a helpful assistant."
}
```

Fragment 条目示例：

```json
{
  "id": "builtin.fragment.permissions_instructions",
  "enabled": false
}
```

Tool 条目示例：

```json
{
  "id": "builtin.tool.apply_patch",
  "enabled": false,
  "name": "apply_patch",
  "spec": {
    "type": "custom",
    "name": "apply_patch",
    "description": "Use the `apply_patch` tool to edit files.",
    "format": {
      "type": "grammar",
      "syntax": "lark",
      "definition": "..."
    }
  }
}
```

Tool 条目中的 `name` 必须等于原内置工具名。第一版不支持修改 tool name。实现可在读取 lock 时拒绝不匹配的 name，也可忽略 lock 中的 name 并以 id 对应的原名为准。推荐拒绝不匹配，避免用户误以为改名可用。

## 9. 稳定 ID 规则

每个可接管的内置对象必须有稳定 id。稳定 id 是 lock 文件和源码接入点之间的契约。

推荐 id 前缀：

- `builtin.base_instructions.*`
- `builtin.fragment.*`
- `builtin.template.*`
- `builtin.tool.*`

示例 id：

- `builtin.base_instructions.model_catalog.current`
- `builtin.base_instructions.protocol_default`
- `builtin.fragment.permissions_instructions`
- `builtin.fragment.collaboration_mode_instructions`
- `builtin.fragment.environment_context`
- `builtin.fragment.apps_instructions`
- `builtin.fragment.available_plugins_instructions`
- `builtin.fragment.available_skills_scaffold`
- `builtin.fragment.personality_spec`
- `builtin.fragment.model_switch`
- `builtin.fragment.image_generation`
- `builtin.fragment.realtime_start`
- `builtin.fragment.realtime_end`
- `builtin.tool.exec_command`
- `builtin.tool.write_stdin`
- `builtin.tool.shell`
- `builtin.tool.apply_patch`
- `builtin.tool.spawn_agent`
- `builtin.tool.send_input`
- `builtin.tool.wait_agent`
- `builtin.tool.get_goal`
- `builtin.tool.create_goal`
- `builtin.tool.update_goal`
- `builtin.tool.update_plan`
- `builtin.tool.request_user_input`
- `builtin.tool.list_mcp_resources`
- `builtin.tool.read_mcp_resource`

The exact inventory must be generated from the current codebase before implementation. The list above is a seed, not an exhaustive contract.

ID names must refer to semantic builtin objects, not source file paths. Source file paths may change; builtin object identity should remain stable when the model-visible object remains the same.

## 10. Base Instructions 接入规则

Builtin Context Lock 接管的是 Codex 内置 base instructions fallback，不覆盖用户显式配置。

推荐优先级：

1. 用户显式 `instructions` / `model_instructions_file` / config-level base override。
2. 恢复线程历史中的 base instructions。
3. Builtin Context Lock 中启用的 base instructions 条目。
4. Codex 原始 model catalog / fallback base instructions。

该优先级保持用户显式配置高于 builtin lock。Builtin lock 的定位是替代 Codex 内置默认值，而不是抢占用户配置。

如果 lock 中对应 base instructions 条目存在且 `enabled = true`，并且没有更高优先级的用户显式 base override，则使用 `content`。

如果 lock 中对应 base instructions 条目存在且 `enabled = false`，并且没有更高优先级的用户显式 base override，则 base instructions 使用空字符串或该路径允许的最小空值。实现必须选择一个固定行为。推荐使用空字符串，让用户可以构造无内置 instructions 的请求。

如果 lock 中没有对应 base instructions 条目，则原逻辑不变。

## 11. Fragment 接入规则

Builtin fragments 是由 Codex 内置代码生成并进入 model input 的 developer/user context items。它们可能来自 `ContextualUserFragment` 实现，也可能来自 session assembly 中的固定 developer section。

运行时接入规则：

- 在每个内置 fragment 渲染或加入 section 前，查询 builtin lock 对应 id。
- 找到 `enabled = false`：不加入该 fragment。
- 找到 `enabled = true` 且条目包含 `content`：使用 lock content 构造相同 role 的 message fragment。
- 找到 `enabled = true` 但条目只表达开关：继续使用原渲染结果。
- 未找到条目：原逻辑不变。

带 runtime 参数的 fragment 第一版可以只支持禁用，不支持模板编辑。示例：environment context、image generation path、model switch instructions、realtime end reason。这些 fragment 的动态值不是 builtin lock 的核心对象。若后续需要编辑模板，应单独定义 placeholder 规则，不在第一版隐式支持。

纯静态内置 fragment 可以支持 `content` override。示例：apps instructions、available plugins scaffold、某些 warning text。

Fragment role 必须保持原实现角色。第一版不支持通过 lock 改 role。

## 12. Tool 接入规则

Builtin tool specs 是 Codex 内置工具的模型可见定义。它们包括 tool name、description、parameters schema、schema field descriptions、freeform format definition 等。

运行时接入规则：

- 内置工具 spec 被创建后、进入 tool registry/model_visible_specs 前，应用 builtin lock。
- 找到对应 tool 条目且 `enabled = false`：该内置工具不进入 model-visible specs，也不应作为模型可调用工具注册。Rust handler 可以仍然存在于代码中，但本 session 的模型可见工具世界没有该工具。
- 找到对应 tool 条目且 `enabled = true`：使用 lock 中的 spec 替代原 spec 的模型可见部分。
- 未找到对应 tool 条目：原内置 spec 原样使用。

第一版不支持修改工具名。工具名必须与原内置工具名一致。

第一版不支持修改 handler routing。工具执行能力仍由 Codex 原始 handler 提供。

如果 tool 条目 enabled 且 spec 缺少必要字段，读取 lock 时应报错并停止使用该 malformed 条目。实现可选择使整个 session 初始化失败，或记录错误后对该条目回退原逻辑。为了避免 silent prompt 差异，推荐 session 初始化失败并显示明确错误。

MCP tools、dynamic tools、extension tools 不属于 builtin tool lock，除非它们通过 Codex 内置工具包装成固定 builtin tool。普通外部 tool source 保持原行为。

## 13. Templates 接入规则

Templates 是 Codex 内置 Markdown 或 Rust 字符串模板。示例包括 permissions approval policy templates、sandbox mode templates、realtime templates、collaboration mode templates、apply_patch grammar。

如果某个模板只服务于一个 fragment 或 tool，第一版可以通过 fragment/tool 条目接管最终模型可见结果，而不必单独暴露模板条目。

如果多个运行路径复用同一模板，或用户明确需要细粒度修改该模板，则可暴露 `templates` 条目。

Template 条目语义：

- `enabled = true`：使用 lock content 替代内置模板。
- `enabled = false`：该模板产生的内置 fragment/tool text 被压制，具体压制行为由使用该模板的 source 定义。
- 未出现：原模板不变。

第一版应优先实现 fragment-level 和 tool-level 接管。Template-level 接管可以作为实现便利或后续扩展，但不能成为完成 pure mode 的必要条件。

## 14. Pure Responses 使用场景

Builtin Context Lock 必须支持把 Codex runtime 降到接近手写 Responses API 脚本的纯净输入。

目标请求形态：

```json
{
  "instructions": "You are a helpful assistant.",
  "input": [
    { "role": "user", "content": "你好，你可以帮我做什么？" }
  ],
  "tools": []
}
```

达到该形态需要：

- builtin lock 接管 base instructions，并设置为 `You are a helpful assistant.`。
- builtin lock 禁用所有内置 fragments。
- builtin lock 禁用所有内置 tools。
- 项目不启用外部 AGENTS/MCP/skills/plugins/environment context，或通过现有配置关闭它们。

Builtin Context Lock 只负责前三项。外部来源控制不属于该 lock。

## 15. 生成命令的 catalog 来源

生成命令应从 Codex 当前二进制和当前配置生成 builtin catalog。它应调用或复用现有内置构造逻辑，避免手写第二套列表。

生成 base instructions 时，应导出当前配置下会作为内置 fallback 的 base instructions，以及必要的默认/fallback条目。

生成 fragments 时，应导出当前配置下可能由 Codex 内置渲染的 fragment 条目。对于带 runtime 参数的 fragment，生成命令可导出 disabled-capable entry，不必导出完整 runtime content。

生成 tools 时，应导出当前配置下内置 tool specs。外部 MCP/dynamic/extension tools 不导出。

生成文件的默认 enabled 状态应反映 Codex 当前默认行为。默认会出现的内置条目 enabled=true；当前配置下不会出现但仍属于可管理 inventory 的条目是否导出，由实现决定。第一版建议只导出当前会出现的内置条目，以减少无关对象。

## 16. 错误处理

配置了 `builtin_context_lock.path` 但文件不存在、不可读或 JSON 无法解析时，Codex 应在 session 初始化阶段报错。路径配置表示用户要求使用该 lock；静默忽略会让模型输入不可预测。

Lock 文件 schema_version 不支持时，Codex 应报错并说明支持的版本。

条目 id 未知时，Codex 可以忽略该条目并发出 warning。未知 id 不应导致 session 失败，因为 lock 文件可能由较新版本生成。若用户希望强校验，可通过独立工具实现；核心 runtime 不需要引入额外模式。

已知条目 payload malformed 时，Codex 应报错。已知 id 表示用户正在接管一个明确内置对象，payload 错误会导致模型可见内容不确定。

Tool name 与 builtin id 不匹配时应报错。第一版不支持 tool rename。

## 17. 测试要求

必须添加 pure mode 回归测试。

测试设置：

- 创建临时项目 config，指定 builtin lock path。
- lock 中 base instructions 为 `You are a helpful assistant.`。
- lock 禁用所有当前内置 tool 条目。
- lock 禁用所有当前内置 fragment 条目。
- 外部来源不启用。
- 构造一轮 user message。
- 捕获构造出的 `Prompt` 或 `ResponsesApiRequest`。

断言：

- `instructions` 等于 `You are a helpful assistant.`。
- `tools` 为空。
- `input` 不包含 permissions/collaboration/environment/skills/plugins/apps 等内置 fragment。
- `input` 至少包含当前用户消息。

必须添加内置工具单项禁用测试。

- lock 禁用 `builtin.tool.apply_patch`。
- 构造 tool router。
- 断言 `model_visible_specs()` 不包含 `apply_patch`。
- 断言其他未管理内置工具保持原行为。

必须添加内置工具 description override 测试。

- lock 接管某个工具并修改 description。
- 构造 tool router。
- 断言模型可见 spec 使用 lock description。
- 断言 tool name 未变。

必须添加 base instructions 优先级测试。

- 同时配置用户显式 `instructions` 和 builtin lock base instructions。
- 断言用户显式配置优先。
- 仅配置 builtin lock 时，断言 lock base instructions 生效。

必须添加 malformed lock 测试。

- 无法解析 JSON 报错。
- 不支持 schema_version 报错。
- 已知 tool id 的 name mismatch 报错。

## 18. 实现接入建议

实现应新增一个小型 builtin lock 解析模块，负责：

- 读取配置路径。
- 解析 lock JSON。
- 建立按 id 查询的 map。
- 提供 typed helper：`base_instruction_override(id)`、`fragment_decision(id)`、`tool_decision(id)`。

不要让各调用点直接解析 JSON。

Base instructions 接入点应靠近 session 解析 base instructions 的位置。目标是只替换内置 fallback，不覆盖用户显式配置。

Fragment 接入点应靠近 `build_initial_context` 中 push 内置 fragment 的位置，或在各 `ContextualUserFragment::render()` 调用外包一层 helper。不要修改外部 contributor、AGENTS、MCP、skills 内容的原始加载逻辑。

Tool 接入点应位于内置 tool specs 构造后、tool router build 完成前。理想位置是工具 spec plan 或 registry builder 层，使 builtin lock 可以过滤和替换内置 specs，同时不影响 MCP/dynamic/extension specs。

生成命令应复用同一 inventory 和 id 定义。运行时和生成时不能维护两套 id 字符串。

## 19. UI 边界

UI 不是 Builtin Context Lock 的必需能力。任何 UI 都应作为外部编辑器消费 lock JSON。Codex core 不保存 UI 状态，不保存 original/effective 双份，不保存 diff 信息，不管理 UI profiles。

UI 可以提供开关、文本编辑、JSON schema 校验、复制文件、diff、profile 管理，但这些属于 UI 工程，不属于 runtime lock contract。

Builtin Context Lock 的 contract 是文件格式和运行时接管语义。

## 20. 接受标准

该功能完成后，用户能够执行以下工作流：

1. 在项目中配置 builtin lock path。
2. 运行生成命令得到一份 lock 文件。
3. 手动编辑 lock，把 base instructions 改成 `You are a helpful assistant.`。
4. 禁用所有内置 fragments。
5. 禁用所有内置 tools。
6. 启动 Codex 并发送一条普通 user message。
7. 捕获或测试最终 request，确认模型可见内容接近纯 Responses 请求：指定 instructions、当前 user message、空 tools。

该功能完成后，默认项目不配置 builtin lock 时，Codex 行为必须保持不变。

该功能完成后，外部 AGENTS/MCP/skills/plugins 的控制方式保持原样，不因 builtin lock 机制被重写。

该功能完成后，内置工具名不可通过 lock 修改。禁用和说明文本修改是第一版支持的工具控制能力。

## 21. 设计摘要

Builtin Context Lock 是 Codex builtin model-visible catalog 的项目级 override。它不替代 Codex runtime，不替代外部配置系统，不拦截网络请求。它把 Codex 编译内置的 prompt、fragment 和 tool spec 从不可改默认值变成项目可声明数据。

该机制的核心价值是让 Codex runtime 和 Codex 默认 prompt 解耦。用户可以复用 Codex runtime，同时构造纯净、可复现、可控的模型输入边界。
````

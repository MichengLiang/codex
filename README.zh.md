<p align="center"><code>npm i -g @openai/codex</code><br />或 <code>brew install --cask codex</code></p>
<p align="center"><a href="./README.md">English</a></p>
<p align="center"><strong>Codex CLI</strong> 是 OpenAI 的本地编码 agent。</p>
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
如果要在代码编辑器中使用 Codex（VS Code、Cursor、Windsurf），请<a href="https://developers.openai.com/codex/ide">安装 IDE 扩展</a>。
</br>如果要使用桌面应用体验，请运行 <code>codex app</code>，或访问 <a href="https://chatgpt.com/codex?app-landing-page=true">Codex App 页面</a>。
</br>如果要使用 OpenAI 的云端 agent <strong>Codex Web</strong>，请访问 <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>。</p>

---

## 快速开始

### 安装并运行 Codex CLI

使用偏好的包管理器全局安装：

```shell
# 使用 npm 安装
npm install -g @openai/codex
```

```shell
# 使用 Homebrew 安装
brew install --cask codex
```

安装后运行 `codex` 即可开始。

<details>
<summary>也可以前往 <a href="https://github.com/openai/codex/releases/latest">GitHub 最新 Release</a> 下载对应平台的 binary。</summary>

每个 GitHub Release 包含多个可执行文件。通常需要下载以下文件之一：

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64（较旧 Mac 硬件）: `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

每个归档中包含一个带平台名的单一可执行文件，例如 `codex-x86_64-unknown-linux-musl`。解压后通常需要将它重命名为 `codex`。

</details>

### 使用 ChatGPT 订阅登录 Codex

运行 `codex` 并选择 **Sign in with ChatGPT**。建议登录 ChatGPT 账号，以便通过 Plus、Pro、Business、Edu 或 Enterprise 订阅使用 Codex。[了解 ChatGPT 订阅包含的 Codex 能力](https://help.openai.com/en/articles/11369540-codex-in-chatgpt)。

也可以使用 API key 运行 Codex，但需要[额外配置](https://developers.openai.com/codex/auth#sign-in-with-an-api-key)。

## MCP Text Contract

MCP Text Contract 是用户配置 MCP server 的 opt-in adapter contract。它让外部 MCP 工具对齐模型可见的 Responses 语义，同时保持 MCP 执行协议不变。MCP server 可以继续作为外部进程存在，并获得更接近 Codex 内置工具的文本输出和 custom/freeform 输入表面。

在单个 MCP server 上启用：

```toml
[mcp_servers.<name>]
model_content_only = true
mcp_freeform = true
```

### 模型可见输出

`model_content_only = true` 时，MCP 工具写回模型上下文的输出等于第一个 MCP text content item：

```text
model_visible_output = CallToolResult.content[0].text
```

模型可见输出不包含 `structuredContent`、`isError`、`_meta`、MCP `content` wrapper JSON、Codex wall-time header 或 `Output:` header。空的 `content[0].text` 保持为空模型输出。Codex 不为空输出或非 text MCP content 生成诊断文本。

完整 MCP result 仍保留给宿主侧消费者。Codex 事件、hook、trace、code-mode result 和运行时诊断仍可观察协议形状的 `CallToolResult`，包括 `structuredContent`、`isError` 和 `_meta`。该投影只控制写回模型上下文的内容。

这个规则允许 MCP server 分离两个结果通道：

- `content[0].text` 放模型应阅读的文本。
- `structuredContent` 放宿主、UI、日志、hook、trace 或自动化流程应读取的结构化数据。

### Freeform MCP 工具

`mcp_freeform = true` 时，直接暴露的 MCP tool 如果 input schema 精确表达唯一 required 字符串字段 `freeform`，Codex 会把它作为 Responses custom/freeform tool 暴露给模型：

```json
{
  "type": "object",
  "properties": {
    "freeform": {
      "type": "string"
    }
  },
  "required": ["freeform"]
}
```

匹配的 MCP tool 会使用 canonical MCP display name 作为顶层 Responses custom tool 名称，例如：

```text
mcp__server__tool_name
```

模型写入 raw custom input。Codex 执行 MCP 调用前，将该 raw input 包装回标准 MCP arguments：

```text
Responses custom input -> MCP arguments { "freeform": input }
```

MCP server 不会收到裸字符串。它仍然收到普通 MCP `tools/call` arguments object。

freeform 映射采用精确匹配。Schema-level `description` / `title` annotation 和 `freeform` property 上的 `description` / `title` annotation 可以存在；其中 `freeform.description` 会合并到 custom tool description，因为 custom tool 不向模型暴露参数 schema。若 schema 增加第二个输入字段、使用 `freeform` 以外的字段名、让 `freeform` 不是 string、给字符串加入额外约束，或显式允许 additional properties，该工具保持普通 MCP function tool。

### 作用范围

该 contract 覆盖用户配置 MCP server 中直接暴露的工具。它不会把所有 MCP 工具都转换为 freeform 工具，不修复没有提供 `content[0].text` 的 MCP server，不定义多模态 MCP rendering，也不通过 code-mode nested tool prompt 或 deferred tool discovery 暴露 MCP freeform 工具。

完整 contract、实现状态和验证记录见 [`micheng/Codex MCP Text Contract/`](<micheng/Codex MCP Text Contract/>)。

## 文档

- [**Codex 文档**](https://developers.openai.com/codex)
- [**贡献指南**](./docs/contributing.md)
- [**安装与构建**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

本仓库使用 [Apache-2.0 License](LICENSE)。

<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install --cask codex</code></p>
<p align="center"><a href="./README.zh.md">中文</a></p>
<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Codex CLI

Install globally with your preferred package manager:

```shell
# Install using npm
npm install -g @openai/codex
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `codex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Business, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## MCP Text Contract

MCP Text Contract is an opt-in adapter contract for user-configured MCP servers. It aligns external MCP tools with the model-visible Responses surface while keeping MCP execution unchanged. An MCP server can stay as an external process and still use the same text-output and custom/freeform input semantics that otherwise require tighter Codex integration.

Enable the contract per MCP server:

```toml
[mcp_servers.<name>]
model_content_only = true
mcp_freeform = true
```

### Model-Visible Output

`model_content_only = true` makes the model-visible output of an MCP tool equal to the first MCP text content item:

```text
model_visible_output = CallToolResult.content[0].text
```

The model-visible output does not include `structuredContent`, `isError`, `_meta`, the MCP `content` wrapper JSON, Codex wall-time headers, or `Output:` headers. Empty `content[0].text` remains an empty model-visible output. Codex does not synthesize diagnostic text for empty or non-text MCP content.

The full MCP result is still retained for host-side consumers. Codex events, hooks, traces, code-mode results, and runtime diagnostics can still observe the protocol-shaped `CallToolResult`, including `structuredContent`, `isError`, and `_meta`. The projection only controls what is written back into the model context.

This lets an MCP server separate its two result channels:

- Put model-readable text in `content[0].text`.
- Put host-readable structured data in `structuredContent`.

### Freeform MCP Tools

`mcp_freeform = true` lets directly exposed MCP tools use Responses custom/freeform input when their input schema is exactly one required string field named `freeform`:

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

A matching MCP tool is declared to the model as a top-level Responses custom tool, using its canonical MCP display name, such as:

```text
mcp__server__tool_name
```

The model writes raw custom input. Codex wraps that raw input back into standard MCP arguments before execution:

```text
Responses custom input -> MCP arguments { "freeform": input }
```

The MCP server never receives a bare string. It still receives the normal MCP `tools/call` arguments object.

The freeform mapping is exact by design. Schema-level `description` / `title` annotations and `freeform` property `description` / `title` annotations are allowed; the `freeform.description` text is merged into the custom tool description because custom tools do not expose parameter schemas to the model. A tool remains an ordinary MCP function tool when its schema adds another input field, uses a field name other than `freeform`, makes `freeform` non-string, adds string constraints, or explicitly allows additional properties.

### Scope

The contract covers directly exposed tools from user-configured MCP servers. It does not turn every MCP tool into a freeform tool, does not repair MCP servers that do not provide `content[0].text`, does not define multimodal MCP rendering, and does not expose MCP freeform tools through code-mode nested tool prompts or deferred tool discovery.

See [`micheng/Codex MCP Text Contract/`](<micheng/Codex MCP Text Contract/>) for the full contract, implementation status, and verification notes.

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).

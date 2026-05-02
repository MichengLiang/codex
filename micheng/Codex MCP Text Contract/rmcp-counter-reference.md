# rmcp Counter Reference

## Purpose

This reference records the local `rmcp 1.6.0` counter experiment that supports the Codex MCP Text Contract. The experiment is not part of the implementation plan. It is evidence for the MCP result shapes that the contract consumes.

## Source Experiment

- Path: `/home/t103o/workbench/examples/temporary/rust-mcp-counter`
- SDK crate: `rmcp = 1.6.0`
- Transport: stdio child process
- Server entry: `src/bin/counter_server.rs`
- Inspect client: `src/bin/inspect_client.rs`
- Core server: `src/lib.rs`

## Verified Commands

Run from `/home/t103o/workbench/examples/temporary/rust-mcp-counter`:

```bash
cargo test
cargo run --bin inspect-client
```

Latest local verification:

```text
cargo test: 5 passed; 0 failed
cargo run --bin inspect-client: exited 0
```

## Tool Surface

The experiment exposes five MCP tools:

- `counter`: returns Markdown text in `content[0].text`.
- `get_count`: returns plain text in `content[0].text`.
- `set_count`: accepts JSON arguments and returns plain text in `content[0].text`.
- `counter_report`: returns `Json<CounterReport>`.
- `counter_pretty_report`: returns pretty Markdown in `content[0].text` and the same report as `structuredContent`.

The inspect client observed:

```text
tool counter output_schema = false
tool counter_pretty_report output_schema = true
tool counter_report output_schema = true
tool get_count output_schema = false
tool set_count output_schema = false
```

## Result Shape Evidence

Plain text tools return one text content item and no `structuredContent`:

```json
{
  "content": [
    {
      "type": "text",
      "text": "1"
    }
  ],
  "isError": false
}
```

`Json<T>` tools return structured JSON and also place JSON text in `content[0].text`:

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

Pretty dual-channel tools can put model-readable Markdown in `content[0].text` while keeping machine-readable JSON in `structuredContent`:

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

## Contract Relevance

The experiment supports three contract claims:

1. MCP tool results can provide a stable text surface at `content[0].text`.
2. The same result can carry host-readable `structuredContent` without making it the model-readable text.
3. A content-only Codex adapter can project `content[0].text` without changing the MCP wire result.

The experiment does not cover MCP freeform input. Freeform input still needs Codex-side tests for tool declaration, custom call routing, and `{ "freeform": raw_input }` argument wrapping.

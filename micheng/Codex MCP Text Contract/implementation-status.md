# Codex MCP Text Contract Implementation Status

## Status

The `feature/mcp-text-contract` branch contains the first implementation of the MCP Text Contract. The branch is suitable for continuing feature work from a frozen implementation baseline, subject to the residual verification gaps listed in this document.

The implementation is not described as mathematically complete. The external contract has direct evidence for its two central behaviors, and the remaining boundary behaviors are covered by focused implementation tests and code review rather than by the current black-box harness.

## Branch And Commits

Current branch:

```text
feature/mcp-text-contract
```

Relevant committed sequence:

```text
c9caa71aab docs: freeze MCP text contract design
30d74bc034 feat: add MCP text contract opt-ins
46894618a7 fix: tighten MCP freeform routing boundaries
eddbeb38ae fix: scope MCP freeform routing to direct exposure
ec97a4be4c docs: add MCP text contract review report
```

The process review report created in `ec97a4be4c` is superseded by this status document. Its durable information is captured here; the report itself is not part of the long-lived contract record.

## Implemented Contract Surface

### Configuration

`McpServerConfig` accepts two server-level opt-in fields:

```toml
model_content_only = true
mcp_freeform = true
```

Both fields default to `false`. Both fields are present in the generated config schema. Unknown-field rejection remains part of the raw MCP server config behavior.

### Model Content Projection

When `model_content_only = true` for an MCP server, the model-visible tool output is projected from:

```text
CallToolResult.content[0].text
```

The model-visible output does not include:

- `structuredContent`
- `isError`
- `_meta`
- MCP `content` wrapper JSON
- Codex wall-time header
- `Output:` header

The host-side result remains the full MCP result. Codex JSON events, traces, hooks, and host consumers can still observe protocol-shaped result data.

### MCP Freeform Tool Exposure

When `mcp_freeform = true` for an MCP server, a directly exposed MCP tool is converted to a Responses custom/freeform tool only when its input schema is exactly:

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

The Responses custom tool name is the MCP canonical display name, for example:

```text
mcp__tiny_contract__freeform_probe
```

When the model emits a `custom_tool_call`, Codex routes it back to MCP execution and sends standard MCP tool arguments:

```json
{ "freeform": "<raw custom input>" }
```

The raw string is not sent to the MCP server as a bare string, and it is not wrapped as `{ "input": ... }`.

## Routing Boundaries

The current implementation scopes MCP freeform custom-call routing to tools that are directly exposed in the current turn as Responses custom/freeform tools.

The router does not resolve arbitrary MCP inventory by custom tool name. This prevents a model custom call from reaching a deferred MCP tool, a non-exposed MCP tool, or a same-shaped tool outside the direct Responses tool surface.

Code-mode nested MCP freeform exposure is not part of the first implementation. This matches the design document boundary.

Plugin MCP policy overlay does not expose these fields in the first implementation. User-configured MCP servers are the covered surface.

## Black-Box Verification

### Harness

The black-box verification used a temporary local harness under:

```text
/home/t103o/workbench/projects/codex/tmp/mcp-text-contract-blackbox
```

The harness used:

- `/home/t103o/workbench/experiments/codex-scenario-server`
- an isolated `CODEX_HOME`
- the current Rust Codex CLI through `cargo run -p codex-cli -- exec`
- a tiny stdio MCP server at `tmp/mcp-text-contract-blackbox/tiny_mcp_server.py`
- a local proxy that recorded raw `/v1/responses` request bodies

The scenario server YAML does not currently expose a `namespace` field for function calls. The proxy injected the namespace only for the content-only function-call test so that the scenario could exercise the same Responses namespace shape that Codex receives from real models.

### Content-Only Evidence

The tiny MCP server returned a result containing both model text and structured data:

```json
{
  "content": [
    {
      "type": "text",
      "text": "TEXT_ONLY::GAMMA\n"
    }
  ],
  "structuredContent": {
    "value": "GAMMA",
    "hidden": "STRUCTURED_SHOULD_NOT_REACH_MODEL"
  }
}
```

Codex JSON events confirmed that the MCP tool executed as:

```text
server = tiny_contract
tool = text_probe
arguments = { "value": "GAMMA" }
```

The next Responses request contained:

```json
{
  "type": "function_call_output",
  "call_id": "call-tiny-text",
  "output": "TEXT_ONLY::GAMMA\n"
}
```

The captured model input did not contain `structuredContent`, the MCP `content` wrapper, `Wall time`, or `Output:`.

Captured request bodies were preserved under:

```text
tmp/mcp-text-contract-blackbox/raw-requests-content-only/
```

### Freeform Evidence

The tiny MCP server exposed an exact freeform schema:

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

The first Responses request declared the tool as:

```json
{
  "type": "custom",
  "name": "mcp__tiny_contract__freeform_probe",
  "description": "Exact freeform string tool for MCP custom tool routing tests.",
  "format": {
    "type": "text",
    "syntax": "text",
    "definition": ""
  }
}
```

The same request still exposed the non-freeform MCP tool as a namespace function tool:

```json
{
  "type": "namespace",
  "name": "mcp__tiny_contract__",
  "tools": [
    {
      "type": "function",
      "name": "text_probe"
    }
  ]
}
```

The scenario emitted:

```json
{
  "type": "custom_tool_call",
  "call_id": "call-tiny-freeform",
  "name": "mcp__tiny_contract__freeform_probe",
  "input": "DELTA_PAYLOAD"
}
```

Codex JSON events confirmed that the call was routed to MCP:

```text
server = tiny_contract
tool = freeform_probe
arguments = { "freeform": "DELTA_PAYLOAD" }
```

The next Responses request contained:

```json
{
  "type": "custom_tool_call_output",
  "call_id": "call-tiny-freeform",
  "output": "FREEFORM_ECHO::DELTA_PAYLOAD"
}
```

Captured request bodies were preserved under:

```text
tmp/mcp-text-contract-blackbox/raw-requests-freeform/
```

## Real Docutouch Observation

The real `docutouch` MCP server was also exercised through Codex and the scenario server. It successfully validated the content-only integration path for a normal MCP function tool.

`docutouch` did not validate the freeform path because its `apply_patch` tool currently declares:

```json
{
  "type": "object",
  "properties": {
    "patch": {
      "type": "string"
    }
  },
  "required": ["patch"]
}
```

That schema is intentionally outside this contract. The contract recognizes only the exact required field named `freeform`.

## Focused Review Evidence

The focused review verdict was approve with no blocking findings. The durable test evidence from that review was:

```bash
cargo test -q -p codex-core custom_tool_call_does_not_route_to_deferred_exact_mcp_freeform_tool --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml
cargo test -q -p codex-core stdio_server_non_exact_freeform_like_custom_call_falls_back_to_plain_custom_tool --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml
cargo test -q -p codex-core code_mode_does_not_expose_mcp_freeform_tools_on_global_tools_object --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml
cargo test -q -p codex-config config_schema_includes_mcp_text_contract_flags --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml
```

Those commands passed during review. The full workspace test suite was not run as part of that focused review.

## Residual Verification Gaps

The following items are not proven by the current black-box harness:

- `isError = true` with `content[0].text` projects only the text to the model while host consumers retain error state.
- Empty `content[0].text` projects to an empty model-visible output without generated diagnostic text.
- Default behavior remains unchanged when `model_content_only` and `mcp_freeform` are omitted or set to `false`.
- Config unknown-field rejection remains intact for misspelled MCP text contract fields.

These items should be treated as boundary acceptance candidates before broad integration or upstream submission. They do not block local continuation from this branch, but they are not black-box closed.

## Working Tree Policy

Temporary black-box artifacts live under:

```text
tmp/mcp-text-contract-blackbox/
```

They are evidence artifacts for local inspection, not source artifacts. The repository should ignore `tmp/` so that scenario captures, isolated Codex homes, proxy scripts, and local MCP probes do not enter commits by accident.

The long-lived documentation set in `micheng/Codex MCP Text Contract/` is:

```text
Codex MCP Text Contract 设计文档.md
implementation-status.md
rmcp-counter-reference.md
```

The process review report is intentionally removed after this status document absorbs its durable information.

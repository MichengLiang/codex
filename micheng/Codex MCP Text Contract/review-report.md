# Codex MCP Text Contract Review Report

## Verdict

Approve

## Findings

No blocking findings.

## Tests Run

- `git -C /home/t103o/workbench/projects/codex show --stat --oneline eddbeb38aef65cb791d601cf5c50a723425635a1`
  - Result: reviewed the final follow-up scope for MCP freeform routing.
- `cargo test -q -p codex-core custom_tool_call_does_not_route_to_deferred_exact_mcp_freeform_tool --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml`
  - Result: passed.
- `cargo test -q -p codex-core stdio_server_non_exact_freeform_like_custom_call_falls_back_to_plain_custom_tool --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml`
  - Result: passed.
- `cargo test -q -p codex-core code_mode_does_not_expose_mcp_freeform_tools_on_global_tools_object --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml`
  - Result: passed.
- `cargo test -q -p codex-config config_schema_includes_mcp_text_contract_flags --manifest-path /home/t103o/workbench/projects/codex/codex-rs/Cargo.toml`
  - Result: passed.

## Residual Risks

- I did not run the full workspace test suite. This final review stayed focused on the MCP text contract surfaces and the previously reported routing/config/code-mode regressions.

## Notes

- Previous finding 1 is now closed. `ToolRouter` builds `direct_mcp_freeform_tools_by_custom_name` from `ToolRouterParams.mcp_tools` and the actually registered `ToolSpec::Freeform` set, then resolves `CustomToolCall` through that direct-exposed map instead of through full MCP inventory. The new router test also proves a deferred exact freeform tool is not custom-routed.
- Previous finding 2 remains closed. MCP freeform tools are still excluded from code-mode nested exposure, with focused coverage.
- Previous finding 3 remains closed. The committed `codex-rs/core/config.schema.json` contains both `model_content_only` and `mcp_freeform`.

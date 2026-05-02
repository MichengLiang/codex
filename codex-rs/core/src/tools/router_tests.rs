use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use crate::session::tests::make_session_and_context;
use crate::tools::context::ToolPayload;
use codex_mcp::ToolInfo;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::models::ResponseItem;
use codex_tools::ResponsesApiNamespaceTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use pretty_assertions::assert_eq;
use rmcp::model::JsonObject;
use rmcp::model::Tool;
use serde_json::json;

use super::ToolCall;
use super::ToolRouter;
use super::ToolRouterParams;

#[tokio::test]
#[expect(
    clippy::await_holding_invalid_type,
    reason = "test builds a router from session-owned MCP manager state"
)]
async fn parallel_support_does_not_match_namespaced_local_tool_names() -> anyhow::Result<()> {
    let (session, turn) = make_session_and_context().await;
    let mcp_tools = session
        .services
        .mcp_connection_manager
        .read()
        .await
        .list_all_tools()
        .await;
    let router = ToolRouter::from_config(
        &turn.tools_config,
        ToolRouterParams {
            deferred_mcp_tools: None,
            mcp_tools: Some(mcp_tools),
            unavailable_called_tools: Vec::new(),
            parallel_mcp_server_names: HashSet::new(),
            discoverable_tools: None,
            dynamic_tools: turn.dynamic_tools.as_slice(),
        },
    );

    let parallel_tool_name = ["shell", "local_shell", "exec_command", "shell_command"]
        .into_iter()
        .find(|name| {
            router.tool_supports_parallel(&ToolCall {
                tool_name: ToolName::plain(*name),
                call_id: "call-parallel-tool".to_string(),
                payload: ToolPayload::Function {
                    arguments: "{}".to_string(),
                },
            })
        })
        .expect("test session should expose a parallel shell-like tool");

    assert!(!router.tool_supports_parallel(&ToolCall {
        tool_name: ToolName::namespaced("mcp__server__", parallel_tool_name),
        call_id: "call-namespaced-tool".to_string(),
        payload: ToolPayload::Function {
            arguments: "{}".to_string(),
        },
    }));

    Ok(())
}

#[tokio::test]
async fn build_tool_call_uses_namespace_for_registry_name() -> anyhow::Result<()> {
    let (session, turn) = make_session_and_context().await;
    let session = Arc::new(session);
    let tool_name = "create_event".to_string();

    let router = ToolRouter::from_config(
        &turn.tools_config,
        ToolRouterParams {
            deferred_mcp_tools: None,
            mcp_tools: None,
            unavailable_called_tools: Vec::new(),
            parallel_mcp_server_names: HashSet::new(),
            discoverable_tools: None,
            dynamic_tools: turn.dynamic_tools.as_slice(),
        },
    );
    let call = router
        .build_tool_call(
            &session,
            ResponseItem::FunctionCall {
                id: None,
                name: tool_name.clone(),
                namespace: Some("mcp__codex_apps__calendar".to_string()),
                arguments: "{}".to_string(),
                call_id: "call-namespace".to_string(),
            },
        )
        .await?
        .expect("function_call should produce a tool call");

    assert_eq!(
        call.tool_name,
        ToolName::namespaced("mcp__codex_apps__calendar", tool_name)
    );
    assert_eq!(call.call_id, "call-namespace");
    match call.payload {
        ToolPayload::Function { arguments } => {
            assert_eq!(arguments, "{}");
        }
        other => panic!("expected function payload, got {other:?}"),
    }

    Ok(())
}

#[tokio::test]
async fn mcp_parallel_support_uses_exact_payload_server() -> anyhow::Result<()> {
    let (_, turn) = make_session_and_context().await;
    let router = ToolRouter::from_config(
        &turn.tools_config,
        ToolRouterParams {
            deferred_mcp_tools: None,
            mcp_tools: None,
            unavailable_called_tools: Vec::new(),
            parallel_mcp_server_names: HashSet::from(["echo".to_string()]),
            discoverable_tools: None,
            dynamic_tools: turn.dynamic_tools.as_slice(),
        },
    );

    let deferred_call = ToolCall {
        tool_name: ToolName::namespaced("mcp__echo__", "query_with_delay"),
        call_id: "call-deferred".to_string(),
        payload: ToolPayload::Mcp {
            server: "echo".to_string(),
            tool: "query_with_delay".to_string(),
            raw_arguments: "{}".to_string(),
            model_content_only: false,
            is_freeform: false,
        },
    };
    assert!(router.tool_supports_parallel(&deferred_call));

    let different_server_call = ToolCall {
        tool_name: ToolName::namespaced("mcp__hello_echo__", "query_with_delay"),
        call_id: "call-other-server".to_string(),
        payload: ToolPayload::Mcp {
            server: "hello_echo".to_string(),
            tool: "query_with_delay".to_string(),
            raw_arguments: "{}".to_string(),
            model_content_only: false,
            is_freeform: false,
        },
    };
    assert!(!router.tool_supports_parallel(&different_server_call));

    Ok(())
}

#[tokio::test]
async fn model_visible_specs_filter_deferred_dynamic_tools() -> anyhow::Result<()> {
    let (_, turn) = make_session_and_context().await;
    let hidden_tool = "hidden_dynamic_tool";
    let visible_tool = "visible_dynamic_tool";
    let dynamic_tools = vec![
        DynamicToolSpec {
            namespace: Some("codex_app".to_string()),
            name: hidden_tool.to_string(),
            description: "Hidden until discovered.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false,
            }),
            defer_loading: true,
        },
        DynamicToolSpec {
            namespace: Some("codex_app".to_string()),
            name: visible_tool.to_string(),
            description: "Visible immediately.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false,
            }),
            defer_loading: false,
        },
    ];

    let router = ToolRouter::from_config(
        &turn.tools_config,
        ToolRouterParams {
            deferred_mcp_tools: None,
            mcp_tools: None,
            unavailable_called_tools: Vec::new(),
            parallel_mcp_server_names: HashSet::new(),
            discoverable_tools: None,
            dynamic_tools: &dynamic_tools,
        },
    );

    assert!(
        router
            .find_spec(&ToolName::namespaced("codex_app", hidden_tool))
            .is_some()
    );
    assert_eq!(
        namespace_function_names(&router.specs(), "codex_app"),
        vec![hidden_tool.to_string(), visible_tool.to_string()]
    );
    assert_eq!(
        namespace_function_names(&router.model_visible_specs(), "codex_app"),
        vec![visible_tool.to_string()]
    );

    Ok(())
}

#[tokio::test]
async fn custom_tool_call_does_not_route_to_deferred_exact_mcp_freeform_tool() -> anyhow::Result<()>
{
    let (session, turn) = make_session_and_context().await;
    let session = Arc::new(session);
    let deferred_freeform_tool_name = "mcp__rmcp__freeform_echo".to_string();
    let deferred_freeform_tool = ToolInfo {
        server_name: "rmcp".to_string(),
        callable_name: "freeform_echo".to_string(),
        callable_namespace: "mcp__rmcp__".to_string(),
        server_instructions: None,
        model_content_only: true,
        mcp_freeform: true,
        tool: Tool {
            name: "freeform_echo".to_string().into(),
            title: None,
            description: Some("Deferred freeform echo".to_string().into()),
            input_schema: Arc::new(
                serde_json::from_value::<JsonObject>(json!({
                    "type": "object",
                    "properties": {
                        "freeform": { "type": "string" }
                    },
                    "required": ["freeform"],
                    "additionalProperties": false
                }))
                .expect("freeform schema should deserialize"),
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        connector_id: None,
        connector_name: None,
        plugin_display_names: Vec::new(),
        connector_description: None,
    };

    let router = ToolRouter::from_config(
        &turn.tools_config,
        ToolRouterParams {
            deferred_mcp_tools: Some(HashMap::from([(
                deferred_freeform_tool_name.clone(),
                deferred_freeform_tool,
            )])),
            mcp_tools: None,
            unavailable_called_tools: Vec::new(),
            parallel_mcp_server_names: HashSet::new(),
            discoverable_tools: None,
            dynamic_tools: turn.dynamic_tools.as_slice(),
        },
    );

    let call = router
        .build_tool_call(
            session.as_ref(),
            ResponseItem::CustomToolCall {
                id: None,
                status: None,
                call_id: "call-deferred-freeform".to_string(),
                name: deferred_freeform_tool_name.clone(),
                input: "raw deferred input".to_string(),
            },
        )
        .await?
        .expect("custom tool call should produce a tool call");

    assert_eq!(call.tool_name, ToolName::plain(deferred_freeform_tool_name));
    assert_eq!(call.call_id, "call-deferred-freeform");
    match call.payload {
        ToolPayload::Custom { input } => assert_eq!(input, "raw deferred input"),
        other => panic!("expected plain custom payload, got {other:?}"),
    }

    Ok(())
}

fn namespace_function_names(specs: &[ToolSpec], namespace_name: &str) -> Vec<String> {
    specs
        .iter()
        .find_map(|spec| match spec {
            ToolSpec::Namespace(namespace) if namespace.name == namespace_name => Some(
                namespace
                    .tools
                    .iter()
                    .map(|tool| match tool {
                        ResponsesApiNamespaceTool::Function(tool) => tool.name.clone(),
                    })
                    .collect(),
            ),
            ToolSpec::Function(_)
            | ToolSpec::Freeform(_)
            | ToolSpec::ToolSearch { .. }
            | ToolSpec::LocalShell {}
            | ToolSpec::ImageGeneration { .. }
            | ToolSpec::WebSearch { .. }
            | ToolSpec::Namespace(_) => None,
        })
        .unwrap_or_default()
}

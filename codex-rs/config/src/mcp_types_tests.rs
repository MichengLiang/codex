use super::*;
use crate::schema::config_schema_json;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn deserialize_stdio_command_server_config() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
        "#,
    )
    .expect("should deserialize command config");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec![],
            env: None,
            env_vars: Vec::new(),
            cwd: None,
        }
    );
    assert!(cfg.enabled);
    assert!(!cfg.required);
    assert!(cfg.enabled_tools.is_none());
    assert!(cfg.disabled_tools.is_none());
}

#[test]
fn deserialize_stdio_command_server_config_with_args() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            args = ["hello", "world"]
        "#,
    )
    .expect("should deserialize command config");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            env: None,
            env_vars: Vec::new(),
            cwd: None,
        }
    );
    assert!(cfg.enabled);
}

#[test]
fn deserialize_stdio_command_server_config_with_arg_with_args_and_env() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            args = ["hello", "world"]
            env = { "FOO" = "BAR" }
        "#,
    )
    .expect("should deserialize command config");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            env: Some(HashMap::from([("FOO".to_string(), "BAR".to_string())])),
            env_vars: Vec::new(),
            cwd: None,
        }
    );
    assert!(cfg.enabled);
}

#[test]
fn deserialize_stdio_command_server_config_with_env_vars() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            env_vars = ["FOO", "BAR"]
        "#,
    )
    .expect("should deserialize command config with env_vars");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec![],
            env: None,
            env_vars: vec!["FOO".into(), "BAR".into()],
            cwd: None,
        }
    );
}

#[test]
fn deserialize_stdio_command_server_config_with_env_var_sources() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            env_vars = [
                "LEGACY_TOKEN",
                { name = "LOCAL_TOKEN", source = "local" },
                { name = "REMOTE_TOKEN", source = "remote" },
            ]
        "#,
    )
    .expect("should deserialize command config with sourced env_vars");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec![],
            env: None,
            env_vars: vec![
                McpServerEnvVar::Name("LEGACY_TOKEN".to_string()),
                McpServerEnvVar::Config {
                    name: "LOCAL_TOKEN".to_string(),
                    source: Some("local".to_string()),
                },
                McpServerEnvVar::Config {
                    name: "REMOTE_TOKEN".to_string(),
                    source: Some("remote".to_string()),
                },
            ],
            cwd: None,
        }
    );
}

#[test]
fn deserialize_stdio_command_server_config_rejects_unknown_env_var_source() {
    let err = toml::from_str::<McpServerConfig>(
        r#"
            command = "echo"
            env_vars = [{ name = "TOKEN", source = "elsewhere" }]
        "#,
    )
    .expect_err("unsupported env var source should be rejected");

    assert!(
        err.to_string()
            .contains("unsupported env_vars source `elsewhere`"),
        "unexpected error: {err}"
    );
}

#[test]
fn deserialize_stdio_command_server_config_with_cwd() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            cwd = "/tmp"
        "#,
    )
    .expect("should deserialize command config with cwd");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec![],
            env: None,
            env_vars: Vec::new(),
            cwd: Some(PathBuf::from("/tmp")),
        }
    );
}

#[test]
fn deserialize_disabled_server_config() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            enabled = false
        "#,
    )
    .expect("should deserialize disabled server config");

    assert!(!cfg.enabled);
    assert!(!cfg.required);
}

#[test]
fn deserialize_mcp_text_contract_flags_and_defaults() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            model_content_only = true
            mcp_freeform = true
        "#,
    )
    .expect("should deserialize MCP text contract flags");

    assert!(cfg.model_content_only);
    assert!(cfg.mcp_freeform);

    let default_cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
        "#,
    )
    .expect("should deserialize default MCP config");

    assert!(!default_cfg.model_content_only);
    assert!(!default_cfg.mcp_freeform);
}

#[test]
fn deserialize_required_server_config() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            required = true
        "#,
    )
    .expect("should deserialize required server config");

    assert!(cfg.required);
}

#[test]
fn deserialize_streamable_http_server_config() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            url = "https://example.com/mcp"
        "#,
    )
    .expect("should deserialize http config");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::StreamableHttp {
            url: "https://example.com/mcp".to_string(),
            bearer_token_env_var: None,
            http_headers: None,
            env_http_headers: None,
        }
    );
    assert!(cfg.enabled);
}

#[test]
fn deserialize_streamable_http_server_config_with_env_var() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            url = "https://example.com/mcp"
            bearer_token_env_var = "GITHUB_TOKEN"
        "#,
    )
    .expect("should deserialize http config");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::StreamableHttp {
            url: "https://example.com/mcp".to_string(),
            bearer_token_env_var: Some("GITHUB_TOKEN".to_string()),
            http_headers: None,
            env_http_headers: None,
        }
    );
    assert!(cfg.enabled);
}

#[test]
fn deserialize_streamable_http_server_config_with_headers() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            url = "https://example.com/mcp"
            http_headers = { "X-Foo" = "bar" }
            env_http_headers = { "X-Token" = "TOKEN_ENV" }
        "#,
    )
    .expect("should deserialize http config with headers");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::StreamableHttp {
            url: "https://example.com/mcp".to_string(),
            bearer_token_env_var: None,
            http_headers: Some(HashMap::from([("X-Foo".to_string(), "bar".to_string())])),
            env_http_headers: Some(HashMap::from([(
                "X-Token".to_string(),
                "TOKEN_ENV".to_string()
            )])),
        }
    );
}

#[test]
fn deserialize_streamable_http_server_config_with_oauth_resource() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            url = "https://example.com/mcp"
            oauth_resource = "https://api.example.com"
        "#,
    )
    .expect("should deserialize http config with oauth_resource");

    assert_eq!(
        cfg.oauth_resource,
        Some("https://api.example.com".to_string())
    );
}

#[test]
fn deserialize_server_config_with_tool_filters() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            enabled_tools = ["allowed"]
            disabled_tools = ["blocked"]
        "#,
    )
    .expect("should deserialize tool filters");

    assert_eq!(cfg.enabled_tools, Some(vec!["allowed".to_string()]));
    assert_eq!(cfg.disabled_tools, Some(vec!["blocked".to_string()]));
}

#[test]
fn deserialize_server_config_with_parallel_tool_calls() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            supports_parallel_tool_calls = true
        "#,
    )
    .expect("should deserialize supports_parallel_tool_calls");

    assert!(cfg.supports_parallel_tool_calls);
}

#[test]
fn deserialize_server_config_with_default_tool_approval_mode() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            default_tools_approval_mode = "approve"

            [tools.search]
            approval_mode = "prompt"
        "#,
    )
    .expect("should deserialize default tool approval mode");

    assert_eq!(
        cfg.default_tools_approval_mode,
        Some(AppToolApproval::Approve)
    );
    assert_eq!(
        cfg.tools.get("search"),
        Some(&McpServerToolConfig {
            approval_mode: Some(AppToolApproval::Prompt),
        })
    );

    let serialized = toml::to_string(&cfg).expect("should serialize MCP config");
    assert!(serialized.contains("default_tools_approval_mode = \"approve\""));

    let round_tripped: McpServerConfig =
        toml::from_str(&serialized).expect("should deserialize serialized MCP config");
    assert_eq!(round_tripped, cfg);
}

#[test]
fn serialize_round_trips_server_config_with_parallel_tool_calls() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"
            supports_parallel_tool_calls = true
            tool_timeout_sec = 2.0
        "#,
    )
    .expect("should deserialize supports_parallel_tool_calls");

    let serialized = toml::to_string(&cfg).expect("should serialize MCP config");
    assert!(serialized.contains("supports_parallel_tool_calls = true"));

    let round_tripped: McpServerConfig =
        toml::from_str(&serialized).expect("should deserialize serialized MCP config");
    assert_eq!(round_tripped, cfg);
}

#[test]
fn deserialize_ignores_unknown_server_fields() {
    let cfg = toml::from_str::<McpServerConfig>(
        r#"
            command = "echo"
            trust_level = "trusted"
        "#,
    )
    .expect("unknown MCP server fields should be ignored");

    assert_eq!(
        cfg.transport,
        McpServerTransportConfig::Stdio {
            command: "echo".to_string(),
            args: Vec::new(),
            env: None,
            env_vars: Vec::new(),
            cwd: None,
        }
    );
}

#[test]
fn config_schema_includes_mcp_text_contract_flags() {
    let schema_json = config_schema_json().expect("serialize config schema");
    let schema_value: serde_json::Value =
        serde_json::from_slice(&schema_json).expect("decode schema json");
    let properties = schema_value
        .pointer("/definitions/RawMcpServerConfig/properties")
        .expect("RawMcpServerConfig properties should exist")
        .as_object()
        .expect("RawMcpServerConfig properties should be an object");

    assert_eq!(
        properties
            .get("model_content_only")
            .and_then(|value| value.get("type"))
            .and_then(serde_json::Value::as_str),
        Some("boolean")
    );
    assert_eq!(
        properties
            .get("mcp_freeform")
            .and_then(|value| value.get("type"))
            .and_then(serde_json::Value::as_str),
        Some("boolean")
    );
}

#[test]
fn deserialize_rejects_command_and_url() {
    toml::from_str::<McpServerConfig>(
        r#"
            command = "echo"
            url = "https://example.com"
        "#,
    )
    .expect_err("should reject command+url");
}

#[test]
fn deserialize_rejects_env_for_http_transport() {
    toml::from_str::<McpServerConfig>(
        r#"
            url = "https://example.com"
            env = { "FOO" = "BAR" }
        "#,
    )
    .expect_err("should reject env for http transport");
}

#[test]
fn deserialize_rejects_headers_for_stdio() {
    toml::from_str::<McpServerConfig>(
        r#"
            command = "echo"
            http_headers = { "X-Foo" = "bar" }
        "#,
    )
    .expect_err("should reject http_headers for stdio transport");

    toml::from_str::<McpServerConfig>(
        r#"
            command = "echo"
            env_http_headers = { "X-Foo" = "BAR_ENV" }
        "#,
    )
    .expect_err("should reject env_http_headers for stdio transport");

    let err = toml::from_str::<McpServerConfig>(
        r#"
            command = "echo"
            oauth_resource = "https://api.example.com"
        "#,
    )
    .expect_err("should reject oauth_resource for stdio transport");

    assert!(
        err.to_string()
            .contains("oauth_resource is not supported for stdio"),
        "unexpected error: {err}"
    );
}

#[test]
fn deserialize_rejects_inline_bearer_token_field() {
    let err = toml::from_str::<McpServerConfig>(
        r#"
            url = "https://example.com"
            bearer_token = "secret"
        "#,
    )
    .expect_err("should reject bearer_token field");

    assert!(
        err.to_string().contains("bearer_token is not supported"),
        "unexpected error: {err}"
    );
}

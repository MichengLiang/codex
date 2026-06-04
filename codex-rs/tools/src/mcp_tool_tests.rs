use super::is_exact_freeform_input_schema;
use super::is_exact_freeform_schema;
use super::mcp_call_tool_result_output_schema;
use super::mcp_tool_to_freeform_tool;
use super::parse_mcp_tool;
use crate::AdditionalProperties;
use crate::JsonSchema;
use crate::JsonSchemaPrimitiveType;
use crate::JsonSchemaType;
use crate::ToolDefinition;
use crate::ToolName;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn mcp_tool(name: &str, description: &str, input_schema: serde_json::Value) -> rmcp::model::Tool {
    rmcp::model::Tool::new(
        name.to_string(),
        description.to_string(),
        std::sync::Arc::new(rmcp::model::object(input_schema)),
    )
}

#[test]
fn parse_mcp_tool_inserts_empty_properties() {
    let tool = mcp_tool(
        "no_props",
        "No properties",
        serde_json::json!({
            "type": "object"
        }),
    );

    assert_eq!(
        parse_mcp_tool(&tool).expect("parse MCP tool"),
        ToolDefinition {
            name: "no_props".to_string(),
            description: "No properties".to_string(),
            input_schema: JsonSchema::object(
                BTreeMap::new(),
                /*required*/ None,
                /*additional_properties*/ None
            ),
            output_schema: Some(mcp_call_tool_result_output_schema(serde_json::json!({}))),
            defer_loading: false,
        }
    );
}

#[test]
fn parse_mcp_tool_preserves_top_level_output_schema() {
    let mut tool = mcp_tool(
        "with_output",
        "Has output schema",
        serde_json::json!({
            "type": "object"
        }),
    );
    tool.output_schema = Some(std::sync::Arc::new(rmcp::model::object(
        serde_json::json!({
            "properties": {
                "result": {
                    "properties": {
                        "nested": {}
                    }
                }
            },
            "required": ["result"]
        }),
    )));

    assert_eq!(
        parse_mcp_tool(&tool).expect("parse MCP tool"),
        ToolDefinition {
            name: "with_output".to_string(),
            description: "Has output schema".to_string(),
            input_schema: JsonSchema::object(
                BTreeMap::new(),
                /*required*/ None,
                /*additional_properties*/ None
            ),
            output_schema: Some(mcp_call_tool_result_output_schema(serde_json::json!({
                "properties": {
                    "result": {
                        "properties": {
                            "nested": {}
                        }
                    }
                },
                "required": ["result"]
            }))),
            defer_loading: false,
        }
    );
}

#[test]
fn parse_mcp_tool_preserves_output_schema_without_inferred_type() {
    let mut tool = mcp_tool(
        "with_enum_output",
        "Has enum output schema",
        serde_json::json!({
            "type": "object"
        }),
    );
    tool.output_schema = Some(std::sync::Arc::new(rmcp::model::object(
        serde_json::json!({
            "enum": ["ok", "error"]
        }),
    )));

    assert_eq!(
        parse_mcp_tool(&tool).expect("parse MCP tool"),
        ToolDefinition {
            name: "with_enum_output".to_string(),
            description: "Has enum output schema".to_string(),
            input_schema: JsonSchema::object(
                BTreeMap::new(),
                /*required*/ None,
                /*additional_properties*/ None
            ),
            output_schema: Some(mcp_call_tool_result_output_schema(serde_json::json!({
                "enum": ["ok", "error"]
            }))),
            defer_loading: false,
        }
    );
}

#[test]
fn exact_freeform_schema_rejects_nullable_string_union() {
    let schema = JsonSchema::object(
        BTreeMap::from([(
            "freeform".to_string(),
            JsonSchema {
                schema_type: Some(JsonSchemaType::Multiple(vec![
                    JsonSchemaPrimitiveType::String,
                    JsonSchemaPrimitiveType::Null,
                ])),
                ..Default::default()
            },
        )]),
        Some(vec!["freeform".to_string()]),
        Some(AdditionalProperties::Boolean(false)),
    );

    assert!(!is_exact_freeform_schema(&schema));
}

#[test]
fn exact_freeform_schema_rejects_non_object_schema() {
    let schema = JsonSchema::string(/*description*/ None);

    assert!(!is_exact_freeform_schema(&schema));
}

#[test]
fn exact_freeform_schema_rejects_wrong_property_name() {
    let schema = JsonSchema::object(
        BTreeMap::from([("text".to_string(), JsonSchema::string(/*description*/ None))]),
        Some(vec!["text".to_string()]),
        Some(AdditionalProperties::Boolean(false)),
    );

    assert!(!is_exact_freeform_schema(&schema));
}

#[test]
fn exact_freeform_schema_rejects_missing_required() {
    let schema = JsonSchema::object(
        BTreeMap::from([(
            "freeform".to_string(),
            JsonSchema::string(/*description*/ None),
        )]),
        /*required*/ None,
        Some(AdditionalProperties::Boolean(false)),
    );

    assert!(!is_exact_freeform_schema(&schema));
}

#[test]
fn exact_freeform_schema_accepts_missing_additional_properties() {
    let schema = JsonSchema::object(
        BTreeMap::from([(
            "freeform".to_string(),
            JsonSchema::string(/*description*/ None),
        )]),
        Some(vec!["freeform".to_string()]),
        /*additional_properties*/ None,
    );

    assert!(is_exact_freeform_schema(&schema));
}

#[test]
fn exact_freeform_schema_rejects_additional_properties_true() {
    let schema = JsonSchema::object(
        BTreeMap::from([(
            "freeform".to_string(),
            JsonSchema::string(/*description*/ None),
        )]),
        Some(vec!["freeform".to_string()]),
        Some(AdditionalProperties::Boolean(true)),
    );

    assert!(!is_exact_freeform_schema(&schema));
}

#[test]
fn exact_freeform_input_schema_accepts_contract_shape() {
    let tool = mcp_tool(
        "freeform",
        "Exact freeform input",
        serde_json::json!({
            "type": "object",
            "properties": {
                "freeform": { "type": "string" }
            },
            "required": ["freeform"],
            "additionalProperties": false
        }),
    );

    assert!(is_exact_freeform_input_schema(&tool));
}

#[test]
fn exact_freeform_input_schema_accepts_schema_annotations() {
    let tool = mcp_tool(
        "freeform",
        "Annotated freeform input",
        serde_json::json!({
            "type": "object",
            "description": "Schema-level annotation",
            "properties": {
                "freeform": {
                    "type": "string",
                    "title": "Freeform text",
                    "description": "Raw input text."
                }
            },
            "required": ["freeform"]
        }),
    );

    assert!(is_exact_freeform_input_schema(&tool));
}

#[test]
fn mcp_freeform_text_tool_serializes_to_official_text_format() {
    let tool = mcp_tool(
        "freeform",
        "Exact freeform input",
        serde_json::json!({
            "type": "object",
            "properties": {
                "freeform": { "type": "string" }
            },
            "required": ["freeform"],
            "additionalProperties": false
        }),
    );

    let freeform_tool =
        mcp_tool_to_freeform_tool(&ToolName::namespaced("mcp__sample__", "freeform"), &tool)
            .expect("convert MCP freeform tool");

    assert_eq!(
        serde_json::to_value(freeform_tool).expect("serialize freeform tool"),
        serde_json::json!({
            "name": "mcp__sample__freeform",
            "description": "Exact freeform input",
            "format": {
                "type": "text"
            }
        })
    );
}

#[test]
fn mcp_freeform_text_tool_merges_tool_and_input_descriptions() {
    let tool = mcp_tool(
        "freeform",
        "Run raw source text.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "freeform": {
                    "type": "string",
                    "description": "The source text to run."
                }
            },
            "required": ["freeform"]
        }),
    );

    let freeform_tool =
        mcp_tool_to_freeform_tool(&ToolName::namespaced("mcp__sample__", "freeform"), &tool)
            .expect("convert MCP freeform tool");

    assert_eq!(
        serde_json::to_value(freeform_tool).expect("serialize freeform tool"),
        serde_json::json!({
            "name": "mcp__sample__freeform",
            "description": "Run raw source text.\n\nInput:\nThe source text to run.",
            "format": {
                "type": "text"
            }
        })
    );
}

#[test]
fn exact_freeform_input_schema_rejects_string_constraints() {
    let tool = mcp_tool(
        "freeform",
        "Constrained freeform input",
        serde_json::json!({
            "type": "object",
            "properties": {
                "freeform": {
                    "type": "string",
                    "minLength": 1
                }
            },
            "required": ["freeform"],
            "additionalProperties": false
        }),
    );

    assert!(!is_exact_freeform_input_schema(&tool));
}

#[test]
fn exact_freeform_input_schema_rejects_additional_properties_true() {
    let tool = mcp_tool(
        "freeform",
        "Allows extra input fields",
        serde_json::json!({
            "type": "object",
            "properties": {
                "freeform": { "type": "string" }
            },
            "required": ["freeform"],
            "additionalProperties": true
        }),
    );

    assert!(!is_exact_freeform_input_schema(&tool));
}

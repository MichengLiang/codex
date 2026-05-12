use crate::AdditionalProperties;
use crate::FreeformTool;
use crate::FreeformToolFormat;
use crate::JsonSchema;
use crate::JsonSchemaPrimitiveType;
use crate::JsonSchemaType;
use crate::ToolDefinition;
use crate::parse_tool_input_schema;
use serde_json::Value as JsonValue;
use serde_json::json;

pub fn parse_mcp_tool(tool: &rmcp::model::Tool) -> Result<ToolDefinition, serde_json::Error> {
    let mut serialized_input_schema = serde_json::Value::Object(tool.input_schema.as_ref().clone());

    // OpenAI models mandate the "properties" field in the schema. Some MCP
    // servers omit it (or set it to null), so we insert an empty object to
    // match the behavior of the Agents SDK.
    if let serde_json::Value::Object(obj) = &mut serialized_input_schema
        && obj.get("properties").is_none_or(serde_json::Value::is_null)
    {
        obj.insert(
            "properties".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );
    }

    let input_schema = parse_tool_input_schema(&serialized_input_schema)?;
    let structured_content_schema = tool
        .output_schema
        .as_ref()
        .map(|output_schema| serde_json::Value::Object(output_schema.as_ref().clone()))
        .unwrap_or_else(|| JsonValue::Object(serde_json::Map::new()));

    Ok(ToolDefinition {
        name: tool.name.to_string(),
        description: tool.description.clone().map(Into::into).unwrap_or_default(),
        input_schema,
        output_schema: Some(mcp_call_tool_result_output_schema(
            structured_content_schema,
        )),
        defer_loading: false,
    })
}

pub fn mcp_call_tool_result_output_schema(structured_content_schema: JsonValue) -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "content": {
                "type": "array",
                "items": {
                    "type": "object"
                }
            },
            "structuredContent": structured_content_schema,
            "isError": {
                "type": "boolean"
            },
            "_meta": {
                "type": "object"
            }
        },
        "required": ["content"],
        "additionalProperties": false
    })
}

pub fn is_exact_freeform_schema(schema: &JsonSchema) -> bool {
    if schema.schema_type != Some(JsonSchemaType::Single(JsonSchemaPrimitiveType::Object)) {
        return false;
    }

    let Some(properties) = &schema.properties else {
        return false;
    };
    if properties.len() != 1 {
        return false;
    }

    let Some(freeform) = properties.get("freeform") else {
        return false;
    };
    if freeform.schema_type != Some(JsonSchemaType::Single(JsonSchemaPrimitiveType::String))
        || freeform.any_of.is_some()
        || freeform.enum_values.is_some()
    {
        return false;
    }

    let Some(required) = schema.required.as_ref() else {
        return false;
    };
    if required.len() != 1 || required[0] != "freeform" {
        return false;
    }

    !matches!(
        schema.additional_properties,
        Some(AdditionalProperties::Boolean(true) | AdditionalProperties::Schema(_))
    )
}

pub fn is_exact_freeform_input_schema(tool: &rmcp::model::Tool) -> bool {
    let schema = JsonValue::Object(tool.input_schema.as_ref().clone());
    is_exact_freeform_input_schema_value(&schema)
}

fn is_exact_freeform_input_schema_value(schema: &JsonValue) -> bool {
    let Some(schema) = schema.as_object() else {
        return false;
    };
    if schema.get("type").and_then(JsonValue::as_str) != Some("object") {
        return false;
    }
    if !schema.keys().all(|key| {
        matches!(
            key.as_str(),
            "type" | "properties" | "required" | "additionalProperties" | "description" | "title"
        )
    }) {
        return false;
    }
    match schema.get("additionalProperties") {
        None | Some(JsonValue::Bool(false)) => {}
        Some(_) => return false,
    }
    let Some(required) = schema.get("required").and_then(JsonValue::as_array) else {
        return false;
    };
    if required.len() != 1 || required[0].as_str() != Some("freeform") {
        return false;
    }

    let Some(properties) = schema.get("properties").and_then(JsonValue::as_object) else {
        return false;
    };
    if properties.len() != 1 {
        return false;
    }
    let Some(freeform) = properties.get("freeform").and_then(JsonValue::as_object) else {
        return false;
    };
    freeform.get("type").and_then(JsonValue::as_str) == Some("string")
        && freeform
            .keys()
            .all(|key| matches!(key.as_str(), "type" | "description" | "title"))
}

pub fn mcp_tool_to_freeform_tool(
    tool_name: &crate::ToolName,
    tool: &rmcp::model::Tool,
) -> Result<FreeformTool, serde_json::Error> {
    let definition = parse_mcp_tool(tool)?;
    let schema = JsonValue::Object(tool.input_schema.as_ref().clone());
    let property_description = schema
        .get("properties")
        .and_then(JsonValue::as_object)
        .and_then(|properties| properties.get("freeform"))
        .and_then(JsonValue::as_object)
        .and_then(|freeform| freeform.get("description"))
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|description| !description.is_empty());
    let description = match (definition.description.trim(), property_description) {
        ("", Some(property_description)) => format!("Input:\n{property_description}"),
        (tool_description, Some(property_description)) => {
            format!("{tool_description}\n\nInput:\n{property_description}")
        }
        (tool_description, None) => tool_description.to_string(),
    };
    Ok(FreeformTool {
        name: tool_name.to_string(),
        description,
        format: FreeformToolFormat {
            r#type: "text".to_string(),
            syntax: String::new(),
            definition: String::new(),
        },
    })
}

#[cfg(test)]
#[path = "mcp_tool_tests.rs"]
mod tests;

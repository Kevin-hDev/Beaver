use serde_json::{json, Value};

use super::route_profile::SchemaPolicy;
#[cfg(test)]
use super::tool_schema_names::{
    has_provider_name_shape, wire_name, wire_name_with_tools, MAX_PROVIDER_TOOL_NAME,
};
pub(crate) use super::tool_schema_names::{
    restore_tool_name, restore_tool_name_for_provider, ToolNameMap,
};
use super::tool_schema_profile::{apply_strict_mode, remove_unsupported_keywords};

pub(crate) fn tools_for_policy(profile: SchemaPolicy, strict: bool, tools: &[Value]) -> Vec<Value> {
    let names = ToolNameMap::new(tools);
    tools
        .iter()
        .cloned()
        .map(|mut tool| {
            if let Some(function) = tool.get_mut("function").and_then(Value::as_object_mut) {
                if let Some(name) = function.get("name").and_then(Value::as_str) {
                    let wire_name = if profile == SchemaPolicy::Qwen {
                        names.wire_name_for_provider("qwen", name)
                    } else {
                        names.wire_name(name)
                    };
                    function.insert("name".to_string(), Value::String(wire_name));
                }
                if let Some(parameters) = function.get_mut("parameters") {
                    normalize_schema(parameters, profile);
                }
                apply_strict_mode(function, strict);
            }
            tool
        })
        .collect()
}

fn normalize_schema(value: &mut Value, profile: SchemaPolicy) {
    match value {
        Value::Object(map) => {
            remove_unsupported_keywords(map, profile);
            for (key, child) in map.iter_mut() {
                if key == "properties" {
                    normalize_properties(child, profile);
                } else if key == "additionalProperties" && child.is_boolean() {
                    continue;
                } else {
                    normalize_schema(child, profile);
                }
            }
            repair_structural_schema(map);
        }
        Value::Array(items) => {
            for item in items {
                normalize_schema(item, profile);
            }
        }
        Value::Bool(_) if !is_generic_schema(profile) => {
            *value = json!({"type": "string"});
        }
        _ => {}
    }
}

fn normalize_properties(value: &mut Value, profile: SchemaPolicy) {
    let Some(properties) = value.as_object_mut() else {
        return;
    };
    for schema in properties.values_mut() {
        match schema {
            Value::Object(_) | Value::Array(_) => normalize_schema(schema, profile),
            Value::Bool(_) if is_generic_schema(profile) => {}
            Value::Bool(_) => *schema = json!({"type": "string"}),
            _ => *schema = json!({"type": "string"}),
        }
    }
}

fn is_generic_schema(profile: SchemaPolicy) -> bool {
    matches!(profile, SchemaPolicy::Generic | SchemaPolicy::Qwen)
}

fn repair_structural_schema(map: &mut serde_json::Map<String, Value>) {
    if map.is_empty() {
        map.insert("type".to_string(), Value::String("string".to_string()));
        return;
    }
    match map.get("type").and_then(Value::as_str) {
        Some("array") if !map.contains_key("items") => {
            map.insert("items".to_string(), json!({"type": "string"}));
        }
        Some("object") => {
            let missing = map
                .get("properties")
                .and_then(Value::as_object)
                .is_none_or(serde_json::Map::is_empty);
            if missing {
                map.insert(
                    "properties".to_string(),
                    json!({"_unused": {"type": "string"}}),
                );
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "tool_schema_profile_tests.rs"]
mod profile_tests;
#[cfg(test)]
#[path = "tool_schema_tests.rs"]
mod tests;

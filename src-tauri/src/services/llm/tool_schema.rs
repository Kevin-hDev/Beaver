use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::tool_schema_profile::{
    apply_strict_mode, remove_unsupported_keywords, resolve, SchemaProfile,
};

const MAX_PROVIDER_TOOL_NAME: usize = 64;
const ALIAS_STEM_CHARS: usize = 26;

pub fn tools_for_provider(provider_id: &str, model: &str, tools: &[Value]) -> Vec<Value> {
    let profile = resolve(provider_id, model);
    tools
        .iter()
        .cloned()
        .map(|mut tool| {
            if let Some(function) = tool.get_mut("function").and_then(Value::as_object_mut) {
                if let Some(name) = function.get("name").and_then(Value::as_str) {
                    function.insert("name".to_string(), Value::String(wire_name(name)));
                }
                if let Some(parameters) = function.get_mut("parameters") {
                    normalize_schema(parameters, profile);
                }
                apply_strict_mode(function, provider_id, profile);
            }
            tool
        })
        .collect()
}

pub fn wire_name(name: &str) -> String {
    if is_common_provider_name(name) {
        return name.to_string();
    }
    let stem: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .take(ALIAS_STEM_CHARS)
        .collect();
    let digest = Sha256::digest(name.as_bytes());
    format!("tool_{stem}_{}", hex::encode(&digest[..16]))
}

pub fn restore_tool_name(name: &str, tools: &[Value]) -> String {
    tools
        .iter()
        .filter_map(tool_name)
        .find(|candidate| wire_name(candidate) == name)
        .unwrap_or(name)
        .to_string()
}

fn tool_name(tool: &Value) -> Option<&str> {
    tool.pointer("/function/name").and_then(Value::as_str)
}

fn is_common_provider_name(name: &str) -> bool {
    has_provider_name_shape(name) && !is_reserved_wire_name(name)
}

fn has_provider_name_shape(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    name.len() <= MAX_PROVIDER_TOOL_NAME
        && (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn is_reserved_wire_name(name: &str) -> bool {
    let Some((stem, digest)) = name.rsplit_once('_') else {
        return false;
    };
    stem.starts_with("tool_")
        && stem.len() > "tool_".len()
        && digest.len() == 32
        && digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn normalize_schema(value: &mut Value, profile: SchemaProfile) {
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
        Value::Bool(_) if profile != SchemaProfile::Generic => {
            *value = json!({"type": "string"});
        }
        _ => {}
    }
}

fn normalize_properties(value: &mut Value, profile: SchemaProfile) {
    let Some(properties) = value.as_object_mut() else {
        return;
    };
    for schema in properties.values_mut() {
        match schema {
            Value::Object(_) | Value::Array(_) => normalize_schema(schema, profile),
            Value::Bool(_) if profile == SchemaProfile::Generic => {}
            Value::Bool(_) => *schema = json!({"type": "string"}),
            _ => *schema = json!({"type": "string"}),
        }
    }
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
#[path = "tool_schema_tests.rs"]
mod tests;

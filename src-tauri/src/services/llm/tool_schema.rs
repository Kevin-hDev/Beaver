use serde_json::{json, Value};

pub fn tools_for_provider(provider_id: &str, model: &str, tools: &[Value]) -> Vec<Value> {
    if !needs_gemini_schema(provider_id, model) {
        return tools.to_vec();
    }
    tools
        .iter()
        .cloned()
        .map(|mut tool| {
            if let Some(params) = tool
                .get_mut("function")
                .and_then(|f| f.get_mut("parameters"))
            {
                normalize_schema(params);
            }
            tool
        })
        .collect()
}

fn needs_gemini_schema(provider_id: &str, model: &str) -> bool {
    provider_id == "google"
        || (provider_id == "openrouter" && model.to_lowercase().starts_with("google/"))
}

fn normalize_schema(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                *value = json!({"type": "string"});
                return;
            }

            for (key, child) in map.iter_mut() {
                if key == "properties" {
                    normalize_properties(child);
                } else {
                    normalize_schema(child);
                }
            }

            match map.get("type").and_then(Value::as_str) {
                Some("array") if !map.contains_key("items") => {
                    map.insert("items".to_string(), json!({"type": "string"}));
                }
                Some("object") if !map.contains_key("properties") => {
                    map.insert(
                        "properties".to_string(),
                        json!({
                            "_unused": {
                                "type": "string",
                                "description": "Optional placeholder for providers requiring non-empty object schemas."
                            }
                        }),
                    );
                }
                Some("object") => {
                    let needs_placeholder = map
                        .get("properties")
                        .and_then(Value::as_object)
                        .is_some_and(|props| props.is_empty());
                    if needs_placeholder {
                        map.insert(
                            "properties".to_string(),
                            json!({
                                "_unused": {
                                    "type": "string",
                                    "description": "Optional placeholder for providers requiring non-empty object schemas."
                                }
                            }),
                        );
                    }
                }
                _ => {}
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize_schema(item);
            }
        }
        _ => {}
    }
}

fn normalize_properties(value: &mut Value) {
    if let Value::Object(properties) = value {
        for schema in properties.values_mut() {
            normalize_property_schema(schema);
        }
    }
}

fn normalize_property_schema(value: &mut Value) {
    match value {
        Value::Object(_) => normalize_schema(value),
        Value::Bool(_) => {}
        _ => *value = json!({"type": "string"}),
    }
}

#[cfg(test)]
#[path = "tool_schema_tests.rs"]
mod tests;

use super::*;

#[test]
fn leaves_non_gemini_tools_unchanged() {
    let tools =
        vec![json!({"type":"function","function":{"name":"x","parameters":{"type":"array"}}})];

    assert_eq!(tools_for_provider("openai", "gpt-4o", &tools), tools);
}

#[test]
fn adds_items_and_object_properties_for_gemini_backends() {
    let tools = vec![json!({
        "type": "function",
        "function": {
            "name": "x",
            "parameters": {
                "type": "object",
                "properties": {
                    "items": {"type": "array"},
                    "payload": {"type": "object"}
                },
                "required": ["items"]
            }
        }
    })];

    let fixed = tools_for_provider("openrouter", "google/gemma-4-31b-it", &tools);
    let params = &fixed[0]["function"]["parameters"]["properties"];
    assert_eq!(params["items"]["items"]["type"], "string");
    assert!(params["payload"]["properties"].is_object());
}

#[test]
fn replaces_empty_schema_objects_for_gemini_backends() {
    let tools = vec![json!({
        "type": "function",
        "function": {
            "name": "x",
            "parameters": {
                "type": "object",
                "properties": {
                    "value": {}
                }
            }
        }
    })];

    let fixed = tools_for_provider("google", "gemma-4-26b-a4b-it", &tools);
    assert_eq!(
        fixed[0]["function"]["parameters"]["properties"]["value"]["type"],
        "string"
    );
}

#[test]
fn wraps_primitive_property_schemas_for_gemini_backends() {
    let tools = vec![json!({
        "type": "function",
        "function": {
            "name": "x",
            "parameters": {
                "type": "object",
                "properties": {
                    "type": "string",
                    "allowed": true
                }
            }
        }
    })];

    let fixed = tools_for_provider("google", "gemini-2.5-pro", &tools);
    let props = &fixed[0]["function"]["parameters"]["properties"];
    assert_eq!(props["type"]["type"], "string");
    assert_eq!(props["allowed"], true);
}

#[test]
fn app_tool_definitions_are_safe_for_gemini_backends() {
    let tools = crate::services::agent_local::tool_dispatcher::get_tool_definitions();
    let fixed = tools_for_provider("openrouter", "google/gemma-4-31b-it", &tools);

    for tool in fixed {
        let params = &tool["function"]["parameters"];
        assert_schema_safe(params);
    }
}

fn assert_schema_safe(value: &Value) {
    match value {
        Value::Object(map) => {
            assert!(!map.is_empty(), "empty schema object");
            match map.get("type").and_then(Value::as_str) {
                Some("array") => assert!(map.contains_key("items"), "array schema without items"),
                Some("object") => {
                    let props = map
                        .get("properties")
                        .and_then(Value::as_object)
                        .expect("object schema without properties");
                    assert!(!props.is_empty(), "object schema with empty properties");
                }
                _ => {}
            }
            for child in map.values() {
                assert_schema_safe(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_schema_safe(item);
            }
        }
        _ => {}
    }
}

use super::*;

fn tool(name: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": "test",
            "parameters": parameters
        }
    })
}

#[test]
fn tool_schema_profile_sets_explicit_non_strict_mode_where_supported() {
    let tools = vec![tool(
        "read_file",
        json!({"type": "object", "properties": {}}),
    )];

    for provider in ["openai", "moonshot", "deepseek"] {
        let policy = super::super::route_profile::tool_policy(provider, "model").unwrap();
        let fixed = tools_for_policy(policy.schema, policy.strict, &tools);
        assert_eq!(fixed[0]["function"]["strict"], false, "{provider}");
    }
    let policy = super::super::route_profile::tool_policy("xai", "grok-4").unwrap();
    let xai = tools_for_policy(policy.schema, policy.strict, &tools);
    assert!(xai[0]["function"].get("strict").is_none());
}

#[test]
fn removes_kimi_unsupported_validation_keywords() {
    let tools = vec![tool(
        "read_file",
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 100,
                    "pattern": "x"
                },
                "count": {"type": "integer", "minimum": 1, "maximum": 5}
            },
            "additionalProperties": false
        }),
    )];

    let policy = super::super::route_profile::tool_policy("moonshot", "kimi-k2.7-code").unwrap();
    let fixed = tools_for_policy(policy.schema, policy.strict, &tools);
    let parameters = &fixed[0]["function"]["parameters"];
    let path = &parameters["properties"]["path"];
    let count = &parameters["properties"]["count"];
    assert!(path.get("minLength").is_none());
    assert!(path.get("pattern").is_none());
    assert!(count.get("minimum").is_none());
    assert_eq!(parameters["additionalProperties"], false);
}

#[test]
fn provider_copy_never_weakens_beaver_argument_validation() {
    let original_schema = json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "minLength": 1}
        },
        "required": ["path"],
        "additionalProperties": false
    });
    let tools = vec![tool("read_file", original_schema.clone())];

    let policy = super::super::route_profile::tool_policy("moonshot", "kimi-k2.7-code").unwrap();
    let provider_copy = tools_for_policy(policy.schema, policy.strict, &tools);

    assert!(
        provider_copy[0]["function"]["parameters"]["properties"]["path"]
            .get("minLength")
            .is_none()
    );
    assert!(crate::services::mcp_bridge::arguments::validate(
        &json!({"path": "", "unexpected": true}),
        Some(&original_schema),
    )
    .is_err());
}

#[test]
fn repairs_provider_structural_schemas_without_touching_original() {
    let tools = vec![tool(
        "inspect",
        json!({
            "type": "object",
            "properties": {
                "items": {"type": "array"},
                "payload": {"type": "object"},
                "allowed": true
            }
        }),
    )];

    let policy = super::super::route_profile::tool_policy("google", "gemini-3.5-flash").unwrap();
    let fixed = tools_for_policy(policy.schema, policy.strict, &tools);
    let properties = &fixed[0]["function"]["parameters"]["properties"];
    assert_eq!(properties["items"]["items"]["type"], "string");
    assert!(properties["payload"]["properties"].is_object());
    assert_eq!(properties["allowed"]["type"], "string");
    assert_eq!(
        tools[0]["function"]["parameters"]["properties"]["allowed"],
        true
    );
}

#[test]
fn app_tool_definitions_remain_structurally_safe_for_google() {
    let tools = crate::services::agent_local::tool_dispatcher::get_tool_definitions();
    let policy = super::super::route_profile::tool_policy("google", "gemini-3.5-flash").unwrap();
    let fixed = tools_for_policy(policy.schema, policy.strict, &tools);

    for tool in fixed {
        assert_schema_safe(&tool["function"]["parameters"]);
    }
}

fn assert_schema_safe(value: &Value) {
    match value {
        Value::Object(map) => {
            assert!(!map.is_empty(), "empty schema object");
            if map.get("type").and_then(Value::as_str) == Some("array") {
                assert!(map.contains_key("items"), "array schema without items");
            }
            if map.get("type").and_then(Value::as_str) == Some("object") {
                assert!(
                    map.get("properties")
                        .and_then(Value::as_object)
                        .is_some_and(|properties| !properties.is_empty()),
                    "object schema without properties"
                );
            }
            for child in map.values() {
                assert_schema_safe(child);
            }
        }
        Value::Array(items) => items.iter().for_each(assert_schema_safe),
        _ => {}
    }
}

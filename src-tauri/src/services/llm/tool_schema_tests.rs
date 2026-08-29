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
fn aliases_extension_names_and_restores_them() {
    let original = "beaver.office.documents.create";
    let tools = vec![tool(original, json!({"type": "object", "properties": {}}))];

    let policy = super::super::route_profile::tool_policy("openai", "gpt-5.6-sol").unwrap();
    let fixed = tools_for_policy(policy.schema, policy.strict, &tools);
    let alias = fixed[0]["function"]["name"].as_str().unwrap();

    assert_eq!(alias, "beaver_office_documents_create");
    assert!(alias.len() <= MAX_PROVIDER_TOOL_NAME);
    assert!(has_provider_name_shape(alias));
    assert_eq!(restore_tool_name(alias, &tools), original);
}

#[test]
fn only_hashes_names_when_normalization_really_collides() {
    let dotted = "beaver.office.documents.create";
    let underscored = "beaver_office_documents_create";
    let tools = vec![
        tool(dotted, json!({"type": "object"})),
        tool(underscored, json!({"type": "object"})),
    ];
    let left = wire_name_with_tools(dotted, &tools);
    let repeated = wire_name_with_tools(dotted, &tools);
    let right = wire_name_with_tools(underscored, &tools);

    assert_eq!(left, repeated);
    assert_ne!(left, right);
    assert_ne!(left, wire_name(dotted));
    assert_eq!(right, underscored);
    assert!(left.len() <= MAX_PROVIDER_TOOL_NAME);
    assert!(right.len() <= MAX_PROVIDER_TOOL_NAME);
    assert_eq!(restore_tool_name(&left, &tools), dotted);
    assert_eq!(restore_tool_name(&right, &tools), underscored);
}

#[test]
fn leaves_common_provider_names_unchanged() {
    assert_eq!(wire_name("read_file"), "read_file");
    assert_eq!(wire_name("search"), "search");
    assert_eq!(wire_name("_internal-tool"), "_internal-tool");
}

#[test]
fn hashes_only_the_overflow_of_names_longer_than_provider_limits() {
    let name = format!("beaver.{}", "very_long_extension_tool_name.".repeat(4));
    let alias = wire_name(&name);

    assert_eq!(alias.len(), MAX_PROVIDER_TOOL_NAME);
    assert!(has_provider_name_shape(&alias));
}

#[test]
fn overflow_alias_cannot_impersonate_an_exact_canonical_name() {
    let long_name = format!("beaver.{}", "very_long_extension_tool_name.".repeat(4));
    let overflow_alias = wire_name(&long_name);
    let tools = vec![
        tool(&long_name, json!({"type": "object"})),
        tool(&overflow_alias, json!({"type": "object"})),
    ];

    let long_wire = wire_name_with_tools(&long_name, &tools);
    let exact_wire = wire_name_with_tools(&overflow_alias, &tools);

    assert_ne!(long_wire, exact_wire);
    assert_eq!(exact_wire, overflow_alias);
    assert_eq!(restore_tool_name(&long_wire, &tools), long_name);
}

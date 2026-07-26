use super::transport::{validate_tools, McpToolDef, MAX_DESC_CHARS, MAX_TOOLS};
use serde_json::json;

fn tool(name: &str) -> McpToolDef {
    McpToolDef {
        name: name.to_string(),
        description: None,
        input_schema: Some(json!({"type": "object", "properties": {}})),
    }
}

#[test]
fn invalid_names_are_rejected_instead_of_rewritten() {
    assert!(validate_tools(vec![tool("read<secret>")]).is_err());
}

#[test]
fn duplicate_names_reject_the_whole_catalog() {
    assert!(validate_tools(vec![tool("read_data"), tool("read_data")]).is_err());
}

#[test]
fn oversized_catalog_is_rejected_instead_of_truncated() {
    let tools = (0..=MAX_TOOLS)
        .map(|index| tool(&format!("tool_{index}")))
        .collect();
    assert!(validate_tools(tools).is_err());
}

#[test]
fn valid_names_remain_exact_and_descriptions_are_bounded() {
    let mut definition = tool("read_data-v2");
    definition.description = Some(format!("start\0{}", "x".repeat(500)));
    let tools = validate_tools(vec![definition]).unwrap();
    assert_eq!(tools[0].name, "read_data-v2");
    let description = tools[0].description.as_deref().unwrap();
    assert!(!description.contains('\0'));
    assert_eq!(description.chars().count(), MAX_DESC_CHARS);
}

#[test]
fn missing_or_unsupported_schema_is_rejected() {
    let mut missing = tool("missing");
    missing.input_schema = None;
    assert!(validate_tools(vec![missing]).is_err());

    let mut unsupported = tool("unsupported");
    unsupported.input_schema = Some(json!({"type": "object", "$ref": "#/$defs/input"}));
    assert!(validate_tools(vec![unsupported]).is_err());
}

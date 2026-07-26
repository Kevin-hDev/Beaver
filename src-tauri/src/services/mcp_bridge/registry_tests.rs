use super::registry::select_exact_tool;
use super::transport::McpToolDef;
use serde_json::json;

fn tool(name: &str) -> McpToolDef {
    McpToolDef {
        name: name.to_string(),
        description: None,
        input_schema: Some(json!({"type": "object"})),
    }
}

#[test]
fn only_the_exact_last_listed_tool_is_selected() {
    let tools = vec![tool("read_public"), tool("read_private")];
    assert_eq!(
        select_exact_tool(&tools, "read_public").unwrap().name,
        "read_public"
    );
    assert!(select_exact_tool(&tools, "read").is_err());
    assert!(select_exact_tool(&tools, "hidden_tool").is_err());
}

#[test]
fn duplicate_tool_names_are_rejected_at_call_time_too() {
    let tools = vec![tool("duplicate"), tool("duplicate")];
    assert!(select_exact_tool(&tools, "duplicate").is_err());
}

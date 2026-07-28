use super::super::extension_tool_set_apply::{
    active_definitions_with, append_capacity_notice, definition_name,
};
use super::super::extension_tool_selection::CapacityDecision;
use super::ExtensionToolSet;
use serde_json::json;

#[test]
fn passthrough_never_manages_chat_tools_as_extensions() {
    let definitions = vec![json!({"function": {"name": "web_search"}})];
    let mut tools = ExtensionToolSet::passthrough(definitions.clone());

    tools.apply(&["example.replacement".to_string()]);

    assert_eq!(tools.active(), definitions);
    assert!(tools.selected_extension_names().is_empty());
}

#[test]
fn definitions_are_selected_by_injected_capacity_decision() {
    let tools = vec![
        json!({"function": {"name": "read_file"}}),
        json!({"function": {"name": "plugin.one"}}),
        json!({"function": {"name": "plugin.two"}}),
    ];
    let decision = CapacityDecision {
        active_plugin_ids: vec!["example.one".to_string()],
        omitted_plugin_ids: vec!["example.two".to_string()],
    };
    let selected = active_definitions_with(&tools, &decision, 3, |name| match name {
        "plugin.one" => Some("example.one".to_string()),
        "plugin.two" => Some("example.two".to_string()),
        _ => None,
    });

    assert_eq!(selected.len(), 2);
    assert_eq!(definition_name(&selected[1]), Some("plugin.one"));
}

#[test]
fn capacity_notice_is_added_only_when_needed() {
    let mut tools = vec![json!({
        "function": {
            "name": crate::services::extensions::SEARCH_TOOL_NAME,
            "description": "Catalogue"
        }
    })];

    append_capacity_notice(&mut tools, &["example.large".to_string()]);

    let description = tools[0]["function"]["description"]
        .as_str()
        .unwrap_or_default();
    assert!(description.contains("Provider limit"));
    assert!(description.contains("example.large"));
}

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

    assert_eq!(selected.tools.len(), 2);
    assert_eq!(definition_name(&selected.tools[1]), Some("plugin.one"));
}

#[test]
fn capacity_notice_is_added_only_when_needed() {
    let mut tools = vec![json!({
        "function": {
            "name": crate::services::extensions::SEARCH_TOOL_NAME,
            "description": "Catalogue"
        }
    })];

    append_capacity_notice(
        &mut tools,
        &["example.large".to_string()],
        &[],
        0,
    );

    let description = tools[0]["function"]["description"]
        .as_str()
        .unwrap_or_default();
    assert!(description.contains("Provider tool limit"));
    assert!(description.contains("example.large"));
}

#[test]
fn an_inactive_replacement_restores_the_native_definition() {
    let tools = vec![json!({
        "_beaverCoreFallback": {
            "function": {"name": "read_file", "description": "native"}
        },
        "function": {"name": "read_file", "description": "plugin"}
    })];
    let selected = active_definitions_with(
        &tools,
        &CapacityDecision::default(),
        1,
        |_| Some("example.replacement".to_string()),
    );

    assert_eq!(selected.tools[0]["function"]["description"], "native");
    assert!(selected.tools[0].get("_beaverCoreFallback").is_none());
}

#[test]
fn an_active_replacement_never_exposes_internal_fallback_metadata() {
    let tools = vec![json!({
        "_beaverCoreFallback": {
            "function": {"name": "read_file", "description": "native"}
        },
        "function": {"name": "read_file", "description": "plugin"}
    })];
    let decision = CapacityDecision {
        active_plugin_ids: vec!["example.replacement".to_string()],
        omitted_plugin_ids: Vec::new(),
    };
    let selected = active_definitions_with(&tools, &decision, 1, |_| {
        Some("example.replacement".to_string())
    });

    assert_eq!(selected.tools[0]["function"]["description"], "plugin");
    assert!(selected.tools[0].get("_beaverCoreFallback").is_none());
}

#[test]
fn a_core_tool_displaced_by_search_is_reported() {
    let tools = vec![
        json!({"function": {"name": "read_file"}}),
        json!({"function": {"name": "write_file"}}),
        json!({"function": {"name": crate::services::extensions::SEARCH_TOOL_NAME}}),
    ];
    let selected = active_definitions_with(&tools, &CapacityDecision::default(), 2, |_| None);

    assert_eq!(
        selected
            .tools
            .iter()
            .filter_map(definition_name)
            .collect::<Vec<_>>(),
        vec!["read_file", crate::services::extensions::SEARCH_TOOL_NAME]
    );
    assert_eq!(selected.omitted_tool_names, vec!["write_file"]);
}

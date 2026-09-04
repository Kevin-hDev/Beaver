use super::tool_catalog::*;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn defaults_match_product_choice() {
    assert_eq!(
        default_enabled_optional_tools(),
        vec![
            "load_skill",
            "manage_automation",
            "ask_user_choice",
            "delegate_task",
            "list_subagents",
            "get_subagent",
            "cancel_subagent",
            "message_subagent",
            "archive_subagent",
            "inspect_subagent_changes",
            "apply_subagent_changes",
            "discard_subagent_changes",
            "plan_mode"
        ]
    );
}

#[test]
fn rejects_locked_and_unknown_tool_ids() {
    assert!(validate_optional_tool_id("bash").is_err());
    assert!(validate_optional_tool_id("bash_control").is_err());
    assert!(validate_optional_tool_id("missing_tool").is_err());
    assert!(validate_optional_tool_id("load_skill").is_ok());
}

#[test]
fn filtered_definitions_keep_locked_and_enabled_optional_tools() {
    let enabled = vec!["load_skill".to_string()];
    let defs = super::tool_definitions::get_tool_definitions();
    let names = tool_names(&filter_tool_definitions(defs, &enabled));

    assert!(has_tool(&names, "bash"));
    assert!(has_tool(&names, "bash_control"));
    assert!(has_tool(&names, "search_mcp_tools"));
    assert!(has_tool(&names, "list_extensions"));
    assert!(has_tool(&names, "load_skill"));
    assert!(!has_tool(&names, "todo_write"));
    assert!(!has_tool(&names, "forecast_run"));
}

#[test]
fn forecast_audit_can_be_enabled_without_enabling_forecast_runs() {
    let enabled = vec!["forecast_data_audit".to_string()];
    let defs = super::tool_definitions::get_tool_definitions();
    let names = tool_names(&filter_tool_definitions(defs, &enabled));

    assert!(has_tool(&names, "forecast_data_audit"));
    assert!(!has_tool(&names, "forecast_run"));
}

#[test]
fn delegate_task_enables_all_subagent_control_tools() {
    let enabled = normalize_enabled_optional_tools(&["delegate_task".to_string()]);
    for tool_id in SUBAGENT_TOOLS {
        assert!(enabled.iter().any(|id| id == tool_id));
    }
}

#[test]
fn enabled_subagent_bundle_exposes_change_lifecycle_tools() {
    let enabled = normalize_enabled_optional_tools(&["delegate_task".to_string()]);
    let defs = super::tool_definitions::get_tool_definitions();
    let names = tool_names(&filter_tool_definitions(defs, &enabled));

    for tool_id in [
        "inspect_subagent_changes",
        "apply_subagent_changes",
        "discard_subagent_changes",
    ] {
        assert!(has_tool(&names, tool_id), "missing tool: {tool_id}");
    }
}

#[test]
fn native_definitions_catalog_and_groups_are_exhaustively_consistent() {
    let entries = catalog();
    assert_eq!(entries.len(), 46);
    assert_eq!(entries.iter().filter(|entry| entry.locked).count(), 14);
    assert_eq!(entries.iter().filter(|entry| !entry.locked).count(), 32);
    assert_eq!(entries.iter().filter(|entry| entry.default_enabled).count(), 27);
    let entry_by_id = entries
        .iter()
        .map(|entry| (entry.id, entry))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(entry_by_id.len(), entries.len(), "duplicate catalog id");

    let definitions = super::tool_definitions::native_tool_definitions();
    let definition_names = tool_names(&definitions);
    let definition_ids = definition_names.iter().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(definition_ids.len(), definition_names.len(), "duplicate definition");
    assert_eq!(
        definition_ids,
        entry_by_id.keys().copied().collect(),
        "native definitions and flat catalog diverged"
    );

    let groups = super::tool_group_catalog::groups();
    assert_eq!(groups.len(), 16);
    assert_eq!(groups.iter().filter(|group| group.locked).count(), 5);
    assert_eq!(groups.iter().filter(|group| !group.locked).count(), 11);
    let mut group_ids = BTreeSet::new();
    let mut grouped_tools = BTreeSet::new();
    for group in groups {
        assert!(group_ids.insert(group.id), "duplicate group: {}", group.id);
        for tool_id in group.tool_ids {
            let entry = entry_by_id
                .get(tool_id)
                .unwrap_or_else(|| panic!("unknown grouped tool: {tool_id}"));
            assert_eq!(entry.locked, group.locked, "locked mismatch: {tool_id}");
            assert_eq!(
                entry.default_enabled, group.default_enabled,
                "default mismatch: {tool_id}"
            );
            assert!(grouped_tools.insert(*tool_id), "tool in two groups: {tool_id}");
        }
    }

    let ungrouped = entry_by_id
        .keys()
        .filter(|tool_id| !grouped_tools.contains(**tool_id))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        ungrouped,
        [
            "inspect_extensions",
            "list_extensions",
            super::tool_extension_resource::NAME,
        ]
    );
    assert!(entry_by_id["inspect_extensions"].locked);
    assert!(entry_by_id["list_extensions"].locked);
    assert!(entry_by_id[super::tool_extension_resource::NAME].locked);
}

#[test]
fn required_native_parameters_always_explain_their_contract() {
    let mut missing = Vec::new();
    for definition in super::tool_definitions::native_tool_definitions() {
        let name = definition["function"]["name"].as_str().unwrap_or_default();
        let parameters = &definition["function"]["parameters"];
        collect_undocumented_required(parameters, name, &mut missing);
    }
    assert!(
        missing.is_empty(),
        "required parameters without description: {}",
        missing.join(", ")
    );
}

fn collect_undocumented_required(
    schema: &serde_json::Value,
    path: &str,
    missing: &mut Vec<String>,
) {
    if let Some(required) = schema["required"].as_array() {
        for property in required.iter().filter_map(|value| value.as_str()) {
            let child = &schema["properties"][property];
            if child["description"]
                .as_str()
                .is_none_or(|description| description.trim().is_empty())
            {
                missing.push(format!("{path}.{property}"));
            }
        }
    }
    if let Some(properties) = schema["properties"].as_object() {
        for (name, child) in properties {
            collect_undocumented_required(child, &format!("{path}.{name}"), missing);
        }
    }
    if let Some(items) = schema.get("items") {
        collect_undocumented_required(items, &format!("{path}[]"), missing);
    }
}

#[test]
fn default_native_catalog_measurement_is_reproducible() {
    let all = super::tool_definitions::native_tool_definitions();
    let enabled = default_enabled_optional_tools();
    let active = filter_tool_definitions(all.clone(), &enabled);
    let description_chars = all
        .iter()
        .filter_map(|definition| definition["function"]["description"].as_str())
        .map(|description| description.chars().count())
        .sum::<usize>();
    let schema_chars = all
        .iter()
        .map(|definition| definition["function"]["parameters"].to_string().chars().count())
        .sum::<usize>();
    let serialized_chars = active
        .iter()
        .map(|definition| definition.to_string().chars().count())
        .sum::<usize>();
    let estimated_tokens =
        crate::services::compress::token_estimate::estimate_tool_tokens(&active);

    assert_eq!(all.len(), 46);
    assert_eq!(active.len(), 27);
    println!(
        "TOOL_CATALOG_MEASUREMENT={}",
        serde_json::json!({
            "method": "unicode_chars_and_beaver_token_estimator",
            "nativeTools": all.len(),
            "defaultActiveTools": active.len(),
            "allDescriptionChars": description_chars,
            "allSchemaChars": schema_chars,
            "defaultSerializedChars": serialized_chars,
            "defaultEstimatedTokens": estimated_tokens,
        })
    );
}

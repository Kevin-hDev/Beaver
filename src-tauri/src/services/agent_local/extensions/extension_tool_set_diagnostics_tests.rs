use super::*;
use crate::services::agent_local::extension_tool_selection::{
    decide, PluginDescriptor, SelectionPolicy,
};
use serde_json::json;

fn plugin(id: &str) -> PluginDescriptor {
    PluginDescriptor {
        id: id.to_string(),
        tool_count: 1,
        definition_count: 1,
    }
}

#[test]
fn selection_reasons_follow_the_contractual_precedence() {
    let protected = vec!["plugin.protected".to_string()];
    let essential = vec!["plugin.essential".to_string()];
    let discovered = vec!["plugin.discovered".to_string()];
    let masked_active = vec![
        "plugin.protected".to_string(),
        "plugin.essential".to_string(),
        "plugin.discovered".to_string(),
    ];
    let omitted = vec!["plugin.capacity".to_string()];
    let evidence = SelectionEvidence {
        masked: true,
        active_plugin_ids: &masked_active,
        omitted_plugin_ids: &omitted,
        protected_plugin_ids: &protected,
        essential_plugin_ids: &essential,
        discovered_plugin_ids: &discovered,
    };

    assert_eq!(
        selection_reason("plugin.protected", &evidence).as_str(),
        "protected"
    );
    assert_eq!(
        selection_reason("plugin.essential", &evidence).as_str(),
        "essential"
    );
    assert_eq!(
        selection_reason("plugin.discovered", &evidence).as_str(),
        "previously_discovered"
    );
    assert_eq!(
        selection_reason("plugin.masked", &evidence).as_str(),
        "masked"
    );
    assert_eq!(
        selection_reason("plugin.capacity", &evidence).as_str(),
        "provider_capacity"
    );

    let catalog_active = vec!["plugin.catalog".to_string()];
    let catalog_evidence = SelectionEvidence {
        masked: false,
        active_plugin_ids: &catalog_active,
        omitted_plugin_ids: &[],
        protected_plugin_ids: &[],
        essential_plugin_ids: &[],
        discovered_plugin_ids: &[],
    };
    assert_eq!(
        selection_reason("plugin.catalog", &catalog_evidence).as_str(),
        "catalog_visible"
    );
}

#[test]
fn diagnostic_groups_are_derived_from_reachable_selection_states() {
    let masked_plugins = vec![
        plugin("plugin.protected"),
        plugin("plugin.essential"),
        plugin("plugin.discovered"),
        plugin("plugin.masked"),
    ];
    let order = masked_plugins
        .iter()
        .map(|plugin| plugin.id.clone())
        .collect::<Vec<_>>();
    let protected = vec!["plugin.protected".to_string()];
    let essential = vec!["plugin.essential".to_string()];
    let discovered = vec!["plugin.discovered".to_string()];
    let masked_decision = decide(
        &masked_plugins,
        SelectionPolicy {
            masked: true,
            tool_capacity: 4,
            ordered_plugin_ids: &order,
            capacity_plugin_ids: &order,
            protected_plugin_ids: &protected,
            essential_plugin_ids: &essential,
            discovered_plugin_ids: &discovered,
        },
    );
    let masked_evidence = SelectionEvidence {
        masked: true,
        active_plugin_ids: &masked_decision.active_plugin_ids,
        omitted_plugin_ids: &masked_decision.omitted_plugin_ids,
        protected_plugin_ids: &protected,
        essential_plugin_ids: &essential,
        discovered_plugin_ids: &discovered,
    };
    let masked_groups = selection_groups(&masked_plugins, &masked_evidence);

    assert_eq!(masked_groups.len(), 4);
    assert!(masked_groups.iter().any(|group| {
        group.reason == ExtensionDiagnosticReason::Masked && group.plugin_ids == ["plugin.masked"]
    }));

    let capacity_plugins = vec![plugin("plugin.catalog"), plugin("plugin.capacity")];
    let capacity_order = capacity_plugins
        .iter()
        .map(|plugin| plugin.id.clone())
        .collect::<Vec<_>>();
    let capacity_decision = decide(
        &capacity_plugins,
        SelectionPolicy {
            masked: false,
            tool_capacity: 1,
            ordered_plugin_ids: &capacity_order,
            capacity_plugin_ids: &capacity_order,
            protected_plugin_ids: &[],
            essential_plugin_ids: &[],
            discovered_plugin_ids: &[],
        },
    );
    let capacity_evidence = SelectionEvidence {
        masked: false,
        active_plugin_ids: &capacity_decision.active_plugin_ids,
        omitted_plugin_ids: &capacity_decision.omitted_plugin_ids,
        protected_plugin_ids: &[],
        essential_plugin_ids: &[],
        discovered_plugin_ids: &[],
    };
    let capacity_groups = selection_groups(&capacity_plugins, &capacity_evidence);

    assert!(capacity_groups.iter().any(|group| {
        group.reason == ExtensionDiagnosticReason::CatalogVisible
            && group.plugin_ids == ["plugin.catalog"]
    }));
    assert!(capacity_groups.iter().any(|group| {
        group.reason == ExtensionDiagnosticReason::ProviderCapacity
            && group.plugin_ids == ["plugin.capacity"]
    }));
}

#[test]
fn refresh_delta_uses_actual_definition_names_not_plugin_tool_counts() {
    let before = vec![
        json!({"function": {"name": "read_file", "description": "native"}}),
        json!({"function": {"name": "list_extensions"}}),
    ];
    let after = vec![
        json!({"function": {"name": "read_file", "description": "plugin replacement"}}),
        json!({"function": {"name": "list_extensions"}}),
        json!({"function": {"name": "plugin_new_tool"}}),
    ];

    assert_eq!(added_definition_names(&before, &after), ["plugin_new_tool"]);
}

#[test]
fn diagnostic_groups_cover_the_entire_bounded_registry() {
    let plugins = (0..crate::services::extensions::MAX_DISCOVERED_PLUGINS)
        .map(|index| plugin(&format!("plugin.item{index}")))
        .collect::<Vec<_>>();
    let evidence = SelectionEvidence {
        masked: true,
        active_plugin_ids: &[],
        omitted_plugin_ids: &[],
        protected_plugin_ids: &[],
        essential_plugin_ids: &[],
        discovered_plugin_ids: &[],
    };

    let groups = selection_groups(&plugins, &evidence);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].reason, ExtensionDiagnosticReason::Masked);
    assert_eq!(groups[0].plugin_ids.len(), plugins.len());
}

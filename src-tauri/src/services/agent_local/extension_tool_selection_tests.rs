use super::*;

fn plugin(id: &str, tool_count: usize) -> PluginDescriptor {
    PluginDescriptor {
        id: id.to_string(),
        tool_count,
        definition_count: tool_count,
    }
}

#[test]
fn never_selects_only_part_of_a_plugin() {
    let plugins = vec![plugin("example.large", 3), plugin("example.small", 1)];
    let order = vec!["example.large".to_string(), "example.small".to_string()];
    let decision = decide(
        &plugins,
        SelectionPolicy {
            masked: false,
            tool_capacity: 2,
            ordered_plugin_ids: &order,
            capacity_plugin_ids: &order,
            protected_plugin_ids: &[],
            essential_plugin_ids: &[],
            discovered_plugin_ids: &[],
        },
    );

    assert_eq!(decision.active_plugin_ids, vec!["example.small"]);
    assert_eq!(decision.omitted_plugin_ids, vec!["example.large"]);
}

#[test]
fn discovered_plugins_survive_masking_and_keep_order() {
    let plugins = vec![plugin("example.one", 1), plugin("example.two", 1)];
    let order = vec!["example.one".to_string(), "example.two".to_string()];
    let discovered = vec!["example.two".to_string()];
    let decision = decide(
        &plugins,
        SelectionPolicy {
            masked: true,
            tool_capacity: 2,
            ordered_plugin_ids: &order,
            capacity_plugin_ids: &order,
            protected_plugin_ids: &[],
            essential_plugin_ids: &[],
            discovered_plugin_ids: &discovered,
        },
    );

    assert_eq!(decision.active_plugin_ids, discovered);
}

#[test]
fn user_priority_wins_before_capacity_ranking() {
    let plugins = vec![plugin("example.user", 1), plugin("example.frequent", 1)];
    let stable = vec!["example.user".to_string(), "example.frequent".to_string()];
    let ranked = vec!["example.frequent".to_string(), "example.user".to_string()];
    let protected = vec!["example.user".to_string()];
    let decision = decide(
        &plugins,
        SelectionPolicy {
            masked: false,
            tool_capacity: 1,
            ordered_plugin_ids: &stable,
            capacity_plugin_ids: &ranked,
            protected_plugin_ids: &protected,
            essential_plugin_ids: &[],
            discovered_plugin_ids: &[],
        },
    );

    assert_eq!(decision.active_plugin_ids, protected);
    assert_eq!(decision.omitted_plugin_ids, vec!["example.frequent"]);
}

#[test]
fn usage_ranking_only_changes_selection_when_capacity_overflows() {
    let plugins = vec![plugin("example.alpha", 1), plugin("example.frequent", 1)];
    let stable = vec!["example.alpha".to_string(), "example.frequent".to_string()];
    let ranked = vec!["example.frequent".to_string(), "example.alpha".to_string()];
    let decision = decide(
        &plugins,
        SelectionPolicy {
            masked: false,
            tool_capacity: 1,
            ordered_plugin_ids: &stable,
            capacity_plugin_ids: &ranked,
            protected_plugin_ids: &[],
            essential_plugin_ids: &[],
            discovered_plugin_ids: &[],
        },
    );

    assert_eq!(decision.active_plugin_ids, vec!["example.frequent"]);
}

use super::*;
use crate::services::extensions::types::ExtensionTool;
use serde_json::json;

fn plugin(id: &str, name: &str, description: Option<&str>, essential: bool) -> IndexedPlugin {
    IndexedPlugin {
        id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: description.map(str::to_string),
        essential,
        tools: vec![ExtensionTool {
            name: format!("{id}.run"),
            description: "Run".to_string(),
            parameters: json!({"type": "object"}),
            replaces_core: false,
        }],
    }
}

#[test]
fn protected_then_essential_then_stable_rest() {
    let plugins = vec![
        plugin("example.z", "Zulu", None, false),
        plugin("example.a", "Alpha", Some("  one\n line  "), true),
        plugin("example.b", "Beta", None, false),
    ];
    let snapshot = build(&plugins, &["example.b".to_string()], &BTreeMap::new());

    assert_eq!(
        snapshot.ordered_plugin_ids,
        vec!["example.b", "example.a", "example.z"]
    );
    assert_eq!(snapshot.text, "- Beta\n- Alpha : one line\n- Zulu");
}

#[test]
fn plugin_names_cannot_add_phantom_catalog_lines() {
    let plugins = vec![plugin("example.a", "Alpha\n- Phantom", None, false)];

    let snapshot = build(&plugins, &[], &BTreeMap::new());

    assert_eq!(snapshot.text, "- Alpha - Phantom");
}

#[test]
fn an_enabled_plugin_without_tools_stays_in_the_catalog() {
    let mut empty = plugin("example.empty", "Empty", Some("No callable tools"), false);
    empty.tools.clear();

    let snapshot = build(&[empty], &[], &BTreeMap::new());

    assert_eq!(snapshot.text, "- Empty : No callable tools");
    assert_eq!(snapshot.ordered_plugin_ids, vec!["example.empty"]);
}

#[test]
fn self_declared_essential_plugins_are_bounded() {
    let plugins = (0..MAX_SELF_DECLARED_ESSENTIAL_PLUGINS + 2)
        .map(|index| {
            plugin(
                &format!("example.plugin{index}"),
                &format!("Plugin {index}"),
                None,
                true,
            )
        })
        .collect::<Vec<_>>();

    let snapshot = build(&plugins, &[], &BTreeMap::new());

    assert_eq!(
        snapshot.essential_plugin_ids.len(),
        MAX_SELF_DECLARED_ESSENTIAL_PLUGINS
    );
}

#[test]
fn fingerprint_changes_with_a_schema() {
    let first = vec![plugin("example.a", "Alpha", None, false)];
    let mut second = first.clone();
    second[0].tools[0].parameters = json!({"type": "object", "required": ["value"]});

    assert_ne!(
        build(&first, &[], &BTreeMap::new()).version,
        build(&second, &[], &BTreeMap::new()).version
    );
}

#[test]
fn fingerprint_changes_with_tool_selection_metadata() {
    let first = vec![plugin("example.a", "Alpha", None, false)];
    let mut second = first.clone();
    second[0].tools[0].description = "Updated behavior".to_string();
    second[0].tools[0].replaces_core = true;

    assert_ne!(
        build(&first, &[], &BTreeMap::new()).version,
        build(&second, &[], &BTreeMap::new()).version
    );
}

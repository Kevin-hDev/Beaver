use super::*;
use crate::services::extensions::registry_index::IndexedPlugin;
use crate::services::extensions::types::ExtensionTool;
use serde_json::json;

fn plugin(name: &str, description: &str) -> IndexedPlugin {
    IndexedPlugin {
        id: format!("example.{}", name.to_lowercase()),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some(description.to_string()),
        essential: false,
        tools: vec![ExtensionTool {
            name: format!("example.{}.create", name.to_lowercase()),
            description: description.to_string(),
            parameters: json!({"type": "object"}),
            replaces_core: false,
        }],
    }
}

#[test]
fn ranks_plugin_identity_above_generic_copy() {
    let presentation = ranked_match(
        "PowerPoint",
        plugin("Presentations", "Create Microsoft PowerPoint PPTX files"),
    )
    .expect("presentation match");
    let generic = ranked_match("PowerPoint", plugin("Files", "Create a generic file"));

    assert!(generic.is_none() || presentation.score > generic.unwrap().score);
}

#[test]
fn accents_are_normalized_for_explicit_search() {
    let result = ranked_match(
        "présentation",
        plugin("Presentations", "Create a presentation"),
    );

    assert!(result.is_some());
}

#[test]
fn query_clipping_is_utf8_safe() {
    let query = "é".repeat(MAX_SEARCH_QUERY_CHARS + 20);

    assert_eq!(
        clip_chars(&query, MAX_SEARCH_QUERY_CHARS).chars().count(),
        MAX_SEARCH_QUERY_CHARS
    );
}

#[test]
fn any_tool_description_can_match_its_complete_plugin() {
    let mut candidate = plugin("Sheets", "Create spreadsheets");
    candidate.tools.push(ExtensionTool {
        name: "example.sheets.inspect".to_string(),
        description: "Audit workbook formulas".to_string(),
        parameters: json!({"type": "object"}),
        replaces_core: false,
    });

    let result = ranked_match("formulas", candidate).expect("plugin match");

    assert_eq!(result.extension_id, "example.sheets");
}

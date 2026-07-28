use super::*;
use crate::services::extensions::registry_index::IndexedTool;
use crate::services::extensions::types::ExtensionTool;
use serde_json::json;

fn indexed(name: &str, plugin: &str, description: &str) -> IndexedTool {
    IndexedTool {
        extension_id: format!("example.{plugin}"),
        extension_name: plugin.to_string(),
        extension_description: description.to_string(),
        tool: ExtensionTool {
            name: name.to_string(),
            description: description.to_string(),
            parameters: json!({"type": "object", "properties": {"path": {"type": "string"}}}),
            replaces_core: false,
        },
    }
}

#[test]
fn ranks_identity_above_generic_description_words() {
    let presentation = ranked_match(
        "Create a PowerPoint",
        indexed(
            "beaver.office.presentations.create",
            "Presentations",
            "Create editable Microsoft PowerPoint PPTX presentations",
        ),
    )
    .expect("presentation match");
    let generic = ranked_match(
        "Create a PowerPoint",
        indexed("example.files.create", "Files", "Create a generic file"),
    )
    .expect("generic create match");

    assert!(presentation.score > generic.score);
    assert!(presentation.score >= MIN_RELEVANCE_SCORE);
}

#[test]
fn product_name_alone_reaches_automatic_selection_threshold() {
    let result = ranked_match(
        "PowerPoint",
        indexed(
            "beaver.office.presentations.create",
            "Presentations",
            "Create editable Microsoft PowerPoint PPTX presentations",
        ),
    )
    .expect("product match");

    assert!(result.score >= MIN_RELEVANCE_SCORE);
}

#[test]
fn accents_match_normalized_plugin_identity() {
    let result = ranked_match(
        "prépare une présentation",
        indexed(
            "beaver.office.presentations.create",
            "Presentations",
            "Create a presentation",
        ),
    )
    .expect("accented match");

    assert!(result.score >= MIN_RELEVANCE_SCORE);
}

#[test]
fn auto_query_keeps_capabilities_and_drops_generic_actions() {
    assert_eq!(auto_query("Créer un fichier PowerPoint"), "powerpoint");
    assert!(auto_query("create a file").is_empty());
}

#[test]
fn search_queries_are_bounded_by_characters() {
    let query = "é".repeat(MAX_SEARCH_QUERY_CHARS + 20);

    assert_eq!(
        clip_chars(&query, MAX_SEARCH_QUERY_CHARS).chars().count(),
        512
    );
}

#[test]
fn never_selects_only_part_of_a_plugin() {
    let indexed = vec![
        indexed("example.sheets.create", "Sheets", "Create sheets"),
        indexed("example.sheets.inspect", "Sheets", "Inspect sheets"),
    ];
    let ids = vec!["example.Sheets".to_string()];

    assert!(select_complete_plugins(&indexed, &ids, 1).is_empty());
    assert_eq!(
        select_complete_plugins(&indexed, &ids, 2),
        vec!["example.sheets.create", "example.sheets.inspect"]
    );
}

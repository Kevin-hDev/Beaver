use super::*;

fn snapshot(version: &str, text: &str, capacity_plugin_ids: &[&str]) -> CatalogSnapshot {
    CatalogSnapshot {
        text: text.to_string(),
        version: version.to_string(),
        ordered_plugin_ids: Vec::new(),
        capacity_plugin_ids: capacity_plugin_ids
            .iter()
            .map(|id| (*id).to_string())
            .collect(),
        protected_plugin_ids: Vec::new(),
        essential_plugin_ids: Vec::new(),
    }
}

#[test]
fn unchanged_registry_version_preserves_catalog_and_capacity_order() {
    let previous = snapshot("same", "stable", &["example.alpha"]);
    let next = snapshot("same", "changed", &["example.frequent"]);

    let selected = stable_catalog(previous.clone(), next);

    assert_eq!(selected.text, previous.text);
    assert_eq!(selected.capacity_plugin_ids, previous.capacity_plugin_ids);
}

#[test]
fn changed_registry_version_accepts_the_new_catalog() {
    let previous = snapshot("old", "stable", &["example.alpha"]);
    let next = snapshot("new", "updated", &["example.frequent"]);

    let selected = stable_catalog(previous, next.clone());

    assert_eq!(selected.text, next.text);
    assert_eq!(selected.capacity_plugin_ids, next.capacity_plugin_ids);
}

#[test]
fn unavailable_usage_scores_never_block_registry_rebuilds() {
    let scores = usage_scores_with(|| Err("unavailable".to_string()));

    assert!(scores.is_empty());
}

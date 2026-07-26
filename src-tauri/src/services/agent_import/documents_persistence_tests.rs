use super::tests::prepare;
use super::*;

#[test]
fn imported_document_cannot_be_unselected_after_migration() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    save_source_selection_to(&home, &data, selection.clone(), false).unwrap();
    let mut changed = selection;
    changed.selected_document_ids.clear();

    save_source_selection_to(&home, &data, changed, false).unwrap();

    let registry = registry::read_from(&data.join("external-agent-sources.json"));
    assert_eq!(registry.sources[0].selected_document_ids.len(), 1);
    assert!(registry.documents[0].enabled);
}

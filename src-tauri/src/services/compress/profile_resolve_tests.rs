use super::profile_defaults::beaver_profile;
use super::profile_resolve::{
    active_selection, resolve_from_document, ResolvedCompressionProfileSource,
};
use super::profile_store_document::CompressionProfileDocument;
use super::profile_types::CompressionWindowBand;
use crate::services::agent_local::types_session::SessionCompressionProfileSelection;

fn document_with_custom() -> CompressionProfileDocument {
    let mut document = CompressionProfileDocument::default();
    let mut custom = beaver_profile();
    custom.id = "custom".to_string();
    custom.name = "Custom".to_string();
    custom.revision = 7;
    custom.allow_under_64k = true;
    document.profiles.push(custom);
    document
}

fn choice(revision: u64) -> SessionCompressionProfileSelection {
    SessionCompressionProfileSelection {
        profile_id: "custom".to_string(),
        global_selection_revision: revision,
    }
}

#[test]
fn new_sessions_follow_the_global_profile() {
    let document = document_with_custom();
    let resolved = resolve_from_document(None, &document).unwrap();

    assert_eq!(resolved.profile.id, "beaver");
    assert_eq!(resolved.source, ResolvedCompressionProfileSource::Global);
}

#[test]
fn session_choice_is_active_only_for_the_current_global_revision() {
    let mut document = document_with_custom();
    let current = choice(document.global_selection_revision);
    let resolved = resolve_from_document(Some(&current), &document).unwrap();
    assert_eq!(resolved.profile.id, "custom");
    assert_eq!(resolved.source, ResolvedCompressionProfileSource::Session);

    document.global_selection_revision += 1;
    let invalidated = resolve_from_document(Some(&current), &document).unwrap();
    assert_eq!(invalidated.profile.id, "beaver");
    assert_eq!(invalidated.source, ResolvedCompressionProfileSource::Global);
}

#[test]
fn rename_keeps_the_id_but_deletion_falls_back_to_global() {
    let mut document = document_with_custom();
    let current = choice(document.global_selection_revision);
    document.profiles[1].name = "Renamed".to_string();
    document.profiles[1].revision += 1;
    let renamed = resolve_from_document(Some(&current), &document).unwrap();
    assert_eq!(renamed.profile.name, "Renamed");
    assert_eq!(renamed.profile_revision, 8);

    document.profiles.pop();
    let deleted = resolve_from_document(Some(&current), &document).unwrap();
    assert_eq!(deleted.profile.id, "beaver");
    assert_eq!(deleted.source, ResolvedCompressionProfileSource::Global);
}

#[test]
fn projection_uses_the_effective_window_and_under_64k_policy() {
    let mut document = document_with_custom();
    let custom = choice(document.global_selection_revision);
    let resolved = resolve_from_document(Some(&custom), &document).unwrap();
    assert_eq!(resolved.band(0), None);
    assert!(resolved.available(0));
    assert_eq!(resolved.band(32_768), Some(CompressionWindowBand::Under64K));
    assert!(resolved.available(32_768));
    assert_eq!(resolved.band(96_000), Some(CompressionWindowBand::Compact));
    assert_eq!(resolved.band(128_000), Some(CompressionWindowBand::Large));

    document.profiles[1].allow_under_64k = false;
    let disabled = resolve_from_document(Some(&custom), &document).unwrap();
    assert!(!disabled.available(32_768));
}

#[test]
fn clone_copies_only_an_active_session_choice() {
    let document = document_with_custom();
    let current = choice(document.global_selection_revision);
    assert_eq!(
        active_selection(Some(&current), &document)
            .unwrap()
            .unwrap()
            .profile_id,
        "custom"
    );

    let stale = choice(document.global_selection_revision + 1);
    assert!(active_selection(Some(&stale), &document).unwrap().is_none());
}

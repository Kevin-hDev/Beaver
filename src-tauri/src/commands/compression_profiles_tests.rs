use std::time::{Duration, Instant};

use crate::models::compression_profile_contract::CompressionProfileInput;
use crate::services::compress::profile_defaults::{beaver_profile, BEAVER_PROFILE_ID};
use crate::services::compress::profile_limits::MAX_PROFILES;
use crate::services::compress::profile_store_document::CompressionProfileDocument;
use crate::services::compress::profile_types::{
    CompressionCategory, CompressionWindowBand, SummaryFailurePolicy,
};

use super::compression_profiles_mutations as mutations;
use super::compression_profiles_projection::project;
use super::compression_profiles_undo::{UndoSlot, UNDO_DURATION};

fn input_from_profile(
    profile: &crate::services::compress::profile_types::CompressionProfile,
) -> CompressionProfileInput {
    serde_json::from_value(serde_json::to_value(profile).expect("serialize profile"))
        .expect("deserialize strict input")
}

#[test]
fn create_copies_source_selects_copy_and_increments_global_revision() {
    let mut document = CompressionProfileDocument::default();
    let before_revision = document.global_selection_revision;
    mutations::create(&mut document, BEAVER_PROFILE_ID, "Compact local".into())
        .expect("create profile");

    let created = document
        .profiles
        .iter()
        .find(|profile| profile.id != BEAVER_PROFILE_ID)
        .expect("created profile");
    assert_eq!(created.name, "Compact local");
    assert_eq!(created.revision, 1);
    assert_eq!(document.global_profile_id, created.id);
    assert_eq!(document.global_selection_revision, before_revision + 1);
}

#[test]
fn create_refuses_the_bounded_collection_limit() {
    let mut document = CompressionProfileDocument::default();
    for index in 1..MAX_PROFILES {
        let source = document.global_profile_id.clone();
        mutations::create(&mut document, &source, format!("Profile {index}"))
            .expect("fill bounded profile collection");
    }
    let source = document.global_profile_id.clone();
    assert!(mutations::create(&mut document, &source, "One too many".into()).is_err());
    assert_eq!(document.profiles.len(), MAX_PROFILES);
}

#[test]
fn rename_and_reset_preserve_global_selection_revision() {
    let mut document = CompressionProfileDocument::default();
    mutations::create(&mut document, BEAVER_PROFILE_ID, "First".into()).expect("create");
    let custom_id = document.global_profile_id.clone();
    let selection_revision = document.global_selection_revision;

    mutations::rename(&mut document, &custom_id, "Renamed".into()).expect("rename");
    assert_eq!(document.global_selection_revision, selection_revision);
    assert!(mutations::rename(&mut document, BEAVER_PROFILE_ID, "Locked".into()).is_err());

    let beaver_revision = document.profiles[0].revision;
    mutations::reset_beaver(&mut document).expect("reset Beaver");
    assert_eq!(document.profiles[0].revision, beaver_revision + 1);
    assert_eq!(document.global_selection_revision, selection_revision);
}

#[test]
fn select_increments_only_when_the_profile_changes() {
    let mut document = CompressionProfileDocument::default();
    mutations::create(&mut document, BEAVER_PROFILE_ID, "Custom".into()).expect("create");
    let custom_id = document.global_profile_id.clone();
    let revision = document.global_selection_revision;

    mutations::select_global(&mut document, &custom_id).expect("same selection");
    assert_eq!(document.global_selection_revision, revision);
    mutations::select_global(&mut document, BEAVER_PROFILE_ID).expect("new selection");
    assert_eq!(document.global_selection_revision, revision + 1);
}

#[test]
fn automatic_switch_changes_only_the_global_automatic_policy() {
    let mut document = CompressionProfileDocument::default();
    let selection_revision = document.global_selection_revision;

    mutations::set_automatic_enabled(&mut document, false).expect("disable automatic");

    assert!(!document.automatic_enabled);
    assert_eq!(document.global_selection_revision, selection_revision);
    assert_eq!(document.profiles[0], beaver_profile());
}

#[test]
fn save_rejects_a_stale_profile_revision() {
    let mut document = CompressionProfileDocument::default();
    let mut input = input_from_profile(&document.profiles[0]);
    input.revision += 1;
    assert!(mutations::save(&mut document, input).is_err());
    assert_eq!(document.profiles[0], beaver_profile());
}

#[test]
fn failure_policy_requires_an_explicit_fallback_model() {
    let mut document = CompressionProfileDocument::default();
    let mut input = input_from_profile(&document.profiles[0]);
    input.summary.failure_policy = SummaryFailurePolicy::TryFallback;
    assert!(mutations::save(&mut document, input).is_err());
}

#[test]
fn reduction_order_accepts_exactly_the_five_mockup_categories() {
    let mut document = CompressionProfileDocument::default();
    let mut input = input_from_profile(&document.profiles[0]);
    input.reduction_order = vec![
        CompressionCategory::Images,
        CompressionCategory::Files,
        CompressionCategory::Tools,
        CompressionCategory::AssistantMessages,
        CompressionCategory::UserMessages,
    ];
    mutations::save(&mut document, input).expect("valid reduction order");

    let mut invalid = input_from_profile(&document.profiles[0]);
    invalid.reduction_order.pop();
    assert!(mutations::save(&mut document, invalid).is_err());
}

#[test]
fn delete_global_falls_back_to_beaver_and_undo_restores_exactly() {
    let mut document = CompressionProfileDocument::default();
    mutations::create(&mut document, BEAVER_PROFILE_ID, "Disposable".into()).expect("create");
    let profile_id = document.global_profile_id.clone();
    let before = document.clone();
    mutations::delete(&mut document, &profile_id).expect("delete");
    let after = document.clone();
    assert_eq!(document.global_profile_id, BEAVER_PROFILE_ID);

    let now = Instant::now();
    let mut undo = UndoSlot::default();
    let token = undo.record(before.clone(), after, now);
    let (restored, expected_after) = undo.candidate(&token, now).expect("undo candidate");
    assert_eq!(restored, before);
    assert_eq!(expected_after, document);
    undo.clear_if_token(&token);
    assert!(undo.candidate(&token, now).is_err());
}

#[test]
fn undo_expires_and_a_new_snapshot_invalidates_the_previous_token() {
    let document = CompressionProfileDocument::default();
    let mut changed = document.clone();
    changed.global_selection_revision += 1;
    let now = Instant::now();
    let mut undo = UndoSlot::default();
    let expired = undo.record(document.clone(), changed.clone(), now);
    assert!(undo
        .candidate(&expired, now + UNDO_DURATION + Duration::from_millis(1))
        .is_err());

    let old = undo.record(document.clone(), changed.clone(), now);
    let new = undo.record(changed.clone(), document, now);
    assert!(undo.candidate(&old, now).is_err());
    assert!(undo.candidate(&new, now).is_ok());
}

#[test]
fn projection_accepts_an_arbitrary_window_and_includes_agentic_categories() {
    let projection = project(&beaver_profile(), CompressionWindowBand::Compact, 72_345)
        .expect("arbitrary context window");
    assert_eq!(projection.context_window, 72_345);
    assert!(projection.system_tools_tokens > 0);
    assert!(projection.categories_tokens > 0);
    assert!(projection.total_tokens >= u64::from(projection.categories_tokens));
}

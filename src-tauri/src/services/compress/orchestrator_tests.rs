use super::orchestrator::eligible;
use super::profile_resolve::resolve_from_document;
use super::profile_store_document::CompressionProfileDocument;
use super::profile_types::CompressionTrigger;

fn profile() -> super::profile_resolve::ResolvedCompressionProfile {
    resolve_from_document(None, &CompressionProfileDocument::default()).unwrap()
}

#[test]
fn automatic_uses_the_profile_threshold_and_known_window() {
    let profile = profile();

    assert!(!eligible(
        &profile,
        CompressionTrigger::Automatic,
        100_000,
        89_999
    ));
    assert!(eligible(
        &profile,
        CompressionTrigger::Automatic,
        100_000,
        90_000
    ));
    assert!(!eligible(
        &profile,
        CompressionTrigger::Automatic,
        0,
        100_000
    ));
}

#[test]
fn under_64k_is_disabled_by_default_for_both_triggers() {
    let profile = profile();

    assert!(!eligible(
        &profile,
        CompressionTrigger::Automatic,
        63_999,
        63_999
    ));
    assert!(!eligible(
        &profile,
        CompressionTrigger::Explicit,
        63_999,
        63_999
    ));
    assert!(eligible(
        &profile,
        CompressionTrigger::Automatic,
        64_000,
        57_600
    ));
}

#[test]
fn explicit_allows_an_unknown_window_without_inventing_a_projection() {
    assert!(eligible(
        &profile(),
        CompressionTrigger::Explicit,
        0,
        10_000
    ));
}

#[test]
fn under_64k_can_be_enabled_by_the_profile() {
    let mut document = CompressionProfileDocument::default();
    document.profiles[0].allow_under_64k = true;
    let profile = resolve_from_document(None, &document).unwrap();

    assert!(eligible(
        &profile,
        CompressionTrigger::Automatic,
        32_000,
        28_800
    ));
    assert!(eligible(&profile, CompressionTrigger::Explicit, 32_000, 1));
}

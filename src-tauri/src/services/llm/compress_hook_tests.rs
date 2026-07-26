use crate::services::compress::state::context_used_for_compression;

#[test]
fn uses_estimate_when_current_messages_are_larger() {
    assert_eq!(context_used_for_compression(Some(10_000), 12_000), 12_000);
}

#[test]
fn uses_provider_count_when_it_is_larger() {
    assert_eq!(context_used_for_compression(Some(15_000), 12_000), 15_000);
}

#[test]
fn context_falls_back_to_estimate_when_real_usage_missing() {
    assert_eq!(context_used_for_compression(None, 12_000), 12_000);
}

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

#[test]
fn global_switch_disables_only_automatic_compression() {
    let document = CompressionProfileDocument {
        automatic_enabled: false,
        ..CompressionProfileDocument::default()
    };
    let profile = resolve_from_document(None, &document).unwrap();

    assert!(!eligible(
        &profile,
        CompressionTrigger::Automatic,
        128_000,
        128_000
    ));
    assert!(eligible(&profile, CompressionTrigger::Explicit, 128_000, 1));
}

#[test]
fn explicit_under_64k_refusal_has_a_stable_user_facing_code() {
    assert_eq!(
        super::checkpoint_transaction::CompressionError::UnavailableUnder64K.public_message(),
        "compression_disabled_under_64k"
    );
    assert_eq!(
        super::checkpoint_transaction::CompressionError::Unavailable.public_message(),
        "compression_unavailable"
    );
    assert_eq!(
        super::checkpoint_transaction::CompressionError::AutomaticSuspended.public_message(),
        "compression_automatic_suspended"
    );
}

#[test]
fn summary_transport_retryability_uses_stable_provider_codes_only() {
    for code in [
        "rate_limit",
        "provider_temporarily_unavailable",
        "provider_connection_failed",
    ] {
        assert_eq!(
            super::orchestrator_summary::classify(code, false),
            super::summary_request::SummaryAttemptError::Retryable
        );
    }
    for code in [
        "oauth_reauthentication_required",
        "provider_configuration_invalid",
        "the model returned 500 items",
    ] {
        assert_eq!(
            super::orchestrator_summary::classify(code, false),
            super::summary_request::SummaryAttemptError::Fatal
        );
    }
}

#[test]
fn automatic_open_turn_is_silent_before_start_while_explicit_reports_it() {
    let mut messages = super::snapshot_tests::session().messages;
    messages.truncate(2);
    messages[1].tool_calls = Some(vec![
        crate::services::agent_local::types_message::ToolCallRequest {
            id: uuid::Uuid::new_v4().to_string(),
            extra_content: None,
            function: crate::services::agent_local::types_message::ToolCallRequestFunction {
                name: "web_search".into(),
                arguments: serde_json::json!({}),
            },
        },
    ]);

    assert_eq!(
        super::orchestrator::preflight_messages(&messages, CompressionTrigger::Automatic),
        Ok(false)
    );
    assert_eq!(
        super::orchestrator::preflight_messages(&messages, CompressionTrigger::Explicit),
        Err(super::checkpoint_transaction::CompressionError::OpenTurn)
    );
}

#[test]
fn user_cancellation_does_not_increment_the_automatic_failure_guard() {
    assert!(!super::orchestrator::should_record_failure(
        CompressionTrigger::Automatic,
        super::checkpoint_transaction::CompressionError::Cancelled,
    ));
    assert!(super::orchestrator::should_record_failure(
        CompressionTrigger::Automatic,
        super::checkpoint_transaction::CompressionError::SummaryRequestFailed,
    ));
    assert!(!super::orchestrator::should_record_failure(
        CompressionTrigger::Explicit,
        super::checkpoint_transaction::CompressionError::SummaryRequestFailed,
    ));
}

#[test]
fn summary_input_margin_scales_to_five_percent_of_the_effective_window() {
    assert_eq!(
        super::orchestrator_summary::summary_input_safety_tokens(34_000),
        1_700
    );
    assert_eq!(
        super::orchestrator_summary::summary_input_safety_tokens(4_000),
        256
    );
}

use super::checkpoint_transaction::CompressionError;
use super::metrics::{
    CompressionMetricContext, CompressionMetricError, CompressionMetricOutcome,
    CompressionMetricPhase, CompressionMetrics, CompressionSuccessFacts,
};
use super::profile_types::CompressionTrigger;
use crate::services::provider_usage::CacheTokenTotals;

fn metric(
    error: Option<CompressionError>,
    cancelled: bool,
    duration_ms: u64,
) -> CompressionMetrics {
    let session = super::snapshot_tests::session();
    let mut snapshot = super::snapshot_tests::snapshot(&session);
    snapshot.profile.profile.name = "PRIVATE PROFILE NAME".to_string();
    snapshot.profile.profile.summary.system_prompt = "SECRET PROMPT /private/path".to_string();
    CompressionMetrics::finish(
        CompressionMetricContext {
            session_id: &session.id,
            request_id: "7d89c74f-d8ab-4447-b84b-8e4b944dc12b",
            profile: &snapshot.profile,
            trigger: CompressionTrigger::Automatic,
            context_window: 128_000,
            before_tokens: 115_200,
            projected_budget_tokens: 75_000,
            compression_count: 3,
            cache_before: CacheTokenTotals {
                read_tokens: 100,
                write_tokens: 20,
            },
        },
        error.is_none().then_some(CompressionSuccessFacts {
            after_tokens: 72_000,
            summary_tokens: 1_500,
            retained_user_tokens: 20_000,
            retained_tool_results: 14,
            dropped_tool_results: 3,
            retained_images: 8,
            dropped_images: 2,
            retained_subagent_reports: 4,
            compression_count: 4,
        }),
        error,
        cancelled,
        duration_ms,
        CacheTokenTotals {
            read_tokens: 160,
            write_tokens: 30,
        },
    )
}

#[test]
fn successful_metric_contains_only_bounded_technical_facts() {
    let metric = metric(None, false, u64::MAX);
    let log = metric.safe_log_json();

    assert_eq!(metric.outcome, CompressionMetricOutcome::Success);
    assert_eq!(metric.phase, CompressionMetricPhase::Commit);
    assert_eq!(metric.effective_threshold_percent, 90);
    assert_eq!(metric.compression_count, 4);
    assert_eq!(metric.cache_read_tokens_after, 160);
    assert_eq!(metric.duration_ms, 86_400_000);
    assert!(log.contains("\"retained_tool_results\":14"));
    assert!(log.contains("\"projected_budget_tokens\":75000"));
    for forbidden in [
        "PRIVATE PROFILE NAME",
        "SECRET PROMPT",
        "/private/path",
        "filename",
        "content",
    ] {
        assert!(!log.contains(forbidden));
    }
}

#[test]
fn failures_use_closed_error_and_phase_enums() {
    let metric = metric(Some(CompressionError::SummaryInvalid), false, 10);

    assert_eq!(metric.outcome, CompressionMetricOutcome::Failed);
    assert_eq!(metric.phase, CompressionMetricPhase::Summary);
    assert_eq!(metric.error, Some(CompressionMetricError::InvalidSummary));
    assert_eq!(metric.error.unwrap().code(), "invalid_summary");
    assert_eq!(metric.after_tokens, 0);
    assert_eq!(metric.projected_overflow_tokens, 40_200);
}

#[test]
fn cancellation_is_distinct_without_hiding_the_generic_error() {
    let metric = metric(Some(CompressionError::SaveFailed), true, 10);

    assert_eq!(metric.outcome, CompressionMetricOutcome::Cancelled);
    assert_eq!(metric.error, Some(CompressionMetricError::SaveFailed));
}

#[test]
fn malformed_identifiers_are_replaced_instead_of_logged() {
    let session = super::snapshot_tests::session();
    let snapshot = super::snapshot_tests::snapshot(&session);
    let metric = CompressionMetrics::finish(
        CompressionMetricContext {
            session_id: "../../secret-session",
            request_id: "secret request",
            profile: &snapshot.profile,
            trigger: CompressionTrigger::Explicit,
            context_window: 0,
            before_tokens: 1,
            projected_budget_tokens: 1,
            compression_count: 0,
            cache_before: CacheTokenTotals::default(),
        },
        Some(CompressionSuccessFacts::default()),
        None,
        false,
        0,
        CacheTokenTotals::default(),
    );

    assert_eq!(metric.session_id, "invalid");
    assert_eq!(metric.request_id, "invalid");
    assert!(!metric.safe_log_json().contains("secret"));
}

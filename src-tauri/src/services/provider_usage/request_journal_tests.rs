use super::request_journal::{
    served_tier, ProviderRequestMetric, RequestMetricStatus, RequestTiming, ServiceTierServed,
    REQUEST_LIMIT,
};
use super::{RequestUsage, UsageApiFormat};

fn metric(session_id: Option<&str>, attempt: u32) -> ProviderRequestMetric {
    ProviderRequestMetric {
        started_at_ms: 1_780_000_000_000_i64.saturating_add(i64::from(attempt)),
        connection_id: "openai".into(),
        canonical_provider_id: "openai".into(),
        api_format: UsageApiFormat::ChatCompletions,
        model: "gpt-5.6-sol".into(),
        session_id: session_id.map(str::to_string),
        request_id: format!("request-{attempt}"),
        turn: Some(1),
        attempt,
        workload: "primary".into(),
        origin: "manual_chat".into(),
        status: RequestMetricStatus::Completed,
        timing: RequestTiming {
            total_ms: 10,
            ..Default::default()
        },
        usage_complete: false,
        ..Default::default()
    }
}

#[test]
fn served_tier_accepts_only_the_closed_provider_values() {
    assert_eq!(served_tier("fast"), ServiceTierServed::Fast);
    assert_eq!(served_tier("priority"), ServiceTierServed::Fast);
    assert_eq!(served_tier("default"), ServiceTierServed::Default);
    assert_eq!(served_tier("auto"), ServiceTierServed::Unknown);
    assert_eq!(served_tier("ultrafast"), ServiceTierServed::Unknown);
}

#[test]
fn journal_keeps_only_the_latest_two_hundred_attempts_per_session() {
    let mut entries: Vec<_> = (1..=205)
        .map(|attempt| metric(Some("session-1"), attempt))
        .collect();

    super::request_journal::prune(&mut entries);

    assert_eq!(entries.len(), 200);
    assert_eq!(entries.first().map(|entry| entry.attempt), Some(6));
    assert_eq!(entries.last().map(|entry| entry.attempt), Some(205));
}

#[test]
fn journal_global_limit_keeps_the_latest_attempts() {
    let mut entries: Vec<_> = (1..=REQUEST_LIMIT as u32 + 5)
        .map(|attempt| metric(None, attempt.min(1_000)))
        .collect();

    super::request_journal::prune(&mut entries);

    assert_eq!(entries.len(), REQUEST_LIMIT);
    assert_eq!(
        entries.first().map(|entry| entry.started_at_ms),
        Some(1_780_000_000_006),
    );
}

#[test]
fn unsafe_identifiers_are_rejected_before_storage() {
    let mut entry = metric(Some("session-1"), 1);
    entry.request_id = "../secret".into();
    assert!(!entry.is_valid());

    entry.request_id = "request-1".into();
    entry.model = "model\nforged".into();
    assert!(!entry.is_valid());
}

#[test]
fn route_timing_and_usage_states_must_stay_coherent() {
    let mut entry = metric(Some("session-1"), 1);
    entry.canonical_provider_id = "deepseek".into();
    assert!(!entry.is_valid());

    entry.canonical_provider_id = "openai".into();
    entry.timing.headers_ms = Some(8);
    entry.timing.first_event_ms = Some(4);
    assert!(!entry.is_valid());

    entry.timing.headers_ms = Some(2);
    entry.usage_complete = true;
    assert!(!entry.is_valid());

    entry.usage = Some(RequestUsage {
        input_tokens: Some(100),
        cached_input_tokens: Some(50),
        cache_status: super::request_usage::CacheUsageStatus::Unknown,
        ..Default::default()
    });
    assert!(!entry.is_valid());
}

#[test]
fn routed_endpoint_is_allowed_only_as_a_complete_openrouter_pair() {
    let mut entry = metric(Some("session-1"), 1);
    entry.connection_id = "openrouter".into();
    entry.canonical_provider_id = "openrouter".into();
    entry.routed_provider = Some("Google Vertex".into());
    entry.routed_model = Some("google/gemini-3.5-pro".into());
    assert!(entry.is_valid());

    entry.routed_model = None;
    assert!(!entry.is_valid());

    entry.routed_provider = None;
    entry.routed_model = None;
    entry.canonical_provider_id = "openai".into();
    entry.routed_provider = Some("OpenAI".into());
    entry.routed_model = Some("gpt-5.6-sol".into());
    assert!(!entry.is_valid());
}

#[test]
fn deserializer_rejects_more_than_the_hard_request_limit() {
    let entries: Vec<_> = (0..=REQUEST_LIMIT).map(|_| serde_json::json!({})).collect();
    let encoded = serde_json::json!({ "version": 2, "entries": entries });
    let decoded: Result<super::request_journal_store::RequestStore, _> =
        serde_json::from_value(encoded);

    assert!(decoded.is_err());
}

#[test]
fn session_summary_keeps_cache_quality_and_durations() {
    let mut first = metric(Some("session-1"), 1);
    first.usage = Some(RequestUsage {
        input_tokens: Some(100),
        cached_input_tokens: Some(80),
        cache_write_input_tokens: Some(20),
        cache_miss_input_tokens: Some(20),
        cache_miss_source: super::request_usage::CacheMissSource::Reported,
        cache_status: super::request_usage::CacheUsageStatus::Reported,
        ..Default::default()
    });
    let second = metric(Some("session-1"), 2);

    let summaries = super::request_journal_summary::session_summaries(&[first, second], "openai");

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].attempt_count, 2);
    assert_eq!(summaries[0].cache_observation_count, 1);
    assert_eq!(summaries[0].cache_read_observation_count, 1);
    assert_eq!(summaries[0].cache_write_observation_count, 1);
    assert_eq!(summaries[0].cache_miss_observation_count, 1);
    assert_eq!(summaries[0].cache_read_tokens, 80);
    assert_eq!(summaries[0].cache_write_tokens, 20);
    assert_eq!(summaries[0].total_duration_ms, 20);
}

use super::request_usage::RequestUsage;
use super::snapshot::build_snapshot;
use super::types::{LocalSnapshot, RemoteData};
use serde_json::json;

#[test]
fn openai_reasoning_is_not_added_twice() {
    let usage = RequestUsage::from_json(&json!({
        "prompt_tokens": 20,
        "completion_tokens": 12,
        "completion_tokens_details": { "reasoning_tokens": 8 },
        "total_tokens": 32
    }))
    .unwrap();

    assert_eq!(usage.output_tokens, Some(12));
    assert_eq!(usage.reasoning_output_tokens, Some(8));
    assert_eq!(usage.total_tokens, Some(32));
}

#[test]
fn gemini_thoughts_are_included_in_output_total() {
    let usage = RequestUsage::from_json(&json!({
        "promptTokenCount": 10,
        "candidatesTokenCount": 4,
        "thoughtsTokenCount": 6,
        "totalTokenCount": 20
    }))
    .unwrap();

    assert_eq!(usage.output_tokens, Some(10));
    assert_eq!(usage.reasoning_output_tokens, Some(6));
}

#[test]
fn exact_cost_accepts_a_bounded_decimal_string() {
    let usage = RequestUsage::from_json(&json!({
        "prompt_tokens": 2,
        "completion_tokens": 1,
        "cost": "0.000123"
    }))
    .unwrap();
    assert_eq!(usage.exact_cost_usd_micros, Some(123));
}

#[test]
fn cache_or_reasoning_data_alone_is_real_usage() {
    let usage = RequestUsage {
        cached_input_tokens: Some(4),
        reasoning_output_tokens: Some(2),
        ..Default::default()
    };
    assert!(!usage.is_empty());
}

#[test]
fn invalid_connection_is_rejected() {
    assert!(super::types::validate_connection_id("../secret").is_err());
    assert!(super::types::validate_connection_id("openai").is_ok());
}

#[test]
fn snapshot_keeps_remote_timestamp() {
    let remote = RemoteData {
        fetched_at: 123,
        ..Default::default()
    };
    let snapshot = build_snapshot("xai-oauth", LocalSnapshot::default(), remote);

    assert_eq!(snapshot.canonical_provider_id, "xai");
    assert_eq!(snapshot.auth_source, "oauth");
    assert_eq!(snapshot.refreshed_at, 123);
}

#[tokio::test]
async fn exact_provider_cost_wins() {
    let usage = RequestUsage {
        exact_cost_usd_micros: Some(42),
        input_tokens: Some(10),
        output_tokens: Some(5),
        ..Default::default()
    };
    let cost = super::pricing::resolve("openrouter", "unknown", &usage).await;
    assert_eq!(cost.micros, Some(42));
    assert!(cost.exact);
}

#[tokio::test]
async fn catalog_price_produces_an_estimate_only_with_real_tokens() {
    let usage = RequestUsage {
        input_tokens: Some(1_000),
        output_tokens: Some(500),
        ..Default::default()
    };
    let cost = super::pricing::resolve("openai", "gpt-4o", &usage).await;
    assert!(cost.micros.is_some_and(|value| value > 0));
    assert!(!cost.exact);

    let incomplete = RequestUsage {
        input_tokens: Some(1_000),
        ..Default::default()
    };
    assert_eq!(
        super::pricing::resolve("openai", "gpt-4o", &incomplete)
            .await
            .micros,
        None
    );
}

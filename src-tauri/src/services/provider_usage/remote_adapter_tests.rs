use super::{remote_api, remote_oauth, remote_parse};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::json;

#[test]
fn openrouter_exposes_periods_limit_and_balance() {
    let parsed = remote_api::parse(
        "openrouter",
        &json!({"data": {
            "usage": 4.5,
            "usage_daily": 1.0,
            "usage_weekly": 3.0,
            "usage_monthly": 4.5,
            "limit": 10.0,
            "limit_remaining": 5.5
        }}),
    )
    .unwrap();
    assert_eq!(parsed.windows.len(), 4);
    assert_eq!(parsed.balances[0].amount, "5.5");
}

#[test]
fn openrouter_preserves_a_negative_remaining_balance() {
    let parsed = remote_api::parse(
        "openrouter",
        &json!({"data": {
            "usage": 10.25,
            "limit": 10.0,
            "limit_remaining": -0.25
        }}),
    )
    .unwrap();
    assert_eq!(parsed.balances[0].amount, "-0.25");
}

#[test]
fn deepseek_preserves_multiple_original_currencies() {
    let parsed = remote_api::parse(
        "deepseek",
        &json!({"balance_infos": [
            {"currency":"USD","total_balance":"1.25"},
            {"currency":"CNY","total_balance":"8.5"}
        ]}),
    )
    .unwrap();
    assert_eq!(parsed.balances.len(), 2);
    assert_eq!(parsed.balances[1].currency, "CNY");
}

#[test]
fn codex_windows_and_credits_are_bounded() {
    let parsed = remote_oauth::parse(
        "codex-oauth",
        &json!({
            "rate_limit": {"primary_window": {
                "used_percent": 25,
                "limit_window_seconds": 18000,
                "reset_at": 1900000000
            }},
            "credits": {"balance": "12.50"},
            "rate_limit_reset_credits": {"available_count": 2}
        }),
    )
    .unwrap();
    assert_eq!(parsed.windows[0].used_percent, Some(25.0));
    assert_eq!(parsed.balances.len(), 2);
}

#[test]
fn codex_preserves_general_and_named_limit_groups() {
    let parsed = remote_oauth::parse(
        "codex-oauth",
        &json!({
            "rate_limit": {"primary_window": {
                "used_percent": 4,
                "limit_window_seconds": 604800,
                "reset_at": 1900000000
            }},
            "additional_rate_limits": [{
                "limit_name": "GPT-5.3-Codex-Spark",
                "metered_feature": "codex_bengalfox",
                "rate_limit": {"primary_window": {
                    "used_percent": 0,
                    "limit_window_seconds": 604800,
                    "reset_at": 1900000100
                }}
            }]
        }),
    )
    .unwrap();

    assert_eq!(parsed.windows.len(), 2);
    assert_eq!(parsed.windows[0].group_code.as_deref(), Some("general"));
    assert_eq!(parsed.windows[0].group_name, None);
    assert_eq!(
        parsed.windows[1].group_code.as_deref(),
        Some("codex_bengalfox")
    );
    assert_eq!(
        parsed.windows[1].group_name.as_deref(),
        Some("GPT-5.3-Codex-Spark")
    );
}

#[test]
fn invalid_rate_headers_are_ignored() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-ratelimit-limit-requests",
        HeaderValue::from_static("secret"),
    );
    assert!(remote_parse::parse_rate_headers("groq", &headers).is_none());
}

#[test]
fn rate_headers_never_replace_another_provider_balance() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-ratelimit-limit-requests",
        HeaderValue::from_static("100"),
    );
    assert!(remote_parse::parse_rate_headers("openrouter", &headers).is_none());
}

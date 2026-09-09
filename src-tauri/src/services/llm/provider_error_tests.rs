use super::*;
use crate::services::llm::route_profile::ErrorPolicy;

#[test]
fn service_tier_rejection_uses_only_closed_structured_fields() {
    let by_param =
        r#"{"error":{"code":"invalid_request_error","param":"service_tier","message":"private"}}"#;
    let by_code = r#"{"error":{"code":"unsupported_service_tier","message":"private"}}"#;

    assert!(is_service_tier_rejection(by_param));
    assert!(is_service_tier_rejection(by_code));
    assert!(!is_service_tier_rejection(
        r#"{"error":{"message":"service tier unavailable"}}"#
    ));
    assert!(!is_service_tier_rejection(
        r#"{"error":{"param":"other","code":"unsupported_other"}}"#
    ));
}

#[test]
fn service_tier_error_code_has_a_stable_public_value() {
    assert_eq!(
        ProviderErrorCode::ServiceTierUnavailable.as_str(),
        "service_tier_unavailable"
    );
}

#[test]
fn classifies_the_exact_moonshot_membership_response() {
    let body = r#"{"error":{"message":"We're unable to verify your membership benefits at this time. Please ensure your membership is active.","type":"invalid_request_error"}}"#;

    assert_eq!(
        classify_http(ErrorPolicy::Moonshot, 402, body),
        ProviderErrorCode::MoonshotMembershipUnverified
    );
}

#[test]
fn classifies_the_exact_xai_spending_limit_code() {
    let body = r#"{"code":"personal-team-blocked:spending-limit","error":"details"}"#;

    assert_eq!(
        classify_http(ErrorPolicy::XaiOauth, 402, body),
        ProviderErrorCode::XaiSubscriptionOrCreditsRequired
    );
}

#[test]
fn similar_or_unknown_responses_remain_generic() {
    let similar = r#"{"error":{"message":"membership active"}}"#;
    let unknown = r#"{"code":"another-code","error":"private details"}"#;

    assert_eq!(
        classify_http(ErrorPolicy::Moonshot, 402, similar),
        ProviderErrorCode::ProviderAccessUnavailable
    );
    assert_eq!(
        classify_http(ErrorPolicy::XaiOauth, 402, unknown),
        ProviderErrorCode::ProviderAccessUnavailable
    );
    assert!(!classify_http(ErrorPolicy::XaiOauth, 402, unknown)
        .as_str()
        .contains("private details"));
}

#[test]
fn provider_specific_codes_cannot_cross_providers() {
    let moonshot = r#"{"error":{"message":"We're unable to verify your membership benefits at this time. Please ensure your membership is active."}}"#;
    let xai = r#"{"code":"personal-team-blocked:spending-limit"}"#;

    assert_eq!(
        classify_http(ErrorPolicy::XaiOauth, 402, moonshot),
        ProviderErrorCode::ProviderAccessUnavailable
    );
    assert_eq!(
        classify_http(ErrorPolicy::Moonshot, 402, xai),
        ProviderErrorCode::ProviderAccessUnavailable
    );
}

#[test]
fn catalog_errors_keep_only_safe_codes() {
    assert_eq!(
        catalog_code(&LlmError::KnownProvider(
            ProviderErrorCode::MoonshotMembershipUnverified
        )),
        ProviderErrorCode::MoonshotMembershipUnverified
    );
    assert_eq!(
        catalog_code(&LlmError::Unauthorized),
        ProviderErrorCode::OAuthReauthenticationRequired
    );
    assert_eq!(
        catalog_code(&LlmError::Network("private network detail".into())),
        ProviderErrorCode::ModelCatalogUnavailable
    );
}

#[test]
fn log_codes_do_not_mislabel_unrelated_statuses() {
    assert_eq!(
        safe_log_code(ErrorPolicy::Moonshot, 429, "private"),
        "rate_limit"
    );
    assert_eq!(
        safe_log_code(ErrorPolicy::Moonshot, 500, "private"),
        "provider_http_error"
    );
}

#[test]
fn retry_policy_is_bounded_and_never_retries_bad_requests() {
    let ollama = crate::services::llm::route_profile::error_policy("ollama").unwrap();
    let openai = crate::services::llm::route_profile::error_policy("openai").unwrap();

    assert_eq!(ollama.max_server_retries(), 10);
    assert!(ollama.allows_server_retry(503, 9));
    assert!(!ollama.allows_server_retry(503, 10));
    assert!(!ollama.allows_server_retry(400, 0));
    assert_eq!(openai.max_server_retries(), 0);
    assert!(!openai.allows_server_retry(503, 0));
}

#[tokio::test]
async fn catalog_rate_limit_preserves_the_retry_after_delay() {
    let response = tauri::http::Response::builder()
        .status(429)
        .header("retry-after", "7")
        .body("")
        .expect("valid fixture response");
    let error = crate::services::llm::openai_compat_parsing::map_error_status(
        reqwest::Response::from(response),
        ErrorPolicy::OpenAiCompatible,
    )
    .await;

    assert!(matches!(
        error,
        crate::services::llm::types::LlmError::RateLimit {
            retry_after_secs: Some(7)
        }
    ));
}

#[test]
fn transport_failures_have_stable_safe_codes() {
    assert_eq!(
        ProviderErrorCode::ProviderConnectionFailed.as_str(),
        "provider_connection_failed"
    );
    assert_eq!(
        ProviderErrorCode::ProviderTemporarilyUnavailable.as_str(),
        "provider_temporarily_unavailable"
    );
    assert_eq!(
        ProviderErrorCode::ProviderRequestRejected.as_str(),
        "provider_request_rejected"
    );
    assert_eq!(
        ProviderErrorCode::ProviderConfigurationInvalid.as_str(),
        "provider_configuration_invalid"
    );
}

#[test]
fn safe_details_keep_only_whitelisted_fields() {
    let details = safe_details(
        r#"{"error":{"type":"invalid_request","code":"bad_schema","param":"tools[0]","message":"private prompt"}}"#,
    );

    assert_eq!(details.error_type.as_deref(), Some("invalid_request"));
    assert_eq!(details.error_code.as_deref(), Some("bad_schema"));
    assert_eq!(details.error_param.as_deref(), Some("tools[0]"));
    assert!(!format!("{details:?}").contains("private prompt"));
}

#[test]
fn openrouter_details_keep_provider_name_but_never_raw_upstream_body() {
    let details = safe_details(
        r#"{"error":{"metadata":{"provider_name":"Google AI Studio","raw":"Bearer secret-sentinel upstream body"}}}"#,
    );
    let value = serde_json::to_value(details).unwrap();

    assert_eq!(value["upstream_provider"], "Google AI Studio");
    let serialized = value.to_string();
    assert!(!serialized.contains("secret-sentinel"));
    assert!(!serialized.contains("upstream body"));
    assert!(!serialized.contains("raw"));
}

#[test]
fn safe_details_extract_google_status_from_single_error_envelopes() {
    let body = r#"{"error":{"code":429,"status":"RESOURCE_EXHAUSTED","message":"private prompt"}}"#;
    for envelope in [body.to_string(), format!("[{body}]")] {
        let details = safe_details(&envelope);
        assert_eq!(details.error_type.as_deref(), Some("RESOURCE_EXHAUSTED"));
        assert_eq!(details.error_code.as_deref(), Some("429"));
        assert!(!format!("{details:?}").contains("private prompt"));
    }
}

#[test]
fn safe_details_keep_quota_categories_but_not_provider_messages_or_identifiers() {
    let details = safe_details(
        r#"[{"error":{"code":429,"status":"RESOURCE_EXHAUSTED","message":"private message","details":[
        {"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[
            {"quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier","quotaValue":"20","quotaDimensions":{"project":"private project"}},
            {"quotaId":"private-secret","quotaValue":"0"}
        ]},
        {"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"12s"}
    ]}}]"#,
    );
    let value = serde_json::to_value(details).unwrap();
    assert_eq!(value["quota"]["requests_per_day"], true);
    assert_eq!(value["quota"]["zero_limit"], true);
    assert_eq!(value["quota"]["retry_after_seconds"], 12);
    assert!(!value.to_string().contains("private"));
}

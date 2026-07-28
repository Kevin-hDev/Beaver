use super::*;

#[test]
fn classifies_the_exact_moonshot_membership_response() {
    let body = r#"{"error":{"message":"We're unable to verify your membership benefits at this time. Please ensure your membership is active.","type":"invalid_request_error"}}"#;

    assert_eq!(
        classify_http("moonshot-oauth", 402, body),
        ProviderErrorCode::MoonshotMembershipUnverified
    );
}

#[test]
fn classifies_the_exact_xai_spending_limit_code() {
    let body = r#"{"code":"personal-team-blocked:spending-limit","error":"details"}"#;

    assert_eq!(
        classify_http("xai-oauth", 402, body),
        ProviderErrorCode::XaiSubscriptionOrCreditsRequired
    );
}

#[test]
fn similar_or_unknown_responses_remain_generic() {
    let similar = r#"{"error":{"message":"membership active"}}"#;
    let unknown = r#"{"code":"another-code","error":"private details"}"#;

    assert_eq!(
        classify_http("moonshot-oauth", 402, similar),
        ProviderErrorCode::ProviderAccessUnavailable
    );
    assert_eq!(
        classify_http("xai-oauth", 402, unknown),
        ProviderErrorCode::ProviderAccessUnavailable
    );
    assert!(!classify_http("xai-oauth", 402, unknown)
        .as_str()
        .contains("private details"));
}

#[test]
fn provider_specific_codes_cannot_cross_providers() {
    let moonshot = r#"{"error":{"message":"We're unable to verify your membership benefits at this time. Please ensure your membership is active."}}"#;
    let xai = r#"{"code":"personal-team-blocked:spending-limit"}"#;

    assert_eq!(
        classify_http("xai-oauth", 402, moonshot),
        ProviderErrorCode::ProviderAccessUnavailable
    );
    assert_eq!(
        classify_http("moonshot-oauth", 402, xai),
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
        safe_log_code("moonshot-oauth", 429, "private"),
        "rate_limit"
    );
    assert_eq!(
        safe_log_code("moonshot-oauth", 500, "private"),
        "provider_http_error"
    );
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

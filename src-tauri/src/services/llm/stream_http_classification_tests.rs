use super::*;

#[test]
fn moonshot_membership_error_has_a_stable_safe_code() {
    let body = r#"{"error":{"message":"We're unable to verify your membership benefits at this time. Please ensure your membership is active.","type":"invalid_request_error"}}"#;
    let error = classify_error(402, body, "Moonshot AI", "moonshot-oauth", true, false);

    assert_eq!(error.to_string(), "moonshot_membership_unverified");
}

#[test]
fn xai_spending_limit_error_has_a_stable_safe_code() {
    let body =
        r#"{"code":"personal-team-blocked:spending-limit","error":"private upstream details"}"#;
    let error = classify_error(402, body, "xAI", "xai-oauth", true, false);

    assert_eq!(error.to_string(), "xai_subscription_or_credits_required");
    assert!(!error.to_string().contains("private upstream details"));
}

#[test]
fn unknown_payment_error_stays_generic() {
    let error = classify_error(
        402,
        r#"{"error":{"message":"private account detail"}}"#,
        "Provider",
        "unknown",
        true,
        false,
    );

    assert_eq!(error.to_string(), "provider_access_unavailable");
    assert!(!error.to_string().contains("private account detail"));
}

#[test]
fn oauth_auth_and_rate_errors_use_frontend_codes() {
    assert_eq!(
        classify_error(401, "", "xAI", "xai-oauth", true, false).to_string(),
        "oauth_reauthentication_required"
    );
    assert_eq!(
        classify_error(403, "", "xAI", "xai-oauth", true, false).to_string(),
        "provider_access_unavailable"
    );
    assert_eq!(
        classify_error(429, "", "xAI", "xai-oauth", true, false).to_string(),
        "rate_limit"
    );
}

#[test]
fn xai_resource_exhausted_without_retry_hint_is_terminal() {
    let error = classify_error(
        429,
        r#"{"code":"resource-exhausted"}"#,
        "xAI",
        "xai-oauth",
        true,
        false,
    );
    assert_eq!(error.to_string(), "provider_quota_exhausted");
    assert_eq!(
        classify_error(
            429,
            r#"{"code":"resource-exhausted"}"#,
            "xAI",
            "xai-oauth",
            true,
            true,
        )
        .to_string(),
        "rate_limit"
    );
}

#[test]
fn payload_too_large_has_a_distinct_stable_code() {
    let error = classify_error(413, "", "Groq", "groq", false, false);

    assert!(matches!(error, RequestError::PayloadTooLarge));
    assert_eq!(error.to_string(), "provider_payload_too_large");
}

#[test]
fn provider_wording_never_disables_tools_silently() {
    let error = classify_error(
        404,
        r#"{"error":{"message":"tool use is unavailable"}}"#,
        "Provider",
        "unknown",
        false,
        false,
    );

    assert_eq!(error.to_string(), "provider_request_rejected");
}

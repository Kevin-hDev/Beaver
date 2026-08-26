pub(super) fn classify_status(status: u16, body: &str, has_retry_after: bool) -> &'static str {
    match status {
        401 => "oauth_reauthentication_required",
        403 => "provider_access_unavailable",
        429 if !has_retry_after
            && crate::services::llm::provider_error::safe_details(body)
                .error_code
                .as_deref()
                == Some("resource-exhausted") =>
        {
            "provider_quota_exhausted"
        }
        429 => "rate_limit",
        500..=599 => "provider_temporarily_unavailable",
        _ => "provider_request_rejected",
    }
}

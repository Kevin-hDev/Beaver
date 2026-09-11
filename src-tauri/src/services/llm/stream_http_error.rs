#[derive(Debug)]
pub enum RequestError {
    Fatal(String),
    PayloadTooLarge,
    InvalidConfiguration,
}

pub(super) async fn read_provider_error(
    mut response: reqwest::Response,
) -> zeroize::Zeroizing<String> {
    let mut routing = super::provider_diagnostics::openrouter::take(&mut response);
    let body = match crate::services::secure_http::read_bounded(
        response,
        crate::services::secure_http::PROVIDER_ERROR_LIMIT,
    )
    .await
    {
        Ok(bytes) => zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => zeroize::Zeroizing::new(String::new()),
    };
    if let Some(routing) = routing.as_mut() {
        if let Ok(value) = serde_json::from_str(&body) {
            routing.observe(&value);
        }
    }
    body
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal(message) => f.write_str(message),
            Self::PayloadTooLarge => f.write_str("provider_payload_too_large"),
            Self::InvalidConfiguration => f.write_str("provider_configuration_invalid"),
        }
    }
}

pub(super) fn classify_error(
    status: u16,
    body: &str,
    _provider_name: &str,
    error_policy: super::route_profile::ErrorPolicy,
    oauth: bool,
    has_retry_after: bool,
) -> RequestError {
    if super::provider_error::is_service_tier_rejection(body) {
        return RequestError::Fatal(
            super::provider_error::ProviderErrorCode::ServiceTierUnavailable
                .as_str()
                .to_string(),
        );
    }
    match status {
        402 => RequestError::Fatal(
            super::provider_error::classify_http(error_policy, status, body)
                .as_str()
                .to_string(),
        ),
        401 if oauth => RequestError::Fatal("oauth_reauthentication_required".into()),
        401 | 403 => RequestError::Fatal(
            super::provider_error::classify_http(error_policy, status, body)
                .as_str()
                .to_string(),
        ),
        413 => RequestError::PayloadTooLarge,
        429 if error_policy == super::route_profile::ErrorPolicy::XaiOauth
            && !has_retry_after
            && super::provider_error::safe_details(body)
                .error_code
                .as_deref()
                == Some("resource-exhausted") =>
        {
            RequestError::Fatal("provider_quota_exhausted".into())
        }
        429 => RequestError::Fatal("rate_limit".into()),
        500..=599 => RequestError::Fatal(
            super::provider_error::ProviderErrorCode::ProviderTemporarilyUnavailable
                .as_str()
                .to_string(),
        ),
        _ => RequestError::Fatal(
            super::provider_error::ProviderErrorCode::ProviderRequestRejected
                .as_str()
                .to_string(),
        ),
    }
}

pub(super) fn request_error_for_limit(
    error: super::stream_max_tokens::ResolveError,
) -> RequestError {
    match error {
        super::stream_max_tokens::ResolveError::ContextExhausted => RequestError::PayloadTooLarge,
        super::stream_max_tokens::ResolveError::InvalidLimit => RequestError::InvalidConfiguration,
    }
}

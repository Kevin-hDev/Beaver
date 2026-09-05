#[derive(Debug)]
pub enum RequestError {
    Fatal(String),
    PayloadTooLarge,
    InvalidConfiguration,
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
        403 if oauth => RequestError::Fatal("provider_access_unavailable".into()),
        401 | 403 => RequestError::Fatal("auth_failed".into()),
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

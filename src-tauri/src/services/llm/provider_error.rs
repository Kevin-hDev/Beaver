use serde::Serialize;

use super::types::LlmError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderErrorCode {
    MoonshotMembershipUnverified,
    XaiSubscriptionOrCreditsRequired,
    OAuthReauthenticationRequired,
    RateLimited,
    ProviderAccessUnavailable,
    ProviderConnectionFailed,
    ProviderTemporarilyUnavailable,
    ProviderRequestRejected,
    ProviderConfigurationInvalid,
    ModelCatalogUnavailable,
    ServiceTierUnavailable,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct SafeProviderDetails {
    pub error_type: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
}

impl ProviderErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MoonshotMembershipUnverified => "moonshot_membership_unverified",
            Self::XaiSubscriptionOrCreditsRequired => "xai_subscription_or_credits_required",
            Self::OAuthReauthenticationRequired => "oauth_reauthentication_required",
            Self::RateLimited => "rate_limit",
            Self::ProviderAccessUnavailable => "provider_access_unavailable",
            Self::ProviderConnectionFailed => "provider_connection_failed",
            Self::ProviderTemporarilyUnavailable => "provider_temporarily_unavailable",
            Self::ProviderRequestRejected => "provider_request_rejected",
            Self::ProviderConfigurationInvalid => "provider_configuration_invalid",
            Self::ModelCatalogUnavailable => "model_catalog_unavailable",
            Self::ServiceTierUnavailable => "service_tier_unavailable",
        }
    }
}

pub fn is_service_tier_rejection(body: &str) -> bool {
    let Ok(document) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    service_tier_error_fields(
        document.pointer("/error/param"),
        document.pointer("/error/code"),
    )
}

pub fn is_service_tier_response_error(event: &serde_json::Value) -> bool {
    service_tier_error_fields(
        event.pointer("/response/error/param"),
        event.pointer("/response/error/code"),
    )
}

fn service_tier_error_fields(
    param: Option<&serde_json::Value>,
    code: Option<&serde_json::Value>,
) -> bool {
    if param.and_then(serde_json::Value::as_str) == Some("service_tier") {
        return true;
    }
    // Hypothèse défensive fermée, à retirer si la campagne réelle ne l'observe pas.
    code.and_then(serde_json::Value::as_str) == Some("unsupported_service_tier")
}

pub fn classify_http(
    policy: super::route_profile::ErrorPolicy,
    status: u16,
    body: &str,
) -> ProviderErrorCode {
    if status != 402 {
        return ProviderErrorCode::ProviderAccessUnavailable;
    }
    let parsed = serde_json::from_str::<serde_json::Value>(body).ok();
    if policy == super::route_profile::ErrorPolicy::Moonshot
        && parsed
            .as_ref()
            .and_then(|value| value.pointer("/error/message"))
            .and_then(serde_json::Value::as_str)
            == Some(MOONSHOT_MEMBERSHIP_MESSAGE)
    {
        return ProviderErrorCode::MoonshotMembershipUnverified;
    }
    if matches!(
        policy,
        super::route_profile::ErrorPolicy::Xai | super::route_profile::ErrorPolicy::XaiOauth
    ) && parsed
        .as_ref()
        .and_then(|value| value.get("code"))
        .and_then(serde_json::Value::as_str)
        == Some(XAI_SPENDING_LIMIT_CODE)
    {
        return ProviderErrorCode::XaiSubscriptionOrCreditsRequired;
    }
    ProviderErrorCode::ProviderAccessUnavailable
}

pub fn safe_log_code(
    policy: super::route_profile::ErrorPolicy,
    status: u16,
    body: &str,
) -> &'static str {
    match status {
        401 => "authentication_required",
        402 => classify_http(policy, status, body).as_str(),
        403 => "provider_access_unavailable",
        429 => "rate_limit",
        _ => "provider_http_error",
    }
}

pub fn safe_details(body: &str) -> SafeProviderDetails {
    let parsed = serde_json::from_str::<serde_json::Value>(body).ok();
    SafeProviderDetails {
        error_type: json_field(parsed.as_ref(), &["/error/type", "/type"]),
        error_code: json_field(parsed.as_ref(), &["/error/code", "/code"]),
        error_param: json_field(parsed.as_ref(), &["/error/param", "/param"]),
    }
}

fn json_field(document: Option<&serde_json::Value>, pointers: &[&str]) -> Option<String> {
    const MAX_SAFE_FIELD_CHARS: usize = 128;
    pointers.iter().find_map(|pointer| {
        let value = document?.pointer(pointer)?;
        let text = match value {
            serde_json::Value::String(text) => text.clone(),
            serde_json::Value::Number(number) => number.to_string(),
            _ => return None,
        };
        let clipped: String = text.chars().take(MAX_SAFE_FIELD_CHARS + 1).collect();
        (clipped.chars().count() <= MAX_SAFE_FIELD_CHARS
            && !clipped.is_empty()
            && clipped.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '_' | '-' | '.' | '/' | '[' | ']')
            }))
        .then_some(clipped)
    })
}

pub fn catalog_code(error: &LlmError) -> ProviderErrorCode {
    match error {
        LlmError::KnownProvider(code) => *code,
        LlmError::Unauthorized => ProviderErrorCode::OAuthReauthenticationRequired,
        LlmError::RateLimit { .. } => ProviderErrorCode::RateLimited,
        _ => ProviderErrorCode::ModelCatalogUnavailable,
    }
}

const MOONSHOT_MEMBERSHIP_MESSAGE: &str =
    "We're unable to verify your membership benefits at this time. Please ensure your membership is active.";
const XAI_SPENDING_LIMIT_CODE: &str = "personal-team-blocked:spending-limit";

#[cfg(test)]
#[path = "provider_error_tests.rs"]
mod tests;

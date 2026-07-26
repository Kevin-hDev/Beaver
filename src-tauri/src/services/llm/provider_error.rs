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
        }
    }
}

pub fn classify_http(provider_id: &str, status: u16, body: &str) -> ProviderErrorCode {
    if status != 402 {
        return ProviderErrorCode::ProviderAccessUnavailable;
    }
    let parsed = serde_json::from_str::<serde_json::Value>(body).ok();
    if is_moonshot(provider_id)
        && parsed
            .as_ref()
            .and_then(|value| value.pointer("/error/message"))
            .and_then(serde_json::Value::as_str)
            == Some(MOONSHOT_MEMBERSHIP_MESSAGE)
    {
        return ProviderErrorCode::MoonshotMembershipUnverified;
    }
    if is_xai(provider_id)
        && parsed
            .as_ref()
            .and_then(|value| value.get("code"))
            .and_then(serde_json::Value::as_str)
            == Some(XAI_SPENDING_LIMIT_CODE)
    {
        return ProviderErrorCode::XaiSubscriptionOrCreditsRequired;
    }
    ProviderErrorCode::ProviderAccessUnavailable
}

pub fn safe_log_code(provider_id: &str, status: u16, body: &str) -> &'static str {
    match status {
        401 => "authentication_required",
        402 => classify_http(provider_id, status, body).as_str(),
        403 => "provider_access_unavailable",
        429 => "rate_limit",
        _ => "provider_http_error",
    }
}

pub fn catalog_code(error: &LlmError) -> ProviderErrorCode {
    match error {
        LlmError::KnownProvider(code) => *code,
        LlmError::Unauthorized => ProviderErrorCode::OAuthReauthenticationRequired,
        LlmError::RateLimit { .. } => ProviderErrorCode::RateLimited,
        _ => ProviderErrorCode::ModelCatalogUnavailable,
    }
}

fn is_moonshot(provider_id: &str) -> bool {
    matches!(provider_id, "moonshot" | "moonshot-oauth")
}

fn is_xai(provider_id: &str) -> bool {
    matches!(provider_id, "xai" | "xai-oauth")
}

const MOONSHOT_MEMBERSHIP_MESSAGE: &str =
    "We're unable to verify your membership benefits at this time. Please ensure your membership is active.";
const XAI_SPENDING_LIMIT_CODE: &str = "personal-team-blocked:spending-limit";

#[cfg(test)]
#[path = "provider_error_tests.rs"]
mod tests;

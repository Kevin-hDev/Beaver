use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmOAuthProvider {
    Xai,
    Kimi,
}

impl LlmOAuthProvider {
    pub const fn index(self) -> usize {
        match self {
            Self::Xai => 0,
            Self::Kimi => 1,
        }
    }

    pub const fn provider_id(self) -> &'static str {
        match self {
            Self::Xai => "xai-oauth",
            Self::Kimi => "moonshot-oauth",
        }
    }

    pub const fn vault_key(self) -> &'static str {
        match self {
            Self::Xai => "_llm_oauth_xai",
            Self::Kimi => "_llm_oauth_kimi",
        }
    }
}

pub struct TokenBundle {
    pub access: Zeroizing<String>,
    pub refresh: Zeroizing<String>,
    pub expires_at: i64,
}

impl TokenBundle {
    pub fn is_fresh(&self) -> bool {
        chrono::Utc::now().timestamp() < self.expires_at.saturating_sub(60)
    }
}

pub struct AccessToken {
    pub value: Zeroizing<String>,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthFailure {
    Cancelled,
    Denied,
    Expired,
    Unauthorized,
    Generic,
}

pub struct DeviceAuthorization {
    pub device_code: Zeroizing<String>,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub interval_seconds: u64,
    pub expires_in_seconds: u64,
}

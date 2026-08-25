use zeroize::Zeroizing;

use crate::services::reasoning_continuity::contract::{CredentialScope, RouteId};

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
            Self::Xai => crate::services::api_keys::LLM_OAUTH_XAI_KEY,
            Self::Kimi => crate::services::api_keys::LLM_OAUTH_KIMI_KEY,
        }
    }

    pub const fn reasoning_route(self) -> RouteId {
        match self {
            Self::Xai => RouteId::XaiOauth,
            Self::Kimi => RouteId::MoonshotOauth,
        }
    }
}

pub struct TokenBundle {
    pub access: Zeroizing<String>,
    pub refresh: Zeroizing<String>,
    pub expires_at: i64,
    pub user_id: Option<Zeroizing<String>>,
    pub credential_scope: Option<CredentialScope>,
}

impl TokenBundle {
    pub fn is_fresh(&self) -> bool {
        chrono::Utc::now().timestamp() < self.expires_at.saturating_sub(60)
    }

    pub(crate) fn assign_new_credential_scope(&mut self) -> Result<(), String> {
        self.credential_scope = Some(crate::services::api_keys::generate_credential_scope()?);
        Ok(())
    }

    pub(crate) fn ensure_credential_scope_for_persistence(&mut self) -> Result<(), String> {
        if self.credential_scope.is_none() {
            self.credential_scope = Some(crate::services::api_keys::generate_credential_scope()?);
        }
        Ok(())
    }

    pub(crate) fn preserve_credential_scope_from(&mut self, current: &Self) -> Result<(), String> {
        self.credential_scope = current.credential_scope.clone();
        self.ensure_credential_scope_for_persistence()
    }
}

pub struct AccessToken {
    pub value: Zeroizing<String>,
    pub generation: u64,
    pub user_id: Option<Zeroizing<String>>,
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

use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

use super::jwt;
use super::store::CodexTokens;

const DEFAULT_EXPIRES_IN_SECS: i64 = 3_600;
const MAX_EXPIRES_IN_SECS: i64 = 86_400;
const REFRESH_RETRY_DELAY_SECS: i64 = 60;

#[derive(Deserialize)]
pub(super) struct CodexTokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

impl Drop for CodexTokenResponse {
    fn drop(&mut self) {
        if let Some(value) = &mut self.access_token {
            value.zeroize();
        }
        if let Some(value) = &mut self.refresh_token {
            value.zeroize();
        }
    }
}

pub(super) fn from_exchange(mut raw: CodexTokenResponse) -> Result<CodexTokens, String> {
    let access = take_required(&mut raw.access_token)?;
    let refresh = take_required(&mut raw.refresh_token)?;
    let claims = jwt::extract_display_claims(&access)?;
    let expires_at = claims
        .expires_at
        .unwrap_or_else(|| fallback_expiry(raw.expires_in));
    Ok(CodexTokens {
        access,
        refresh,
        expires_at,
        refresh_not_before: 0,
        account_hint: Zeroizing::new(claims.account_hint),
        credential_scope: Some(
            crate::services::api_keys::generate_credential_scope().map_err(|_| invalid())?,
        ),
    })
}

pub(super) fn from_refresh(
    mut raw: CodexTokenResponse,
    current: &CodexTokens,
) -> Result<CodexTokens, String> {
    let access = take_optional(&mut raw.access_token)?;
    let refresh = take_optional(&mut raw.refresh_token)?;
    let (access, expires_at, refresh_not_before, account_hint) = match access {
        Some(access) => {
            let claims = jwt::extract_display_claims(&access)?;
            let expires_at = claims
                .expires_at
                .unwrap_or_else(|| fallback_expiry(raw.expires_in));
            (access, expires_at, 0, Zeroizing::new(claims.account_hint))
        }
        None => (
            current.access.clone(),
            current.expires_at,
            chrono::Utc::now()
                .timestamp()
                .saturating_add(REFRESH_RETRY_DELAY_SECS),
            current.account_hint.clone(),
        ),
    };
    Ok(CodexTokens {
        access,
        refresh: refresh.unwrap_or_else(|| current.refresh.clone()),
        expires_at,
        refresh_not_before,
        account_hint,
        credential_scope: current.credential_scope.clone(),
    })
}

fn take_required(value: &mut Option<String>) -> Result<Zeroizing<String>, String> {
    take_optional(value)?.ok_or_else(invalid)
}

fn take_optional(value: &mut Option<String>) -> Result<Option<Zeroizing<String>>, String> {
    let Some(value) = value.take() else {
        return Ok(None);
    };
    if value.is_empty() {
        return Err(invalid());
    }
    Ok(Some(Zeroizing::new(value)))
}

fn fallback_expiry(expires_in: Option<i64>) -> i64 {
    let expires_in = expires_in
        .unwrap_or(DEFAULT_EXPIRES_IN_SECS)
        .clamp(1, MAX_EXPIRES_IN_SECS);
    chrono::Utc::now().timestamp().saturating_add(expires_in)
}

fn invalid() -> String {
    "réponse OAuth invalide".to_string()
}

#[cfg(test)]
#[path = "token_response_tests.rs"]
mod tests;

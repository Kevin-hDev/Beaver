use zeroize::Zeroizing;

use crate::services::api_keys;
use crate::services::reasoning_continuity::contract::CredentialScope;

const REFRESH_WINDOW_SECS: i64 = 5 * 60;
const MAX_ACCESS_TOKEN_BYTES: usize = 512 * 1024;
const MAX_REFRESH_TOKEN_BYTES: usize = 64 * 1024;
const MAX_ACCOUNT_HINT_BYTES: usize = 128;

pub struct CodexTokens {
    pub access: Zeroizing<String>,
    pub refresh: Zeroizing<String>,
    pub expires_at: i64,
    pub(crate) refresh_not_before: i64,
    /// Indice de routage non vérifié. Le serveur valide le bearer token.
    pub account_hint: Zeroizing<String>,
    pub credential_scope: Option<CredentialScope>,
}

impl CodexTokens {
    pub fn needs_refresh(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= self.expires_at - REFRESH_WINDOW_SECS
            && (now >= self.refresh_not_before || now >= self.expires_at)
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.expires_at
    }
}

pub fn save(tokens: &CodexTokens) -> Result<(), String> {
    validate(tokens)?;
    let scope = tokens.credential_scope.clone().ok_or_else(unavailable)?;
    let raw = api_keys::new_codex_oauth_record(
        tokens.access.to_string(),
        tokens.refresh.to_string(),
        tokens.expires_at,
        tokens.refresh_not_before,
        tokens.account_hint.as_str().to_string(),
        scope,
    );
    let json = api_keys::encode_codex_oauth_record(&raw)?;
    api_keys::set_raw(api_keys::CODEX_OAUTH_KEY, &json)
}

pub fn load() -> Result<Option<CodexTokens>, String> {
    if !api_keys::has_raw(api_keys::CODEX_OAUTH_KEY).map_err(|_| unavailable())? {
        return Ok(None);
    }
    let json = api_keys::get_raw(api_keys::CODEX_OAUTH_KEY).map_err(|_| unavailable())?;
    let mut raw = api_keys::decode_codex_oauth_record(&json)?;
    let tokens = CodexTokens {
        access: Zeroizing::new(std::mem::take(&mut raw.access)),
        refresh: Zeroizing::new(std::mem::take(&mut raw.refresh)),
        expires_at: raw.expires_at,
        refresh_not_before: raw.refresh_not_before,
        account_hint: Zeroizing::new(std::mem::take(&mut raw.account_hint)),
        credential_scope: raw.credential_scope.take(),
    };
    validate(&tokens)?;
    Ok(Some(tokens))
}

pub fn clear() -> Result<(), String> {
    api_keys::delete_raw(api_keys::CODEX_OAUTH_KEY)
}

pub fn is_logged_in() -> bool {
    load().ok().flatten().is_some()
}

fn unavailable() -> String {
    "oauth_reauthentication_required".to_string()
}

fn validate(tokens: &CodexTokens) -> Result<(), String> {
    let valid = !tokens.access.is_empty()
        && tokens.access.len() <= MAX_ACCESS_TOKEN_BYTES
        && !tokens.refresh.is_empty()
        && tokens.refresh.len() <= MAX_REFRESH_TOKEN_BYTES
        && !tokens.account_hint.is_empty()
        && tokens.account_hint.len() <= MAX_ACCOUNT_HINT_BYTES
        && tokens
            .account_hint
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        && tokens.expires_at > 0
        && tokens.refresh_not_before >= 0;
    if valid {
        Ok(())
    } else {
        Err(unavailable())
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;

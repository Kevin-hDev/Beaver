use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::services::api_keys;

const VAULT_KEY: &str = "_codex_oauth";
const REFRESH_WINDOW_SECS: i64 = 5 * 60;
const MAX_ACCESS_TOKEN_BYTES: usize = 512 * 1024;
const MAX_REFRESH_TOKEN_BYTES: usize = 64 * 1024;
const MAX_ACCOUNT_HINT_BYTES: usize = 128;

#[derive(Serialize, Deserialize)]
struct Stored {
    access: String,
    refresh: String,
    expires_at: i64,
    #[serde(default)]
    refresh_not_before: i64,
    #[serde(rename = "account_id")]
    account_hint: String,
}

impl Drop for Stored {
    fn drop(&mut self) {
        self.access.zeroize();
        self.refresh.zeroize();
        self.account_hint.zeroize();
    }
}

pub struct CodexTokens {
    pub access: Zeroizing<String>,
    pub refresh: Zeroizing<String>,
    pub expires_at: i64,
    pub(crate) refresh_not_before: i64,
    /// Indice de routage non vérifié. Le serveur valide le bearer token.
    pub account_hint: Zeroizing<String>,
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
    let raw = Stored {
        access: tokens.access.to_string(),
        refresh: tokens.refresh.to_string(),
        expires_at: tokens.expires_at,
        refresh_not_before: tokens.refresh_not_before,
        account_hint: tokens.account_hint.as_str().to_string(),
    };
    let mut json = serde_json::to_string(&raw).map_err(|_| unavailable())?;
    let result = api_keys::set_raw(VAULT_KEY, &json);
    json.zeroize();
    result
}

pub fn load() -> Result<Option<CodexTokens>, String> {
    if !api_keys::has_raw(VAULT_KEY).map_err(|_| unavailable())? {
        return Ok(None);
    }
    let json = api_keys::get_raw(VAULT_KEY).map_err(|_| unavailable())?;
    let mut raw: Stored = serde_json::from_str(&json).map_err(|_| unavailable())?;
    let tokens = CodexTokens {
        access: Zeroizing::new(std::mem::take(&mut raw.access)),
        refresh: Zeroizing::new(std::mem::take(&mut raw.refresh)),
        expires_at: raw.expires_at,
        refresh_not_before: raw.refresh_not_before,
        account_hint: Zeroizing::new(std::mem::take(&mut raw.account_hint)),
    };
    validate(&tokens)?;
    Ok(Some(tokens))
}

pub fn clear() -> Result<(), String> {
    api_keys::delete_raw(VAULT_KEY)
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

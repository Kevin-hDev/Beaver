use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::services::api_keys;

const VAULT_KEY: &str = "_codex_oauth";
const REFRESH_WINDOW_SECS: i64 = 5 * 60;

#[derive(Serialize, Deserialize)]
struct Stored {
    access: String,
    refresh: String,
    expires_at: i64,
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
    /// Indice de routage non vérifié. Le serveur valide le bearer token.
    pub account_hint: Zeroizing<String>,
}

impl CodexTokens {
    pub fn needs_refresh(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.expires_at - REFRESH_WINDOW_SECS
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.expires_at
    }
}

pub fn save(tokens: &CodexTokens) -> Result<(), String> {
    let raw = Stored {
        access: tokens.access.to_string(),
        refresh: tokens.refresh.to_string(),
        expires_at: tokens.expires_at,
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
    if raw.access.is_empty()
        || raw.refresh.is_empty()
        || raw.account_hint.is_empty()
        || raw.expires_at <= 0
    {
        return Err(unavailable());
    }
    Ok(Some(CodexTokens {
        access: Zeroizing::new(std::mem::take(&mut raw.access)),
        refresh: Zeroizing::new(std::mem::take(&mut raw.refresh)),
        expires_at: raw.expires_at,
        account_hint: Zeroizing::new(std::mem::take(&mut raw.account_hint)),
    }))
}

pub fn clear() -> Result<(), String> {
    api_keys::delete_raw(VAULT_KEY)
}

pub fn is_logged_in() -> bool {
    load().ok().flatten().is_some()
}

fn unavailable() -> String {
    "Connexion Codex indisponible".to_string()
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;

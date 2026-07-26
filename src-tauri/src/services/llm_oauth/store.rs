use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use super::{LlmOAuthProvider, TokenBundle};
use crate::services::api_keys;

const MAX_TOKEN_LEN: usize = 4_096;
static GENERATIONS: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];

#[derive(Serialize, Deserialize)]
struct StoredTokens {
    access: String,
    refresh: String,
    expires_at: i64,
}

impl Drop for StoredTokens {
    fn drop(&mut self) {
        self.access.zeroize();
        self.refresh.zeroize();
    }
}

pub fn save_if_generation(
    provider: LlmOAuthProvider,
    tokens: &TokenBundle,
    expected_generation: u64,
) -> Result<u64, String> {
    if generation(provider) != expected_generation {
        return Err("Connexion modifiée".to_string());
    }
    save(provider, tokens)
}

fn save(provider: LlmOAuthProvider, tokens: &TokenBundle) -> Result<u64, String> {
    validate(tokens)?;
    let stored = StoredTokens {
        access: tokens.access.to_string(),
        refresh: tokens.refresh.to_string(),
        expires_at: tokens.expires_at,
    };
    let mut json = serde_json::to_string(&stored).map_err(|_| unavailable())?;
    let result = api_keys::set_raw(provider.vault_key(), &json);
    json.zeroize();
    result?;
    Ok(GENERATIONS[provider.index()].fetch_add(1, Ordering::SeqCst) + 1)
}

pub fn load(provider: LlmOAuthProvider) -> Result<Option<TokenBundle>, String> {
    let json = match api_keys::get_raw(provider.vault_key()) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let mut stored: StoredTokens = serde_json::from_str(&json).map_err(|_| unavailable())?;
    let tokens = TokenBundle {
        access: Zeroizing::new(std::mem::take(&mut stored.access)),
        refresh: Zeroizing::new(std::mem::take(&mut stored.refresh)),
        expires_at: stored.expires_at,
    };
    validate(&tokens)?;
    Ok(Some(tokens))
}

pub fn clear(provider: LlmOAuthProvider) -> Result<(), String> {
    api_keys::delete_raw(provider.vault_key())?;
    GENERATIONS[provider.index()].fetch_add(1, Ordering::SeqCst);
    Ok(())
}

pub fn generation(provider: LlmOAuthProvider) -> u64 {
    GENERATIONS[provider.index()].load(Ordering::SeqCst)
}

fn validate(tokens: &TokenBundle) -> Result<(), String> {
    if !(1..=MAX_TOKEN_LEN).contains(&tokens.access.len())
        || !(1..=MAX_TOKEN_LEN).contains(&tokens.refresh.len())
        || tokens.expires_at <= 0
    {
        return Err(unavailable());
    }
    Ok(())
}

fn unavailable() -> String {
    "Connexion indisponible".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_tokens_before_storage() {
        let tokens = TokenBundle {
            access: Zeroizing::new("a".repeat(MAX_TOKEN_LEN + 1)),
            refresh: Zeroizing::new("r".to_string()),
            expires_at: 1,
        };
        assert!(validate(&tokens).is_err());
    }

    #[test]
    fn providers_use_distinct_vault_entries() {
        assert_ne!(
            LlmOAuthProvider::Xai.vault_key(),
            LlmOAuthProvider::Kimi.vault_key()
        );
        assert!(LlmOAuthProvider::Xai.vault_key().starts_with('_'));
        assert!(LlmOAuthProvider::Kimi.vault_key().starts_with('_'));
    }

    #[test]
    fn stale_generation_is_rejected_before_storage() {
        let provider = LlmOAuthProvider::Kimi;
        let tokens = TokenBundle {
            access: Zeroizing::new("access".to_string()),
            refresh: Zeroizing::new("refresh".to_string()),
            expires_at: 1,
        };
        let stale = generation(provider).saturating_add(1);
        assert!(save_if_generation(provider, &tokens, stale).is_err());
    }
}

use std::sync::atomic::{AtomicU64, Ordering};

use zeroize::Zeroizing;

use super::{LlmOAuthProvider, TokenBundle};
use crate::services::api_keys;

const MAX_TOKEN_LEN: usize = 4_096;
static GENERATIONS: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];

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
    let scope = tokens.credential_scope.clone().ok_or_else(unavailable)?;
    let stored = api_keys::new_llm_oauth_record(
        tokens.access.to_string(),
        tokens.refresh.to_string(),
        tokens.expires_at,
        tokens.user_id.as_ref().map(|value| value.to_string()),
        scope,
    );
    let json = api_keys::encode_llm_oauth_record(&stored, provider.reasoning_route())?;
    api_keys::set_raw(provider.vault_key(), &json)?;
    Ok(GENERATIONS[provider.index()].fetch_add(1, Ordering::SeqCst) + 1)
}

pub fn load(provider: LlmOAuthProvider) -> Result<Option<TokenBundle>, String> {
    let json = match api_keys::get_raw(provider.vault_key()) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let mut stored = api_keys::decode_llm_oauth_record(&json, provider.reasoning_route())?;
    let tokens = TokenBundle {
        access: Zeroizing::new(std::mem::take(&mut stored.access)),
        refresh: Zeroizing::new(std::mem::take(&mut stored.refresh)),
        expires_at: stored.expires_at,
        user_id: stored.user_id.take().map(Zeroizing::new),
        credential_scope: stored.credential_scope.take(),
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
        || tokens
            .user_id
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.len() > 256)
    {
        return Err(unavailable());
    }
    Ok(())
}

fn unavailable() -> String {
    "provider_configuration_invalid".to_string()
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
            user_id: None,
            credential_scope: None,
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
            user_id: None,
            credential_scope: None,
        };
        let stale = generation(provider).saturating_add(1);
        assert!(save_if_generation(provider, &tokens, stale).is_err());
    }

    #[test]
    fn login_rotates_scope_and_refresh_preserves_it() {
        let mut login = TokenBundle {
            access: Zeroizing::new("access".to_string()),
            refresh: Zeroizing::new("refresh".to_string()),
            expires_at: 1,
            user_id: None,
            credential_scope: None,
        };
        login.assign_new_credential_scope().unwrap();
        let first = login.credential_scope.clone().unwrap();

        let mut refreshed = TokenBundle {
            access: Zeroizing::new("new-access".to_string()),
            refresh: Zeroizing::new("refresh".to_string()),
            expires_at: 2,
            user_id: None,
            credential_scope: None,
        };
        refreshed.preserve_credential_scope_from(&login);
        assert_eq!(refreshed.credential_scope.as_ref(), Some(&first));

        let mut next_login = refreshed;
        next_login.assign_new_credential_scope().unwrap();
        assert_ne!(next_login.credential_scope.as_ref(), Some(&first));
    }
}

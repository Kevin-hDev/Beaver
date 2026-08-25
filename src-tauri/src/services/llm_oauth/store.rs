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
    save_record_with(provider, tokens, api_keys::set_raw)?;
    Ok(GENERATIONS[provider.index()].fetch_add(1, Ordering::SeqCst) + 1)
}

fn save_record_with<P>(
    provider: LlmOAuthProvider,
    tokens: &TokenBundle,
    persist: P,
) -> Result<(), String>
where
    P: FnOnce(&str, &str) -> Result<(), String>,
{
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
    persist(provider.vault_key(), &json)
}

pub fn load(provider: LlmOAuthProvider) -> Result<Option<TokenBundle>, String> {
    let json = match api_keys::get_raw(provider.vault_key()) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    load_record(provider, &json).map(Some)
}

fn load_record(provider: LlmOAuthProvider, json: &str) -> Result<TokenBundle, String> {
    let mut stored = api_keys::decode_llm_oauth_record(json, provider.reasoning_route())?;
    let tokens = TokenBundle {
        access: Zeroizing::new(std::mem::take(&mut stored.access)),
        refresh: Zeroizing::new(std::mem::take(&mut stored.refresh)),
        expires_at: stored.expires_at,
        user_id: stored.user_id.take().map(Zeroizing::new),
        credential_scope: stored.credential_scope.take(),
    };
    validate(&tokens)?;
    Ok(tokens)
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
#[path = "store_tests.rs"]
mod tests;

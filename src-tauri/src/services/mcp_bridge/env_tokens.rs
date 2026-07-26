use serde::Deserialize;
use std::collections::HashSet;
use zeroize::{Zeroize, Zeroizing};

use super::config::{self, StoredConnector};

const MAX_SECRET_BYTES: usize = 8192;

#[derive(Deserialize)]
pub struct EnvTokenInput {
    pub env_key: String,
    pub value: String,
}

impl Drop for EnvTokenInput {
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

pub fn validate(connector: &StoredConnector, tokens: &[EnvTokenInput]) -> Result<(), String> {
    config::validate_connector(connector)?;
    if connector.endpoint.is_some() || connector.install_command.is_none() {
        return Err("configuration MCP invalide".to_string());
    }
    let expected = config::validated_env_keys(connector.env_keys.as_deref())?;
    if tokens.len() != expected.len() || tokens.is_empty() {
        return Err("secrets MCP invalides".to_string());
    }
    let mut provided = HashSet::with_capacity(tokens.len());
    for token in tokens {
        if !provided.insert(token.env_key.as_str())
            || !expected.iter().any(|key| key == &token.env_key)
            || token.value.is_empty()
            || token.value.len() > MAX_SECRET_BYTES
            || token.value.chars().any(char::is_control)
        {
            return Err("secrets MCP invalides".to_string());
        }
    }
    Ok(())
}

pub fn owned_pairs(tokens: &[EnvTokenInput]) -> Vec<(String, Zeroizing<String>)> {
    tokens
        .iter()
        .map(|token| (token.env_key.clone(), Zeroizing::new(token.value.clone())))
        .collect()
}

pub fn vault_key(connector_id: &str, env_key: &str) -> String {
    format!("mcp_{connector_id}_{}", env_key.to_lowercase())
}

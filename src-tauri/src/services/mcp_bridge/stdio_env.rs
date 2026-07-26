use zeroize::Zeroizing;

use super::env_tokens;
use super::stdio::StdioTransport;

impl StdioTransport {
    pub fn new(connector_id: String, install_command: String, env_key_names: Vec<String>) -> Self {
        Self {
            connector_id,
            install_command,
            env_key_names,
            transient_env: None,
        }
    }

    pub fn new_with_env(
        connector_id: String,
        install_command: String,
        env_key_names: Vec<String>,
        transient_env: Vec<(String, Zeroizing<String>)>,
    ) -> Self {
        Self {
            connector_id,
            install_command,
            env_key_names,
            transient_env: Some(transient_env),
        }
    }

    pub(super) fn resolve_env_tokens(&self) -> Vec<(String, Zeroizing<String>)> {
        if let Some(tokens) = &self.transient_env {
            return tokens.clone();
        }
        self.env_key_names
            .iter()
            .filter_map(|key| {
                let vault_key = env_tokens::vault_key(&self.connector_id, key);
                crate::services::api_keys::get_raw(&vault_key)
                    .or_else(|_| crate::services::api_keys::get_key(&vault_key))
                    .ok()
                    .map(|value| (key.clone(), value))
            })
            .collect()
    }
}

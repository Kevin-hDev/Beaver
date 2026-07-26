use std::collections::HashMap;
use zeroize::{Zeroize, Zeroizing};

use super::vault;
pub(crate) mod validate {
    include!("api_keys_validate.rs");
}

include!("api_keys_state.rs");
include!("api_keys_registry.rs");
include!("api_keys_transactions.rs");

fn migrate_raw_prefix(
    master_key: &Zeroizing<Vec<u8>>,
    map: &mut HashMap<String, String>,
) -> Result<(), String> {
    let to_migrate: Vec<String> = map
        .keys()
        .filter(|k| k.starts_with('_') && !k.starts_with(RAW_PREFIX))
        .cloned()
        .collect();
    if to_migrate.is_empty() {
        return Ok(());
    }
    for old_key in &to_migrate {
        let new_key = format!("{RAW_PREFIX}{old_key}");
        if let Some(val) = map.remove(old_key) {
            map.insert(new_key, val);
        }
    }
    vault::write_vault(master_key, map)?;
    eprintln!(
        "[vault] migrated {} raw keys to namespaced prefix",
        to_migrate.len()
    );
    Ok(())
}

pub fn init() -> Result<(), String> {
    let master_key = vault::load_or_create_master_key()?;
    let mut raw_map = ZeroizingMap(vault::read_vault(&master_key)?);
    let marker = vault::vault_path().with_file_name(".vault-migrated");
    if !marker.exists() {
        let legacy = vault::read_legacy_keychain_keys();
        if !legacy.is_empty() {
            for (id, key) in &legacy {
                raw_map
                    .0
                    .entry(id.clone())
                    .or_insert_with(|| key.to_string());
            }
            eprintln!("[vault] migrated {} keys from keychain", legacy.len());
        }
        vault::write_vault(&master_key, &raw_map.0)?;
        crate::services::private_store::atomic_write(&marker, b"ok")?;
    }
    migrate_raw_prefix(&master_key, &mut raw_map.0)?;
    write_registry(&provider_ids(raw_map.0.keys().map(String::as_str)))?;
    let keys = raw_map
        .0
        .drain()
        .map(|(k, v)| (k, Zeroizing::new(v)))
        .collect();
    let mut state = STATE
        .lock()
        .map_err(|_| "coffre indisponible".to_string())?;
    *state = Some(VaultState { master_key, keys });
    Ok(())
}

pub fn get_key(provider_id: &str) -> Result<Zeroizing<String>, String> {
    let state = STATE
        .lock()
        .map_err(|_| "coffre indisponible".to_string())?;
    let s = state.as_ref().ok_or("coffre indisponible")?;
    s.keys
        .get(provider_id)
        .cloned()
        .ok_or_else(|| "clé non trouvée".to_string())
}

pub fn set_key(provider_id: &str, key: &str) -> Result<(), String> {
    validate::validate_key_input(provider_id, key)?;
    transaction(|candidate| {
        candidate.insert(provider_id.to_string(), key.to_string());
        Ok(())
    })?;
    sync_registry_cache();
    Ok(())
}

pub fn delete_key(provider_id: &str) -> Result<(), String> {
    validate::validate_provider(provider_id)?;
    transaction(|candidate| {
        candidate.remove(provider_id);
        Ok(())
    })?;
    sync_registry_cache();
    Ok(())
}

include!("api_keys_raw.rs");

include!("api_keys_http.rs");
include!("api_keys_mcp.rs");

#[cfg(test)]
#[path = "api_keys_validate_tests.rs"]
mod validate_tests;

#[cfg(test)]
#[path = "api_keys_mcp_tests.rs"]
mod mcp_tests;

#[cfg(test)]
#[path = "api_keys_transaction_tests.rs"]
mod transaction_tests;

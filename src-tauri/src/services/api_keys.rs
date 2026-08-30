use std::collections::HashMap;
use zeroize::{Zeroize, Zeroizing};

use super::vault;
pub(crate) mod validate {
    include!("api_keys_validate.rs");
}

include!("api_keys_state.rs");
include!("api_keys_registry.rs");
include!("api_keys_transactions.rs");
#[cfg(any(not(feature = "e2e"), test))]
include!("api_keys_retired.rs");
include!("api_keys_credential_scope_wire.rs");
include!("api_keys_credential_scope.rs");
include!("api_keys_credential_scope_migration.rs");

#[cfg(not(feature = "e2e"))]
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
    ::log::info!(
        "[vault] migrated {} raw keys to namespaced prefix",
        to_migrate.len()
    );
    Ok(())
}

#[cfg(not(feature = "e2e"))]
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
            ::log::info!("[vault] migrated {} keys from keychain", legacy.len());
        }
        vault::write_vault(&master_key, &raw_map.0)?;
        crate::services::private_store::atomic_write(&marker, b"ok")?;
    }
    migrate_raw_prefix(&master_key, &mut raw_map.0)?;
    let migration = commit_credential_scope_migration_with(&mut raw_map.0, |candidate| {
        vault::write_vault(&master_key, candidate)
    });
    for route in migration.blocked {
        ::log::warn!(
            "route={} decision=blocked reason=scope_migration_failed",
            credential_scope_route_label(route)
        );
    }
    let remove_retired_backup = prepare_retired_provider_cleanup(&master_key, &mut raw_map.0)?;
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
    drop(state);
    if crate::services::attachment_access::ensure_attachment_key().is_err() {
        log::warn!("attachment_access_key_unavailable");
    }
    if crate::services::reasoning_continuity::fingerprint::ensure_fingerprint_key().is_err() {
        log::warn!("reasoning_diagnostic_key_unavailable");
    }
    finish_retired_provider_cleanup(remove_retired_backup)?;
    Ok(())
}

#[cfg(feature = "e2e")]
fn ephemeral_vault_state() -> VaultState {
    use rand::RngCore;

    let mut master_key = vec![0_u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut master_key);
    VaultState {
        master_key: Zeroizing::new(master_key),
        keys: HashMap::new(),
    }
}

pub fn init_for_runtime() -> Result<(), String> {
    #[cfg(not(feature = "e2e"))]
    return init();

    #[cfg(feature = "e2e")]
    {
        let mut state = STATE
            .lock()
            .map_err(|_| "coffre indisponible".to_string())?;
        *state = Some(ephemeral_vault_state());
        Ok(())
    }
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
    let route = api_route_for_provider(provider_id);
    let scope = route.map(|_| generate_credential_scope()).transpose()?;
    transaction(|candidate| stage_api_key(candidate, provider_id, Some(key), scope.as_ref()))?;
    sync_registry_cache();
    Ok(())
}

pub fn delete_key(provider_id: &str) -> Result<(), String> {
    validate::validate_provider(provider_id)?;
    transaction(|candidate| stage_api_key(candidate, provider_id, None, None))?;
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
#[path = "api_keys_http_tests.rs"]
mod http_tests;

#[cfg(test)]
#[path = "api_keys_mcp_tests.rs"]
mod mcp_tests;

#[cfg(test)]
#[path = "api_keys_transaction_tests.rs"]
mod transaction_tests;

#[cfg(test)]
#[path = "api_keys_retired_tests.rs"]
mod retired_tests;

#[cfg(test)]
#[path = "api_keys_credential_scope_tests.rs"]
mod credential_scope_tests;

#[cfg(all(test, feature = "e2e"))]
#[path = "api_keys_e2e_tests.rs"]
mod e2e_tests;

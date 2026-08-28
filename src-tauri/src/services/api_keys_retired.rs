const RETIRED_PROVIDER_ID: &str = "groq";
const RETIRED_SCOPE_KEY: &str = "raw:reasoning_scope:groq";
const MAX_VAULT_BACKUP_BYTES: u64 = 5 * 1024 * 1024;

fn purge_retired_provider_entries(map: &mut HashMap<String, String>) -> bool {
    let removed_key = map.remove(RETIRED_PROVIDER_ID).is_some();
    let removed_scope = map.remove(RETIRED_SCOPE_KEY).is_some();
    removed_key || removed_scope
}

fn backup_vault_for_retired_cleanup(
    vault_path: &std::path::Path,
    backup_path: &std::path::Path,
) -> Result<(), String> {
    match crate::services::private_store::read_bounded_regular(
        backup_path,
        MAX_VAULT_BACKUP_BYTES,
    )? {
        crate::services::private_store::BoundedFile::Content(_) => return Ok(()),
        crate::services::private_store::BoundedFile::Missing => {}
    }
    match crate::services::private_store::read_bounded_regular(
        vault_path,
        MAX_VAULT_BACKUP_BYTES,
    )? {
        crate::services::private_store::BoundedFile::Missing => Ok(()),
        crate::services::private_store::BoundedFile::Content(bytes) => {
            crate::services::private_store::atomic_write(backup_path, &bytes)
        }
    }
}

fn remove_retired_cleanup_backup(backup_path: &std::path::Path) -> Result<(), String> {
    match std::fs::remove_file(backup_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("nettoyage du coffre indisponible".to_string()),
    }
}

#[cfg(not(feature = "e2e"))]
fn prepare_retired_provider_cleanup(
    master_key: &[u8],
    map: &mut HashMap<String, String>,
) -> Result<bool, String> {
    let vault_path = vault::vault_path();
    let marker_path = vault_path.with_file_name(".retired-provider-groq-v1");
    let backup_path = vault_path.with_file_name("secrets.enc.pre-groq-removal.bak");
    if marker_path.exists() {
        return Ok(backup_path.exists());
    }

    if map.contains_key(RETIRED_PROVIDER_ID) || map.contains_key(RETIRED_SCOPE_KEY) {
        backup_vault_for_retired_cleanup(&vault_path, &backup_path)?;
        let mut candidate = ZeroizingMap(map.clone());
        purge_retired_provider_entries(&mut candidate.0);
        validate_vault_candidate(&candidate.0)?;
        vault::write_vault(master_key, &candidate.0)?;
        std::mem::swap(map, &mut candidate.0);
    }

    delete_retired_provider_keychain_entry();
    crate::services::private_store::atomic_write(&marker_path, b"ok")?;
    Ok(false)
}

#[cfg(not(feature = "e2e"))]
fn delete_retired_provider_keychain_entry() {
    let Ok(entry) = keyring::Entry::new("cl-go-dash", RETIRED_PROVIDER_ID) else {
        log::warn!("retired_provider_keychain_cleanup_unavailable");
        return;
    };
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(_) => log::warn!("retired_provider_keychain_cleanup_failed"),
    }
}

#[cfg(not(feature = "e2e"))]
fn finish_retired_provider_cleanup(remove_previous_backup: bool) -> Result<(), String> {
    if !remove_previous_backup {
        return Ok(());
    }
    let backup_path = vault::vault_path().with_file_name("secrets.enc.pre-groq-removal.bak");
    remove_retired_cleanup_backup(&backup_path)
}

use std::io::ErrorKind;
use std::path::Path;

const MARKER_FILE: &str = ".security-hardening-v1";

pub fn run() -> Result<(), String> {
    run_in(&crate::services::paths::data_dir())
}

fn run_in(root: &Path) -> Result<(), String> {
    crate::services::security_cleanup_sessions::remove_orphan_backups(
        &root.join("agent-sessions"),
    )?;
    let marker = root.join(MARKER_FILE);
    match std::fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => return Ok(()),
        Ok(_) => return Err("nettoyage de sécurité impossible".to_string()),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => return Err("nettoyage de sécurité impossible".to_string()),
    }

    remove_legacy_file(&root.join("secrets.enc.bak-corrupted"))?;
    remove_legacy_file(&root.join("oauth-providers/moonshot/credentials/kimi-code.json"))?;
    remove_legacy_file(&root.join("oauth-providers/xai/auth.json"))?;
    crate::services::security_cleanup_sessions::sanitize_documents(&root.join("agent-sessions"))?;
    crate::services::private_store::atomic_write(&marker, b"ok")
}

fn remove_legacy_file(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("nettoyage de sécurité impossible".to_string()),
    };
    let kind = metadata.file_type();
    if !kind.is_file() && !kind.is_symlink() {
        return Err("nettoyage de sécurité impossible".to_string());
    }
    std::fs::remove_file(path).map_err(|_| "nettoyage de sécurité impossible".to_string())
}

#[cfg(test)]
#[path = "security_cleanup_tests.rs"]
mod tests;

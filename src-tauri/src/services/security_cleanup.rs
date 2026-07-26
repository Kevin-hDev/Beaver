use std::io::ErrorKind;
use std::path::Path;
use zeroize::Zeroizing;

const MARKER_FILE: &str = ".security-hardening-v1";
const MAX_SESSION_FILES: usize = 4_096;
const MAX_SESSION_FILE_BYTES: u64 = 64 * 1024 * 1024;

pub fn run() -> Result<(), String> {
    run_in(&crate::services::paths::data_dir())
}

fn run_in(root: &Path) -> Result<(), String> {
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
    sanitize_session_files(&root.join("agent-sessions"))?;
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

fn sanitize_session_files(directory: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("nettoyage de sécurité impossible".to_string()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("nettoyage de sécurité impossible".to_string());
    }

    let entries =
        std::fs::read_dir(directory).map_err(|_| "nettoyage de sécurité impossible".to_string())?;
    for (index, entry) in entries.enumerate() {
        if index >= MAX_SESSION_FILES {
            return Err("nettoyage de sécurité impossible".to_string());
        }
        let entry = entry.map_err(|_| "nettoyage de sécurité impossible".to_string())?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|_| "nettoyage de sécurité impossible".to_string())?;
        if file_type.is_symlink() {
            return Err("nettoyage de sécurité impossible".to_string());
        }
        if !file_type.is_file() {
            continue;
        }
        sanitize_session_file(&path)?;
    }
    Ok(())
}

fn sanitize_session_file(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "nettoyage de sécurité impossible".to_string())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_SESSION_FILE_BYTES
    {
        return Err("nettoyage de sécurité impossible".to_string());
    }

    let bytes = Zeroizing::new(
        std::fs::read(path).map_err(|_| "nettoyage de sécurité impossible".to_string())?,
    );
    let mut value: serde_json::Value = serde_json::from_slice(bytes.as_slice())
        .map_err(|_| "nettoyage de sécurité impossible".to_string())?;
    crate::services::agent_local::session_security::sanitize_session_value(&mut value);
    let sanitized = serde_json::to_vec_pretty(&value)
        .map_err(|_| "nettoyage de sécurité impossible".to_string())?;
    crate::services::private_store::atomic_write(path, &sanitized)
}

#[cfg(test)]
#[path = "security_cleanup_tests.rs"]
mod tests;

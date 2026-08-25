use std::io::ErrorKind;
use std::path::Path;

use zeroize::Zeroizing;

use crate::services::agent_local::session_limits::{MAX_SESSION_FILES, MAX_SESSION_FILE_BYTES};

const BACKUP_SUFFIX: &str = ".json.v1.bak";

pub(super) fn remove_orphan_backups(directory: &Path) -> Result<(), String> {
    let Some(entries) = read_entries(directory)? else {
        return Ok(());
    };
    for (index, entry) in entries.enumerate() {
        if index >= MAX_SESSION_FILES {
            return Err(failed());
        }
        let entry = entry.map_err(|_| failed())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(failed)?;
        if !name.ends_with(".v1.bak") {
            continue;
        }
        let id = name.strip_suffix(BACKUP_SUFFIX).ok_or_else(failed)?;
        crate::services::agent_local::session_store::validate_session_id(id)
            .map_err(|_| failed())?;
        let file_type = entry.file_type().map_err(|_| failed())?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(failed());
        }
        let main = directory.join(format!("{id}.json"));
        match std::fs::symlink_metadata(&main) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {}
            Ok(_) => return Err(failed()),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                std::fs::remove_file(entry.path()).map_err(|_| failed())?;
            }
            Err(_) => return Err(failed()),
        }
    }
    Ok(())
}

pub(super) fn sanitize_documents(directory: &Path) -> Result<(), String> {
    let Some(entries) = read_entries(directory)? else {
        return Ok(());
    };
    for (index, entry) in entries.enumerate() {
        if index >= MAX_SESSION_FILES {
            return Err(failed());
        }
        let entry = entry.map_err(|_| failed())?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let file_type = entry.file_type().map_err(|_| failed())?;
        if file_type.is_symlink() {
            return Err(failed());
        }
        if !file_type.is_file() {
            continue;
        }
        sanitize_document(&path)?;
    }
    Ok(())
}

fn sanitize_document(path: &Path) -> Result<(), String> {
    let bytes =
        match crate::services::private_store::read_bounded_regular(path, MAX_SESSION_FILE_BYTES)
            .map_err(|_| failed())?
        {
            crate::services::private_store::BoundedFile::Content(bytes) => Zeroizing::new(bytes),
            crate::services::private_store::BoundedFile::Missing => return Err(failed()),
        };
    let mut value: serde_json::Value =
        serde_json::from_slice(bytes.as_slice()).map_err(|_| failed())?;
    crate::services::agent_local::session_security::sanitize_session_value(&mut value);
    let sanitized = serde_json::to_vec_pretty(&value).map_err(|_| failed())?;
    if sanitized.len() as u64 > MAX_SESSION_FILE_BYTES {
        return Err(failed());
    }
    crate::services::private_store::atomic_write(path, &sanitized).map_err(|_| failed())
}

fn read_entries(directory: &Path) -> Result<Option<std::fs::ReadDir>, String> {
    match std::fs::symlink_metadata(directory) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => return Err(failed()),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(failed()),
    }
    std::fs::read_dir(directory).map(Some).map_err(|_| failed())
}

fn failed() -> String {
    "nettoyage de sécurité impossible".to_string()
}

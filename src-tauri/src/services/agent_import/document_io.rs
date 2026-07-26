use super::limits::{MAX_BACKUPS_PER_DOCUMENT, MAX_INSTRUCTION_BYTES};
use sha2::{Digest, Sha256};
use std::path::Path;
use uuid::Uuid;

pub fn read_instruction(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path).map_err(|_| "Document indisponible")?;
    if !metadata.is_file() || metadata.len() > MAX_INSTRUCTION_BYTES {
        return Err("Document incompatible".into());
    }
    let bytes = std::fs::read(path).map_err(|_| "Document indisponible")?;
    std::str::from_utf8(&bytes).map_err(|_| "Document incompatible")?;
    Ok(bytes)
}

pub fn same_file_content(path: &Path, expected: &[u8]) -> Result<bool, String> {
    let metadata = std::fs::metadata(path).map_err(|_| "Document indisponible")?;
    if !metadata.is_file() || metadata.len() > MAX_INSTRUCTION_BYTES {
        return Ok(false);
    }
    let current = std::fs::read(path).map_err(|_| "Document indisponible")?;
    let left = Sha256::digest(&current);
    let right = Sha256::digest(expected);
    Ok(constant_time_hash_eq(&left, &right))
}

pub fn file_hash_matches(path: &Path, expected_hex: &str) -> bool {
    let Ok(expected) = hex::decode(expected_hex) else {
        return false;
    };
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() || metadata.len() > MAX_INSTRUCTION_BYTES {
        return false;
    }
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let current = Sha256::digest(bytes);
    constant_time_hash_eq(&current, &expected)
}

pub fn backup_document(data_dir: &Path, source: &Path) -> Result<(), String> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Document incompatible".to_string())?;
    let backup_dir = data_dir.join("agent-import-backups");
    let backup = backup_dir.join(format!("{name}.{}.bak", Uuid::new_v4()));
    let metadata = std::fs::metadata(source).map_err(|_| "Document indisponible")?;
    if !metadata.is_file() || metadata.len() > MAX_INSTRUCTION_BYTES {
        return Err("Document incompatible".into());
    }
    let bytes = std::fs::read(source).map_err(|_| "Document indisponible")?;
    crate::services::private_store::atomic_write(&backup, &bytes)?;
    prune_backups(&backup_dir, name)
}

fn constant_time_hash_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

fn prune_backups(directory: &Path, name: &str) -> Result<(), String> {
    let prefix = format!("{name}.");
    let mut files = std::fs::read_dir(directory)
        .map_err(|_| "Sauvegarde indisponible")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.starts_with(&prefix) && value.ends_with(".bak"))
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|value| value.modified())
            .ok()
    });
    let remove_count = files.len().saturating_sub(MAX_BACKUPS_PER_DOCUMENT);
    for path in files.into_iter().take(remove_count) {
        std::fs::remove_file(path).map_err(|_| "Sauvegarde indisponible")?;
    }
    Ok(())
}

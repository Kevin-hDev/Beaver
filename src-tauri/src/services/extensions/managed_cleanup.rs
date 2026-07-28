use super::types::{ExtensionOriginKind, ExtensionRecord, MAX_USER_EXTENSIONS};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const MAX_VERSIONS_PER_EXTENSION: usize = 32;
const STAGING_PREFIX: &str = ".staging-";

pub fn unreferenced(records: &[ExtensionRecord]) -> Result<(), String> {
    let root = super::managed_store::root();
    if !root.exists() {
        return Ok(());
    }
    let root = root
        .canonicalize()
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    let referenced = referenced_installs(records);
    let entries = bounded_entries(&root, MAX_USER_EXTENSIONS + 32)?;
    for entry in entries {
        let metadata = std::fs::symlink_metadata(&entry)
            .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
        let name = entry
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if metadata.file_type().is_dir() && valid_staging(name) {
            std::fs::remove_dir_all(entry)
                .map_err(|_| "Nettoyage des extensions impossible.".to_string())?;
        } else if metadata.file_type().is_dir() && super::validation::identifier(name).is_ok() {
            cleanup_versions(&entry, &referenced)?;
        }
    }
    Ok(())
}

fn referenced_installs(records: &[ExtensionRecord]) -> HashSet<PathBuf> {
    records
        .iter()
        .filter(|record| {
            record.origin.as_ref().is_some_and(|origin| {
                matches!(
                    origin.kind,
                    ExtensionOriginKind::Git | ExtensionOriginKind::Npm
                )
            })
        })
        .take(MAX_USER_EXTENSIONS)
        .filter_map(|record| super::managed_store::install_root(record).ok())
        .collect()
}

fn cleanup_versions(directory: &Path, referenced: &HashSet<PathBuf>) -> Result<(), String> {
    let versions = bounded_entries(directory, MAX_VERSIONS_PER_EXTENSION)?;
    for version in versions {
        let metadata = std::fs::symlink_metadata(&version)
            .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
        let name = version
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if metadata.file_type().is_dir() && valid_token(name) {
            let canonical = version
                .canonicalize()
                .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
            if !referenced.contains(&canonical) {
                std::fs::remove_dir_all(&canonical)
                    .map_err(|_| "Nettoyage des extensions impossible.".to_string())?;
            }
        }
    }
    if std::fs::read_dir(directory)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?
        .next()
        .is_none()
    {
        std::fs::remove_dir(directory)
            .map_err(|_| "Nettoyage des extensions impossible.".to_string())?;
    }
    Ok(())
}

fn bounded_entries(directory: &Path, maximum: usize) -> Result<Vec<PathBuf>, String> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(directory)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?
    {
        if entries.len() >= maximum {
            return Err("Trop de dossiers d'extensions gérées.".to_string());
        }
        entries.push(
            entry
                .map_err(|_| "Stockage des extensions indisponible.".to_string())?
                .path(),
        );
    }
    Ok(entries)
}

fn valid_staging(value: &str) -> bool {
    value.strip_prefix(STAGING_PREFIX).is_some_and(valid_token)
}

fn valid_token(value: &str) -> bool {
    value.len() == 32
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

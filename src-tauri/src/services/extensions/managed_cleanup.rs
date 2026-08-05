use super::types::{ExtensionOriginKind, ExtensionRecord, MAX_USER_EXTENSIONS};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const MAX_VERSIONS_PER_EXTENSION: usize = 32;
const MAX_ROOT_ENTRIES: usize = MAX_USER_EXTENSIONS + 32;
const STAGING_PREFIX: &str = ".staging-";

pub fn unreferenced(records: &[ExtensionRecord]) -> Result<(), String> {
    let root = super::managed_store::root();
    if !root.exists() {
        return Ok(());
    }
    let root = dunce::canonicalize(root)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    unreferenced_at(
        &root,
        &referenced_installs(records),
        MAX_ROOT_ENTRIES,
        MAX_VERSIONS_PER_EXTENSION,
    )
}

fn unreferenced_at(
    root: &Path,
    referenced: &HashSet<PathBuf>,
    root_limit: usize,
    version_limit: usize,
) -> Result<(), String> {
    let mut overflow = false;
    for (processed, entry) in std::fs::read_dir(root)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?
        .enumerate()
    {
        if processed >= root_limit {
            overflow = true;
            break;
        }
        let entry = entry
            .map_err(|_| "Stockage des extensions indisponible.".to_string())?
            .path();
        cleanup_root_entry(&entry, referenced, version_limit)?;
    }
    if overflow {
        Err("Trop de dossiers d'extensions gérées.".to_string())
    } else {
        Ok(())
    }
}

fn cleanup_root_entry(
    entry: &Path,
    referenced: &HashSet<PathBuf>,
    version_limit: usize,
) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(entry)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    let name = entry
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if metadata.file_type().is_dir() && valid_staging(name) {
        std::fs::remove_dir_all(entry)
            .map_err(|_| "Nettoyage des extensions impossible.".to_string())
    } else if metadata.file_type().is_dir() && super::validation::identifier(name).is_ok() {
        cleanup_versions(entry, referenced, version_limit)
    } else {
        Ok(())
    }
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

fn cleanup_versions(
    directory: &Path,
    referenced: &HashSet<PathBuf>,
    limit: usize,
) -> Result<(), String> {
    let mut overflow = false;
    for (processed, entry) in std::fs::read_dir(directory)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?
        .enumerate()
    {
        if processed >= limit {
            overflow = true;
            break;
        }
        cleanup_version(
            &entry
                .map_err(|_| "Stockage des extensions indisponible.".to_string())?
                .path(),
            referenced,
        )?;
    }
    if overflow {
        return Err("Trop de versions d'extensions gérées.".to_string());
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

fn cleanup_version(version: &Path, referenced: &HashSet<PathBuf>) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(version)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    let name = version
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if !metadata.file_type().is_dir() || !valid_token(name) {
        return Ok(());
    }
    let canonical = dunce::canonicalize(version)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    if !referenced.contains(&canonical) {
        std::fs::remove_dir_all(&canonical)
            .map_err(|_| "Nettoyage des extensions impossible.".to_string())?;
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_still_removes_a_bounded_batch() {
        let temporary = tempfile::tempdir().unwrap();
        for index in 0..4 {
            let token = format!("{index:032x}");
            std::fs::create_dir(temporary.path().join(format!(".staging-{token}"))).unwrap();
        }
        let result = unreferenced_at(temporary.path(), &HashSet::new(), 2, 2);
        assert!(result.is_err());
        assert_eq!(std::fs::read_dir(temporary.path()).unwrap().count(), 2);

        assert!(unreferenced_at(temporary.path(), &HashSet::new(), 2, 2).is_ok());
        assert_eq!(std::fs::read_dir(temporary.path()).unwrap().count(), 0);
    }
}

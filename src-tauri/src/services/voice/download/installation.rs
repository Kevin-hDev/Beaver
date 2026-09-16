use std::{fs, path::Path};

use uuid::Uuid;

#[cfg(unix)]
use crate::services::private_store::sync_directory;
use crate::services::private_store::{ensure_private_dir, rename_durable};

use super::{
    catalog_validation::revision_directory_name,
    cleanup_partial,
    extraction::{extract_verified, manifest_paths},
    models_root,
    receipt::{load_receipt, receipt_matches, save_receipt, InstallationReceipt},
    VoiceCatalogEntry,
};

pub(crate) fn install_archive(
    entry: &VoiceCatalogEntry,
    archive: &Path,
    data_dir: &Path,
) -> Result<InstallationReceipt, String> {
    if let Some(receipt) = tolerant_receipt(data_dir, &entry.id) {
        if receipt_matches(&receipt, entry) && receipt.install_dir(data_dir).is_dir() {
            cleanup_partial(data_dir, &entry.id)?;
            return Ok(receipt);
        }
    }
    let root = models_root(data_dir);
    let model_parent = root.join("models").join(&entry.id);
    let final_dir =
        model_parent.join(revision_directory_name(&entry.revision).ok_or_else(storage_error)?);
    let staging_parent = root.join("staging");
    ensure_private_dir(&model_parent)?;
    ensure_private_dir(&staging_parent)?;
    if final_dir.exists() {
        super::removal::remove_receipt_file(data_dir, &entry.id)?;
        fs::remove_dir_all(&final_dir).map_err(|_| storage_error())?;
    }
    let staging = staging_parent.join(format!("{}-{}", entry.id, Uuid::new_v4()));
    let result = publish(entry, archive, &staging, &final_dir, data_dir);
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

pub(crate) fn installed_receipt(
    entry: &VoiceCatalogEntry,
    data_dir: &Path,
) -> Result<Option<InstallationReceipt>, String> {
    Ok(tolerant_receipt(data_dir, &entry.id).filter(|receipt| {
        receipt_matches(receipt, entry) && receipt.install_dir(data_dir).is_dir()
    }))
}

fn tolerant_receipt(data_dir: &Path, id: &str) -> Option<InstallationReceipt> {
    match load_receipt(data_dir, id) {
        Ok(receipt) => receipt,
        Err(error) => {
            ::log::warn!(
                "[voice-download] model={} step=receipt-ignored code={}",
                id,
                error
            );
            None
        }
    }
}

fn publish(
    entry: &VoiceCatalogEntry,
    archive: &Path,
    staging: &Path,
    final_dir: &Path,
    data_dir: &Path,
) -> Result<InstallationReceipt, String> {
    extract_verified(entry, archive, staging)?;
    for path in manifest_paths(staging, &entry.files) {
        fs::File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(|_| storage_error())?;
    }
    #[cfg(unix)]
    sync_tree_dirs(staging)?;
    rename_durable(staging, final_dir).map_err(|_| storage_error())?;
    let receipt = InstallationReceipt::from_entry(entry);
    save_receipt(data_dir, &receipt)?;
    ::log::info!(
        "[voice-download] model={} step=published revision={}",
        entry.id,
        entry.revision
    );
    if let Err(error) = cleanup_partial(data_dir, &entry.id) {
        ::log::warn!(
            "[voice-download] model={} step=partial-cleanup-pending code={}",
            entry.id,
            error
        );
    }
    Ok(receipt)
}

#[cfg(unix)]
fn sync_tree_dirs(root: &Path) -> Result<(), String> {
    let mut directories = vec![root.to_path_buf()];
    for entry in walkdir(root)? {
        if entry.is_dir() {
            directories.push(entry);
        }
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        sync_directory(&directory)?;
    }
    Ok(())
}

#[cfg(unix)]
fn walkdir(root: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut found = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).map_err(|_| storage_error())? {
            let entry = entry.map_err(|_| storage_error())?;
            if entry.file_type().map_err(|_| storage_error())?.is_dir() {
                if found.len() >= crate::services::voice::limits::MAX_CATALOG_FILES_PER_ENTRY {
                    return Err(storage_error());
                }
                pending.push(entry.path());
                found.push(entry.path());
            }
        }
    }
    Ok(found)
}

fn storage_error() -> String {
    "model-download-storage-failed".into()
}

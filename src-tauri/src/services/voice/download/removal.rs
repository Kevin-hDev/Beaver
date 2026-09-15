use std::{fs, path::Path};

use crate::services::private_store::sync_directory;

use super::{models_root, receipt::receipt_path};

pub trait RemovalGate {
    fn release_for_removal(&self, model_id: &str) -> bool;
}

pub fn remove_installation(
    data_dir: &Path,
    model_id: &str,
    gate: &impl RemovalGate,
) -> Result<(), String> {
    if model_id.is_empty()
        || !model_id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err(storage_error());
    }
    if !gate.release_for_removal(model_id) {
        return Err("model-download-model-busy".into());
    }
    let receipt = receipt_path(data_dir, model_id);
    match fs::remove_file(&receipt) {
        Ok(()) => sync_directory(receipt.parent().ok_or_else(storage_error)?)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(storage_error()),
    }
    remove_model_files(data_dir, model_id)
}

pub fn resume_incomplete_removals(data_dir: &Path) -> Result<(), String> {
    cleanup_staging(data_dir)?;
    let models = models_root(data_dir).join("models");
    let Ok(entries) = fs::read_dir(&models) else {
        return Ok(());
    };
    for entry in entries
        .take(crate::services::voice::limits::MAX_CATALOG_ENTRIES)
        .filter_map(Result::ok)
    {
        let id = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().is_ok_and(|kind| kind.is_dir())
            && !receipt_path(data_dir, &id).is_file()
        {
            remove_model_files(data_dir, &id)?;
        }
    }
    Ok(())
}

fn cleanup_staging(data_dir: &Path) -> Result<(), String> {
    let staging = models_root(data_dir).join("staging");
    let Ok(entries) = fs::read_dir(&staging) else {
        return Ok(());
    };
    for entry in entries
        .take(crate::services::voice::limits::MAX_CATALOG_ENTRIES)
        .filter_map(Result::ok)
    {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            fs::remove_dir_all(entry.path()).map_err(|_| storage_error())?;
        }
    }
    sync_directory(&staging)
}

fn remove_model_files(data_dir: &Path, model_id: &str) -> Result<(), String> {
    let directory = models_root(data_dir).join("models").join(model_id);
    match fs::remove_dir_all(&directory) {
        Ok(()) => sync_directory(directory.parent().ok_or_else(storage_error)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(storage_error()),
    }
}

fn storage_error() -> String {
    "model-download-storage-failed".into()
}

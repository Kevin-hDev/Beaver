mod catalog;
mod catalog_types;
mod catalog_validation;
mod checkpoint;
mod disk_budget;
mod extraction;
mod installation;
mod receipt;
mod removal;
mod transfer;
mod transfer_http;
mod transfer_prepare;

const VOICE_MODELS_DIR: &str = "voice-models";
const PARTIALS_DIR: &str = "partials";

pub use catalog_types::*;

pub fn load_catalog(
    resource_dir: &std::path::Path,
) -> Result<VoiceCatalog, crate::services::voice::errors::VoiceError> {
    catalog::load_catalog(resource_dir)
}

pub fn find_catalog_entry(
    resource_dir: &std::path::Path,
    id: &str,
) -> Result<VoiceCatalogEntry, crate::services::voice::errors::VoiceError> {
    load_catalog(resource_dir)?
        .entries
        .into_iter()
        .find(|entry| entry.id == id)
        .ok_or_else(crate::services::voice::errors::VoiceError::configuration_unavailable)
}

pub(crate) use checkpoint::{discover_checkpoints, save_checkpoint};
pub use checkpoint::{inspect_partial, Checkpoint, HttpValidator, PartialDecision};
pub(crate) use extraction::verify_file;
pub(crate) use installation::{install_archive, installed_receipt};
pub(crate) use receipt::InstallationReceipt;
#[cfg(test)]
pub(crate) use receipt::ReceiptFile;
pub(crate) use removal::resume_incomplete_removals;
pub(crate) use removal::{remove_installation, RemovalGate};
pub(crate) use transfer::download_archive;
pub(crate) use transfer_prepare::prepare_download;

pub(crate) fn models_root(data_dir: &std::path::Path) -> std::path::PathBuf {
    data_dir.join(VOICE_MODELS_DIR)
}

pub(crate) fn partial_path(data_dir: &std::path::Path, id: &str) -> std::path::PathBuf {
    models_root(data_dir)
        .join(PARTIALS_DIR)
        .join(format!("{id}.part"))
}

pub(crate) fn checkpoint_path(data_dir: &std::path::Path, id: &str) -> std::path::PathBuf {
    models_root(data_dir)
        .join(PARTIALS_DIR)
        .join(format!("{id}.checkpoint.json"))
}

pub(crate) fn cleanup_partial(data_dir: &std::path::Path, id: &str) -> Result<(), String> {
    for path in [partial_path(data_dir, id), checkpoint_path(data_dir, id)] {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("model-download-cleanup-failed".into()),
        }
    }
    Ok(())
}

pub(crate) fn cleanup_request(
    data_dir: &std::path::Path,
    requested_model_id: &str,
) -> Result<(), String> {
    let mut entry_ids = discover_checkpoints(data_dir)
        .into_iter()
        .filter(|checkpoint| checkpoint.requested_model_id == requested_model_id)
        .map(|checkpoint| checkpoint.entry_id)
        .collect::<Vec<_>>();
    if !entry_ids.iter().any(|id| id == requested_model_id) {
        entry_ids.push(requested_model_id.to_string());
    }
    for entry_id in entry_ids {
        cleanup_partial(data_dir, &entry_id)?;
    }
    Ok(())
}

pub(crate) fn ensure_disk_available(
    data_dir: &std::path::Path,
    entry: &VoiceCatalogEntry,
) -> Result<(), String> {
    let partial = std::fs::metadata(partial_path(data_dir, &entry.id))
        .ok()
        .map_or(0, |metadata| metadata.len().min(entry.archive.bytes));
    disk_budget::ensure_available(
        data_dir,
        entry.archive.bytes,
        entry.installed_bytes,
        partial,
    )
}

#[cfg(test)]
mod checkpoint_tests;

#[cfg(test)]
mod transfer_tests;

#[cfg(test)]
mod disk_budget_tests;

#[cfg(test)]
mod extraction_tests;

#[cfg(test)]
mod installation_tests;

#[cfg(test)]
mod receipt_tests;

#[cfg(test)]
mod removal_tests;

#[cfg(test)]
mod catalog_tests;

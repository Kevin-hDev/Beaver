use std::path::Path;

use super::{
    checkpoint_path, cleanup_partial, inspect_partial, partial_path, save_checkpoint, Checkpoint,
    PartialDecision, VoiceCatalogEntry,
};

pub(crate) fn prepare_download(
    entry: &VoiceCatalogEntry,
    requested_model_id: &str,
    data_dir: &Path,
) -> Result<(), String> {
    super::ensure_disk_available(data_dir, entry)?;
    prepare_checkpoint(
        entry,
        requested_model_id,
        &checkpoint_path(data_dir, &entry.id),
        &partial_path(data_dir, &entry.id),
    )?;
    Ok(())
}

pub(super) fn prepare_checkpoint(
    entry: &VoiceCatalogEntry,
    requested_model_id: &str,
    checkpoint_file: &Path,
    partial_file: &Path,
) -> Result<Checkpoint, String> {
    let loaded = super::checkpoint::load_checkpoint(checkpoint_file)
        .ok()
        .flatten();
    let actual_len = std::fs::metadata(partial_file).ok().map(|meta| meta.len());
    if let Some(checkpoint) = loaded.filter(|checkpoint| {
        checkpoint.matches_entry(entry) && checkpoint.requested_model_id == requested_model_id
    }) {
        match inspect_partial(&checkpoint, actual_len) {
            PartialDecision::Resume { .. } => return Ok(checkpoint),
            PartialDecision::Truncate { durable_bytes } => {
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(partial_file)
                    .and_then(|file| file.set_len(durable_bytes))
                    .map_err(|_| "model-download-storage-failed".to_string())?;
                return Ok(checkpoint);
            }
            PartialDecision::Discard => {}
        }
    }
    cleanup_partial(entry_data_dir(partial_file)?, &entry.id)?;
    let checkpoint = Checkpoint::new(
        requested_model_id.to_string(),
        entry.id.clone(),
        entry.revision.clone(),
        entry.archive.bytes,
        entry.archive.sha256.clone(),
    )
    .ok_or_else(|| "model-download-invalid-model".to_string())?;
    save_checkpoint(checkpoint_file, &checkpoint)?;
    Ok(checkpoint)
}

fn entry_data_dir(partial_file: &Path) -> Result<&Path, String> {
    partial_file
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| "model-download-storage-failed".to_string())
}

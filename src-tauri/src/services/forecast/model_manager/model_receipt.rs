use super::{
    fs_safety,
    model_artifacts::{self, sha256_matches, ModelArtifacts},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

const RECEIPT_FILE: &str = ".artifacts-v1.json";
const MAX_RECEIPT_BYTES: u64 = 64 * 1024;
const READ_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ModelReceipt {
    schema_version: u32,
    model: ModelArtifacts,
}

pub(super) fn is_current(model_dir: &Path, model_id: &str) -> bool {
    let Ok(directory) = std::fs::symlink_metadata(model_dir) else {
        return false;
    };
    if !directory.is_dir() || directory.file_type().is_symlink() {
        return false;
    }
    let Ok(expected) = expected_receipt(model_id) else {
        return false;
    };
    let path = model_dir.join(RECEIPT_FILE);
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        return false;
    };
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_RECEIPT_BYTES
    {
        return false;
    }
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    serde_json::from_slice::<ModelReceipt>(&bytes).is_ok_and(|receipt| receipt == expected)
        && files_have_expected_sizes(model_dir, &expected.model)
}

pub(super) fn has_expected_sizes(model_dir: &Path, model_id: &str) -> bool {
    model_artifacts::model(model_id).is_ok_and(|model| files_have_expected_sizes(model_dir, model))
}

pub(super) async fn write_current(model_dir: &Path, model_id: &str) -> Result<(), String> {
    let metadata = tokio::fs::symlink_metadata(model_dir)
        .await
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Validation du modèle Forecast impossible".to_string());
    }
    let receipt = expected_receipt(model_id)?;
    let bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
    if bytes.len() as u64 > MAX_RECEIPT_BYTES {
        return Err("Validation du modèle Forecast impossible".to_string());
    }
    write_receipt_atomically(model_dir, &bytes).await
}

async fn write_receipt_atomically(model_dir: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = model_dir.join(".artifacts-v1.tmp");
    let destination = model_dir.join(RECEIPT_FILE);
    fs_safety::remove_path(&temporary)
        .await
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
    let result = async {
        let mut file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .await
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        file.write_all(bytes)
            .await
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        file.flush()
            .await
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        file.sync_all()
            .await
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        drop(file);
        fs_safety::remove_path(&destination)
            .await
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        tokio::fs::rename(&temporary, &destination)
            .await
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())
    }
    .await;
    if result.is_err() {
        let _ = fs_safety::remove_path(&temporary).await;
    }
    result
}

pub(super) async fn verify_and_write(
    model_dir: &Path,
    model_id: &str,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let model = model_artifacts::model(model_id)?;
    let metadata = tokio::fs::symlink_metadata(model_dir)
        .await
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Validation du modèle Forecast impossible".to_string());
    }
    for artifact in &model.artifacts {
        verify_artifact(model_dir, artifact, cancel).await?;
    }
    write_current(model_dir, model_id).await
}

async fn verify_artifact(
    model_dir: &Path,
    artifact: &super::model_artifacts::ModelArtifact,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let path = model_dir.join(&artifact.path);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != artifact.size {
        return Err("Validation du modèle Forecast impossible".to_string());
    }
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
    let mut buffer = vec![0u8; READ_BUFFER_BYTES];
    let mut hasher = Sha256::new();
    loop {
        if cancel.is_cancelled() {
            return Err("cancelled".to_string());
        }
        let read = file
            .read(&mut buffer)
            .await
            .map_err(|_| "Validation du modèle Forecast impossible".to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if !sha256_matches(&hasher.finalize(), &artifact.sha256) {
        return Err("Validation du modèle Forecast impossible".to_string());
    }
    Ok(())
}

fn expected_receipt(model_id: &str) -> Result<ModelReceipt, String> {
    Ok(ModelReceipt {
        schema_version: 1,
        model: model_artifacts::model(model_id)?.clone(),
    })
}

fn files_have_expected_sizes(model_dir: &Path, model: &ModelArtifacts) -> bool {
    model.artifacts.iter().all(|artifact| {
        std::fs::symlink_metadata(model_dir.join(&artifact.path)).is_ok_and(|metadata| {
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.len() == artifact.size
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{is_current, write_current};

    #[tokio::test]
    async fn receipt_alone_never_marks_missing_artifacts_as_current() {
        let root = tempfile::tempdir().unwrap();
        write_current(root.path(), "chronos-bolt-tiny")
            .await
            .unwrap();
        assert!(!is_current(root.path(), "chronos-bolt-tiny"));
    }
}

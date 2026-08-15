#![allow(dead_code)]

use super::error::OllamaErrorCode;
use crate::services::paths::OllamaPaths;

pub(super) fn validate_install_confinement(paths: &OllamaPaths) -> Result<(), OllamaErrorCode> {
    let active_parent = paths
        .active
        .parent()
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
    let staging_parent = paths
        .install_staging
        .parent()
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
    if active_parent != staging_parent {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    for path in [&paths.active, &paths.install_staging] {
        if let Ok(metadata) = std::fs::symlink_metadata(path) {
            if metadata.file_type().is_symlink() {
                return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
            }
        }
    }
    Ok(())
}

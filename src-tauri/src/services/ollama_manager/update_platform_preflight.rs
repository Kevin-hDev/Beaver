use super::super::super::error::OllamaErrorCode;
use super::super::super::path_identity::NativePathIdentityResolver;
use super::super::super::spawn_profile::OllamaSpawnProfile;
use super::super::{UpdateRequest, UpdateSidecar};
use std::path::Path;

pub(super) fn validate_request(request: &UpdateRequest) -> Result<(), OllamaErrorCode> {
    if matches!(request.sidecar, UpdateSidecar::External) {
        return Ok(());
    }
    OllamaSpawnProfile::resolve(
        &request.paths,
        request.inherited_environment.clone(),
        &request.inherited_cwd,
        &NativePathIdentityResolver,
    )?;
    Ok(())
}

pub(super) fn ensure_absent(path: &Path) -> Result<(), OllamaErrorCode> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(OllamaErrorCode::OllamaStorageUnavailable),
    }
}

use super::cleanup_inspection;
use super::constants::PROCESS_REAP_FALLBACK_TIMEOUT;
use super::durable_fs::platform_fs;
use super::error::OllamaErrorCode;
use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};
use super::process_receipt::{ProcessReceiptError, ProcessReceiptRecovery, ProcessReceiptStore};
use super::recovery_decision::DirectoryEvidence;
use super::spawn_profile_paths::active_executable;
use super::types::BundleState;
use crate::services::paths::{bundle_receipt_path, OllamaPaths};
use std::time::Instant;

pub(super) fn complete(paths: &OllamaPaths) -> Result<BundleState, OllamaErrorCode> {
    let fs = platform_fs();
    let receipt = super::bundle_receipt::read_receipt(&fs, &bundle_receipt_path(&paths.active))?;
    let store = ProcessReceiptStore::platform(paths.clone());
    let Some(receipt) = receipt else {
        return match store.read().map_err(map_receipt_error)? {
            None => Ok(BundleState::Absent),
            Some(_) => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        };
    };
    let actual = cleanup_inspection::fingerprint(&fs, &paths.active)
        .ok_or(OllamaErrorCode::OllamaUpdateRecoveryRequired)?;
    if !matches!(actual, DirectoryEvidence::Present(ref value) if value == &receipt.fingerprint) {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    let executable = NativePathIdentityResolver
        .canonical_executable(&active_executable(&paths.active))?
        .execution_identity()
        .ok_or(OllamaErrorCode::OllamaUpdateRecoveryRequired)?;
    let deadline = Instant::now() + PROCESS_REAP_FALLBACK_TIMEOUT;
    match store
        .recover_active(&receipt.fingerprint, executable, deadline)
        .map_err(map_receipt_error)?
    {
        ProcessReceiptRecovery::Missing
        | ProcessReceiptRecovery::StaleRemoved
        | ProcessReceiptRecovery::Reaped => Ok(BundleState::Ready),
        ProcessReceiptRecovery::RecoveryRequired | ProcessReceiptRecovery::Exact(_) => {
            Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
        }
    }
}

fn map_receipt_error(error: ProcessReceiptError) -> OllamaErrorCode {
    match error {
        ProcessReceiptError::Storage => OllamaErrorCode::OllamaStorageUnavailable,
        ProcessReceiptError::Missing
        | ProcessReceiptError::Oversized
        | ProcessReceiptError::Invalid => OllamaErrorCode::OllamaUpdateRecoveryRequired,
    }
}

use super::bundle_evidence;
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

pub(super) fn prepare(paths: &OllamaPaths) -> Result<(), OllamaErrorCode> {
    let fs = platform_fs();
    let receipt = super::bundle_receipt::read_receipt(&fs, &bundle_receipt_path(&paths.active))?;
    let store = ProcessReceiptStore::platform(paths.clone());
    // begin_operation garde l'autorité de démarrage pendant cette purge : aucun spawn
    // possédé ne peut publier un nouveau reçu en parallèle.
    store.remove_safe_tmp().map_err(map_receipt_error)?;
    if store.read().map_err(map_receipt_error)?.is_none() {
        return Ok(());
    }
    let Some(receipt) = receipt else {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    };
    verify_bundle(paths, &fs, &receipt.fingerprint)?;
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
        | ProcessReceiptRecovery::Reaped => Ok(()),
        ProcessReceiptRecovery::RecoveryRequired | ProcessReceiptRecovery::Exact(_) => {
            Err(OllamaErrorCode::OllamaUpdateRecoveryRequired)
        }
    }
}

pub(super) fn bundle_state(paths: &OllamaPaths) -> Result<BundleState, OllamaErrorCode> {
    let fs = platform_fs();
    let receipt = super::bundle_receipt::read_receipt(&fs, &bundle_receipt_path(&paths.active))?;
    let Some(receipt) = receipt else {
        return match std::fs::symlink_metadata(&paths.active) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BundleState::Absent),
            _ => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        };
    };
    verify_bundle(paths, &fs, &receipt.fingerprint)?;
    Ok(BundleState::Ready)
}

fn verify_bundle(
    paths: &OllamaPaths,
    fs: &super::durable_fs::PlatformOllamaDurableFs,
    expected: &super::fingerprint::BundleFingerprint,
) -> Result<(), OllamaErrorCode> {
    let actual = bundle_evidence::fingerprint(fs, &paths.active);
    matches!(actual, DirectoryEvidence::Present(ref value) if value == expected)
        .then_some(())
        .ok_or(OllamaErrorCode::OllamaUpdateRecoveryRequired)
}

fn map_receipt_error(error: ProcessReceiptError) -> OllamaErrorCode {
    match error {
        ProcessReceiptError::Storage => OllamaErrorCode::OllamaStorageUnavailable,
        ProcessReceiptError::Missing => OllamaErrorCode::OllamaUpdateRecoveryRequired,
        ProcessReceiptError::Oversized | ProcessReceiptError::Invalid => {
            ::log::error!("[ollama] process receipt rejected classification={error:?}");
            OllamaErrorCode::OllamaUpdateRecoveryRequired
        }
    }
}

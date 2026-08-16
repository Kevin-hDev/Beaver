use super::blocking::run_ollama_blocking;
use super::bundle_evidence;
use super::bundle_receipt::{read_receipt, write_receipt, BundleReceipt};
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::fingerprint::BundleFingerprint;
use super::recovery_decision::DirectoryEvidence;
use super::spawn_profile_paths::active_executable;
use crate::services::paths::{bundle_receipt_path, OllamaPaths};
use std::sync::Arc;

pub(super) async fn validate_and_sync<F>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
    expected: &BundleFingerprint,
) -> Result<(), OllamaErrorCode>
where
    F: OllamaDurableFs + 'static,
{
    let receipt_path = bundle_receipt_path(&paths.active);
    verify_fingerprint(&**fs, paths, expected)?;
    match read_receipt(&**fs, &receipt_path)? {
        Some(receipt) if receipt.fingerprint == *expected => {}
        Some(_) => return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        None => {
            // Les versions Beaver publiées précèdent le reçu J3 : l'empreinte disque est
            // l'autorité de migration, puis le reçu devient obligatoire dès son écriture.
            write_receipt(&**fs, &paths.active, &BundleReceipt::new(expected.clone()))?;
        }
    }
    let fs_for_sync = Arc::clone(fs);
    let version_path = paths.active.join("VERSION");
    let executable_path = active_executable(&paths.active);
    let receipt_for_sync = receipt_path.clone();
    run_ollama_blocking(move || {
        for path in [&version_path, &executable_path, &receipt_for_sync] {
            fs_for_sync
                .sync_file(path)
                .map_err(|error| super::storage_error::durable("adoption-sync-file", error))?;
            fs_for_sync
                .sync_parent(path)
                .map_err(|error| super::storage_error::durable("adoption-sync-parent", error))?;
        }
        Ok(())
    })
    .await?;
    verify_fingerprint(&**fs, paths, expected)
}

fn verify_fingerprint(
    fs: &dyn OllamaDurableFs,
    paths: &OllamaPaths,
    expected: &BundleFingerprint,
) -> Result<(), OllamaErrorCode> {
    matches!(
        bundle_evidence::fingerprint(fs, &paths.active),
        DirectoryEvidence::Present(ref actual) if actual == expected
    )
    .then_some(())
    .ok_or(OllamaErrorCode::OllamaUpdateRecoveryRequired)
}

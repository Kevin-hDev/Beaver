#![allow(dead_code)]

use super::blocking::run_ollama_blocking;
use super::bundle_receipt::{read_receipt, write_receipt_at, write_version, BundleReceipt};
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion};
use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};
use super::probe::PreparedBundle;
use crate::services::paths::OllamaPaths;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub async fn prepare_bundle(
    paths: &OllamaPaths,
    version: &OllamaVersion,
) -> Result<PreparedBundle, OllamaErrorCode> {
    let root_path = paths.install_staging.clone();
    let version = version.clone();
    run_ollama_blocking(move || {
        let identity = NativePathIdentityResolver;
        let root = identity.canonical_directory(&root_path)?;
        let executable = identity.canonical_executable(&binary_path(root.path()))?;
        let digest = super::probe_http::hash_file(executable.path())
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        Ok(PreparedBundle {
            root,
            executable,
            fingerprint: BundleFingerprint {
                version,
                executable_sha256: digest,
            },
        })
    })
    .await
}

pub async fn write_metadata<F: OllamaDurableFs + 'static>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
    prepared: &PreparedBundle,
) -> Result<(), OllamaErrorCode> {
    let fs_version = Arc::clone(fs);
    let root = paths.install_staging.clone();
    let version = prepared.fingerprint.version.to_string();
    run_ollama_blocking(move || write_version(&*fs_version, &root, &version)).await?;
    let receipt = BundleReceipt::new(prepared.fingerprint.clone());
    let fs_receipt = Arc::clone(fs);
    let path = paths.bundle_receipt.clone();
    let tmp = path.with_extension("tmp");
    run_ollama_blocking(move || write_receipt_at(&*fs_receipt, &path, &tmp, &receipt)).await?;
    let fs_sync = Arc::clone(fs);
    let staging = paths.install_staging.clone();
    run_ollama_blocking(move || {
        fs_sync
            .sync_file(&staging)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}

pub async fn reinspect_active<F: OllamaDurableFs + 'static>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
    expected: &BundleFingerprint,
) -> Result<(), OllamaErrorCode> {
    let active = paths.active.clone();
    let expected = expected.clone();
    let receipt_path = paths.bundle_receipt.clone();
    let fs = Arc::clone(fs);
    run_ollama_blocking(move || {
        let identity = NativePathIdentityResolver;
        let root = identity.canonical_directory(&active)?;
        let executable = identity.canonical_executable(&binary_path(root.path()))?;
        let actual = super::probe_http::hash_file(executable.path())
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        let receipt = read_receipt(&*fs, &receipt_path)?;
        receipt
            .as_ref()
            .is_some_and(|receipt| {
                receipt.fingerprint.version == expected.version
                    && receipt
                        .fingerprint
                        .executable_sha256
                        .constant_time_eq(&actual)
            })
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)
    })
    .await
}

fn binary_path(root: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    };
    let nested = root.join("bin").join(name);
    if nested.exists() {
        nested
    } else {
        root.join(name)
    }
}

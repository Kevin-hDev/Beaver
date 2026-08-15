#![allow(dead_code)]

use super::blocking::run_ollama_blocking;
use super::bundle_install::{prepare_bundle, reinspect_active, write_metadata};
#[cfg(not(test))]
use super::download::download_archives;
use super::download::verify_sha256;
use super::durable_fs::{platform_fs, OllamaDurableFs};
use super::error::OllamaErrorCode;
use super::extract::{extract_archive, extract_archive_overlay};
use super::fingerprint::{BundleFingerprint, OllamaVersion};
pub(crate) use super::install_archives::archive_staging_path;
use super::install_archives::remove_archives;
use super::path_identity::NativePathIdentityResolver;
use super::probe::{OllamaTargetProbe, OwnedOllamaTargetProbe, TargetValidation};
use super::release_source::OllamaReleaseManifest;
use super::spawn_profile::OllamaSpawnProfile;
use crate::services::paths::{ollama_paths, OllamaPaths};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

const INSTALL_PROBE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct InstallRequest {
    pub paths: OllamaPaths,
    pub version: Option<OllamaVersion>,
    pub manifest: Option<OllamaReleaseManifest>,
    pub inherited_environment: Vec<(OsString, OsString)>,
    pub inherited_cwd: PathBuf,
    pub cancellation: CancellationToken,
    pub deadline: Option<Instant>,
    #[cfg(test)]
    pub(crate) local_archives: Option<Vec<PathBuf>>,
}

impl InstallRequest {
    pub fn for_test(root: PathBuf) -> Self {
        Self::for_test_with_cancel(root, CancellationToken::new())
    }

    pub fn for_test_with_cancel(root: PathBuf, cancellation: CancellationToken) -> Self {
        let paths = ollama_paths(&root);
        Self {
            paths,
            version: None,
            manifest: None,
            inherited_environment: vec![
                (OsString::from("HOME"), root.clone().into_os_string()),
                (OsString::from("PATH"), OsString::from("/usr/bin")),
            ],
            inherited_cwd: root,
            cancellation,
            deadline: None,
            #[cfg(test)]
            local_archives: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstallOutcome {
    Preparing,
    Installed { fingerprint: BundleFingerprint },
}

impl InstallOutcome {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Installed { .. })
    }
}

pub async fn install(request: InstallRequest) -> Result<InstallOutcome, OllamaErrorCode> {
    super::install_confinement::validate_install_confinement(&request.paths)?;
    if request.paths.active.exists() {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    if request.cancellation.is_cancelled() {
        return Err(OllamaErrorCode::OllamaOperationCancelled);
    }
    let manifest = request
        .manifest
        .clone()
        .ok_or(OllamaErrorCode::OllamaDownloadFailed)?;
    let version = request
        .version
        .clone()
        .unwrap_or_else(|| manifest.version.clone());
    if version != manifest.version {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    let fs = Arc::new(platform_fs());
    prepare_staging(&fs, &request.paths.install_staging).await?;
    let archive_staging = archive_staging_path(&request.paths);
    prepare_staging(&fs, archive_staging).await?;
    #[cfg(test)]
    let archives = super::install_test_support::archive_paths(
        &request,
        &manifest,
        archive_staging,
        &request.cancellation,
    )
    .await?;
    #[cfg(not(test))]
    let archives = download_archives(&manifest, archive_staging, &request.cancellation).await?;
    if archives.len() != manifest.archives().len() {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    for (index, (archive, path)) in manifest.archives().iter().zip(&archives).enumerate() {
        ensure_not_cancelled(&request.cancellation)?;
        verify_sha256(path, &archive.sha256)?;
        let extract = if index == 0 {
            extract_archive
        } else {
            extract_archive_overlay
        };
        extract(
            path,
            &request.paths.install_staging,
            archive.file_name.as_str(),
            &request.cancellation,
        )?;
    }
    remove_archives(&fs, archive_staging, &archives).await?;
    let prepared = prepare_bundle(&request.paths, &version).await?;
    write_metadata(&fs, &request.paths, &prepared).await?;
    let profile = resolve_install_profile(&request, &request.paths.install_staging)?;
    let deadline = request
        .deadline
        .unwrap_or_else(|| Instant::now() + INSTALL_PROBE_TIMEOUT);
    let probe = OwnedOllamaTargetProbe::with_deadline(deadline);
    match probe
        .validate(&prepared, &profile, &request.cancellation)
        .await
    {
        TargetValidation::Valid { fingerprint } if fingerprint == prepared.fingerprint => {}
        TargetValidation::Valid { .. } | TargetValidation::InvalidTarget { .. } => {
            return Err(OllamaErrorCode::OllamaBundleInvalid)
        }
        TargetValidation::Deferred { code } => return Err(code),
    }
    ensure_not_cancelled(&request.cancellation)?;
    commit_staging(&fs, &request.paths).await?;
    reinspect_active(&fs, &request.paths, &prepared.fingerprint).await?;
    Ok(InstallOutcome::Installed {
        fingerprint: prepared.fingerprint,
    })
}

async fn prepare_staging<F: OllamaDurableFs + 'static>(
    fs: &Arc<F>,
    staging: &Path,
) -> Result<(), OllamaErrorCode> {
    if staging.exists() {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    let fs = Arc::clone(fs);
    let staging = staging.to_path_buf();
    run_ollama_blocking(move || {
        fs.create_directory_durable(&staging)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}

fn resolve_install_profile(
    request: &InstallRequest,
    staging: &Path,
) -> Result<OllamaSpawnProfile, OllamaErrorCode> {
    let mut paths = request.paths.clone();
    paths.active = staging.to_path_buf();
    OllamaSpawnProfile::resolve_probe(
        &paths,
        request.inherited_environment.clone(),
        &request.inherited_cwd,
        &NativePathIdentityResolver,
    )
}

async fn commit_staging<F: OllamaDurableFs + 'static>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
) -> Result<(), OllamaErrorCode> {
    let fs = Arc::clone(fs);
    let source = paths.install_staging.clone();
    let active = paths.active.clone();
    run_ollama_blocking(move || {
        if active.exists() {
            return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
        }
        fs.rename_durable(&source, &active)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}

fn ensure_not_cancelled(cancellation: &CancellationToken) -> Result<(), OllamaErrorCode> {
    (!cancellation.is_cancelled())
        .then_some(())
        .ok_or(OllamaErrorCode::OllamaOperationCancelled)
}

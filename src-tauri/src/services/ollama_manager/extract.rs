#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::release_source::AllowlistedArchiveName;
use std::path::{Component, Path};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveMemberKind {
    Regular,
    Directory,
    Symlink,
    Hardlink,
    Other,
}

impl ArchiveMemberKind {
    pub fn validate(self) -> Result<(), OllamaErrorCode> {
        matches!(self, Self::Regular | Self::Directory)
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaExtractionFailed)
    }
}

pub fn validate_member_path(path: &Path) -> Result<(), OllamaErrorCode> {
    if path.as_os_str().is_empty() || path.to_string_lossy().contains('\\') {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    Ok(())
}

pub fn validate_staging_directory(staging: &Path) -> Result<(), OllamaErrorCode> {
    let metadata = std::fs::symlink_metadata(staging).map_err(|error| {
        super::storage_error::io(
            "extract-staging-inspect",
            &error,
            OllamaErrorCode::OllamaStorageUnavailable,
        )
    })?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    Ok(())
}

pub fn validate_empty_staging(staging: &Path) -> Result<(), OllamaErrorCode> {
    validate_staging_directory(staging)?;
    if std::fs::read_dir(staging)
        .map_err(|error| {
            super::storage_error::io(
                "extract-staging-enumerate",
                &error,
                OllamaErrorCode::OllamaStorageUnavailable,
            )
        })?
        .next()
        .is_some()
    {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    Ok(())
}

pub fn extract_archive(
    archive: &Path,
    staging: &Path,
    archive_name: &str,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    extract_archive_with_hook(
        archive,
        staging,
        archive_name,
        cancellation,
        true,
        &mut || Ok::<(), OllamaErrorCode>(()),
    )
}

#[cfg(test)]
pub(super) fn extract_archive_for_test<F>(
    archive: &Path,
    staging: &Path,
    archive_name: &str,
    cancellation: &CancellationToken,
    mut before_write: F,
) -> Result<(), OllamaErrorCode>
where
    F: FnMut() -> Result<(), OllamaErrorCode>,
{
    extract_archive_with_hook(
        archive,
        staging,
        archive_name,
        cancellation,
        true,
        &mut before_write,
    )
}

fn extract_archive_with_hook(
    archive: &Path,
    staging: &Path,
    archive_name: &str,
    cancellation: &CancellationToken,
    require_empty: bool,
    before_write: &mut dyn FnMut() -> Result<(), OllamaErrorCode>,
) -> Result<(), OllamaErrorCode> {
    extract_archive_contents(
        archive,
        staging,
        archive_name,
        cancellation,
        require_empty,
        before_write,
    )
}

pub fn extract_archive_overlay(
    archive: &Path,
    staging: &Path,
    archive_name: &str,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    extract_archive_contents(
        archive,
        staging,
        archive_name,
        cancellation,
        false,
        &mut || Ok::<(), OllamaErrorCode>(()),
    )
}

fn extract_archive_contents(
    archive: &Path,
    staging: &Path,
    archive_name: &str,
    cancellation: &CancellationToken,
    require_empty: bool,
    before_write: &mut dyn FnMut() -> Result<(), OllamaErrorCode>,
) -> Result<(), OllamaErrorCode> {
    let archive_name = AllowlistedArchiveName::parse(archive_name)?;
    ensure_not_cancelled(cancellation)?;
    let root = super::extract_root::ExtractionRoot::open(staging, require_empty)?;
    match archive_name.as_str() {
        name if name.ends_with(".zip") => {
            super::extract_archive::extract_zip(archive, &root, cancellation, before_write)
        }
        name if name.ends_with(".tgz") || name.ends_with(".tar.gz") => {
            let file =
                std::fs::File::open(archive).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
            let decoder = flate2::read::GzDecoder::new(file);
            super::extract_archive::extract_tar(
                tar::Archive::new(decoder),
                &root,
                cancellation,
                before_write,
            )
        }
        name if name.ends_with(".tar.zst") => {
            let file =
                std::fs::File::open(archive).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
            let decoder = zstd::stream::read::Decoder::new(file)
                .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
            super::extract_archive::extract_tar(
                tar::Archive::new(decoder),
                &root,
                cancellation,
                before_write,
            )
        }
        _ => Err(OllamaErrorCode::OllamaBundleInvalid),
    }
}

pub(super) fn ensure_not_cancelled(
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    (!cancellation.is_cancelled())
        .then_some(())
        .ok_or(OllamaErrorCode::OllamaOperationCancelled)
}

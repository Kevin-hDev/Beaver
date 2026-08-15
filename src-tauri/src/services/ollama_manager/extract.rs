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
    let metadata = std::fs::symlink_metadata(staging)
        .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    Ok(())
}

pub fn validate_empty_staging(staging: &Path) -> Result<(), OllamaErrorCode> {
    validate_staging_directory(staging)?;
    if std::fs::read_dir(staging)
        .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?
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
    validate_empty_staging(staging)?;
    extract_archive_contents(archive, staging, archive_name, cancellation)
}

pub fn extract_archive_overlay(
    archive: &Path,
    staging: &Path,
    archive_name: &str,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    validate_staging_directory(staging)?;
    extract_archive_contents(archive, staging, archive_name, cancellation)
}

fn extract_archive_contents(
    archive: &Path,
    staging: &Path,
    archive_name: &str,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let archive_name = AllowlistedArchiveName::parse(archive_name)?;
    ensure_not_cancelled(cancellation)?;
    match archive_name.as_str() {
        name if name.ends_with(".zip") => {
            super::extract_archive::extract_zip(archive, staging, cancellation)
        }
        name if name.ends_with(".tgz") || name.ends_with(".tar.gz") => {
            let file =
                std::fs::File::open(archive).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
            let decoder = flate2::read::GzDecoder::new(file);
            super::extract_archive::extract_tar(tar::Archive::new(decoder), staging, cancellation)
        }
        name if name.ends_with(".tar.zst") => {
            let file =
                std::fs::File::open(archive).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
            let decoder = zstd::stream::read::Decoder::new(file)
                .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
            super::extract_archive::extract_tar(tar::Archive::new(decoder), staging, cancellation)
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

#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::extract::{ensure_not_cancelled, validate_member_path, ArchiveMemberKind};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use tokio_util::sync::CancellationToken;

const MAX_ENTRIES: usize = 50_000;
const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const COPY_BUFFER_BYTES: usize = 64 * 1024;

pub(super) fn extract_tar<R: Read>(
    mut archive: tar::Archive<R>,
    staging: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let root =
        std::fs::canonicalize(staging).map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    let mut names = HashSet::new();
    let mut total = 0_u64;
    for (index, item) in archive
        .entries()
        .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?
        .enumerate()
    {
        ensure_not_cancelled(cancellation)?;
        if index >= MAX_ENTRIES {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let mut entry = item.map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        let name = entry
            .path()
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?
            .into_owned();
        validate_member_path(&name)?;
        if !names.insert(name.clone()) {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let kind = match entry.header().entry_type() {
            kind if kind.is_dir() => ArchiveMemberKind::Directory,
            kind if kind.is_file() => ArchiveMemberKind::Regular,
            kind if kind.is_symlink() => ArchiveMemberKind::Symlink,
            kind if kind.is_hard_link() => ArchiveMemberKind::Hardlink,
            _ => ArchiveMemberKind::Other,
        };
        kind.validate()?;
        let target = safe_target(&root, &name)?;
        if kind == ArchiveMemberKind::Directory {
            std::fs::create_dir_all(&target)
                .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
            continue;
        }
        total = total
            .checked_add(
                entry
                    .header()
                    .size()
                    .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?,
            )
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        if total > MAX_UNPACKED_BYTES {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let mode = entry
            .header()
            .mode()
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        write_entry(&mut entry, &target, mode, cancellation)?;
    }
    Ok(())
}

pub(super) fn extract_zip(
    archive: &Path,
    staging: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    super::extract_zip_validate::validate_zip_directory(archive)?;
    let file = std::fs::File::open(archive).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
    if archive.len() > MAX_ENTRIES {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    let root =
        std::fs::canonicalize(staging).map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    let mut names = HashSet::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        ensure_not_cancelled(cancellation)?;
        let mut entry = archive
            .by_index(index)
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        let raw_name = Path::new(entry.name());
        validate_member_path(raw_name)?;
        let name = entry
            .enclosed_name()
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?
            .to_path_buf();
        validate_member_path(&name)?;
        if !names.insert(name.clone()) {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let symlink = entry
            .unix_mode()
            .map(|mode| mode & 0o170000 == 0o120000)
            .unwrap_or(false);
        ArchiveMemberKind::from_zip(entry.is_dir(), symlink).validate()?;
        let target = safe_target(&root, &name)?;
        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
            continue;
        }
        total = total
            .checked_add(entry.size())
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        if total > MAX_UNPACKED_BYTES {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let parent = target
            .parent()
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        std::fs::create_dir_all(parent).map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
        let mut output = std::fs::File::create(&target)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
        copy_bounded(&mut entry, &mut output, cancellation)?;
        apply_mode(&target, entry.unix_mode());
    }
    Ok(())
}

impl ArchiveMemberKind {
    fn from_zip(directory: bool, symlink: bool) -> Self {
        if directory {
            Self::Directory
        } else if symlink {
            Self::Symlink
        } else {
            Self::Regular
        }
    }
}

fn safe_target(root: &Path, name: &Path) -> Result<PathBuf, OllamaErrorCode> {
    let target = root.join(name);
    let mut current = root.to_path_buf();
    for component in name.components() {
        let Component::Normal(part) = component else {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        };
        current.push(part);
        if let Ok(metadata) = std::fs::symlink_metadata(&current) {
            if metadata.file_type().is_symlink() {
                return Err(OllamaErrorCode::OllamaBundleInvalid);
            }
            let resolved = std::fs::canonicalize(&current)
                .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
            if !resolved.starts_with(root) {
                return Err(OllamaErrorCode::OllamaBundleInvalid);
            }
        }
    }
    Ok(target)
}

fn write_entry<R: Read>(
    entry: &mut tar::Entry<R>,
    target: &Path,
    mode: u32,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let parent = target
        .parent()
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
    std::fs::create_dir_all(parent).map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    let mut output =
        std::fs::File::create(target).map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    copy_bounded(entry, &mut output, cancellation)?;
    apply_mode(target, Some(mode));
    Ok(())
}

fn apply_mode(path: &Path, mode: Option<u32>) {
    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o777));
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
}

fn copy_bounded<R: Read, W: Write>(
    input: &mut R,
    output: &mut W,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    loop {
        ensure_not_cancelled(cancellation)?;
        let read = input
            .read(&mut buffer)
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        if read == 0 {
            return Ok(());
        }
        output
            .write_all(&buffer[..read])
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
    }
}

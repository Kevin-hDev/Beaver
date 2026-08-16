#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::extract::{ensure_not_cancelled, validate_member_path, ArchiveMemberKind};
use super::extract_root::ExtractionRoot;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::Path;
use tokio_util::sync::CancellationToken;

const MAX_ENTRIES: usize = 50_000;
const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const COPY_BUFFER_BYTES: usize = 64 * 1024;

pub(super) fn extract_tar<R: Read>(
    mut archive: tar::Archive<R>,
    root: &ExtractionRoot,
    cancellation: &CancellationToken,
    before_write: &mut dyn FnMut() -> Result<(), OllamaErrorCode>,
) -> Result<(), OllamaErrorCode> {
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
        before_write()?;
        if kind == ArchiveMemberKind::Directory {
            root.create_directory_all(&name)?;
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
        write_entry(&mut entry, root, &name, mode, cancellation)?;
    }
    Ok(())
}

pub(super) fn extract_zip(
    archive: &Path,
    root: &ExtractionRoot,
    cancellation: &CancellationToken,
    before_write: &mut dyn FnMut() -> Result<(), OllamaErrorCode>,
) -> Result<(), OllamaErrorCode> {
    super::extract_zip_validate::validate_zip_directory(archive)?;
    let file = std::fs::File::open(archive).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
    if archive.len() > MAX_ENTRIES {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
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
        if !names.insert(name.clone()) {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let symlink = entry
            .unix_mode()
            .map(|mode| mode & 0o170000 == 0o120000)
            .unwrap_or(false);
        ArchiveMemberKind::from_zip(entry.is_dir(), symlink).validate()?;
        before_write()?;
        if entry.is_dir() {
            root.create_directory_all(&name)?;
            continue;
        }
        total = total
            .checked_add(entry.size())
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        if total > MAX_UNPACKED_BYTES {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let mut output = root.create_file(&name, entry.unix_mode().unwrap_or(0o644))?;
        copy_bounded(&mut entry, &mut output, cancellation)?;
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

fn write_entry<R: Read>(
    entry: &mut tar::Entry<R>,
    root: &ExtractionRoot,
    name: &Path,
    mode: u32,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let mut output = root.create_file(name, mode)?;
    copy_bounded(entry, &mut output, cancellation)?;
    Ok(())
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
        output.write_all(&buffer[..read]).map_err(|error| {
            super::storage_error::io(
                "archive-entry-write",
                &error,
                OllamaErrorCode::OllamaStorageUnavailable,
            )
        })?;
    }
}

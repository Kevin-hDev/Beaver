use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::{VoiceCatalogEntry, VoiceEngine, VoiceModelFile};

const MAX_ARCHIVE_ENTRIES: usize = 128;
const ARCHIVE_OVERHEAD_BYTES: u64 = 32 * 1024 * 1024;

pub(super) fn extract_verified(
    entry: &VoiceCatalogEntry,
    archive: &Path,
    destination: &Path,
) -> Result<(), String> {
    verify_file(archive, entry.archive.bytes, &entry.archive.sha256)?;
    fs::create_dir_all(destination).map_err(|_| storage_error())?;
    if entry.engine == VoiceEngine::SileroVad {
        return extract_raw(entry, archive, destination);
    }
    let file = fs::File::open(archive).map_err(|_| storage_error())?;
    let decoder = bzip2::read::BzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut found = HashSet::with_capacity(entry.files.len());
    let mut total = 0_u64;
    for (index, item) in archive.entries().map_err(|_| archive_error())?.enumerate() {
        if index >= MAX_ARCHIVE_ENTRIES {
            return Err(archive_error());
        }
        let mut item = item.map_err(|_| archive_error())?;
        let path = item.path().map_err(|_| archive_error())?.into_owned();
        validate_archive_path(&path)?;
        if item.header().entry_type().is_dir() {
            continue;
        }
        if !item.header().entry_type().is_file() {
            return Err(archive_error());
        }
        let size = item.header().size().map_err(|_| archive_error())?;
        total = total.checked_add(size).ok_or_else(archive_error)?;
        if total > entry.installed_bytes.saturating_add(ARCHIVE_OVERHEAD_BYTES) {
            return Err(archive_error());
        }
        let Some(expected) = matching_file(&path, &entry.files) else {
            continue;
        };
        if size != expected.bytes || !found.insert(expected.path.clone()) {
            return Err(archive_error());
        }
        write_entry(&mut item, destination, expected)?;
    }
    if found.len() != entry.files.len() {
        return Err(archive_error());
    }
    verify_manifest(destination, &entry.files)
}

pub(super) fn verify_file(path: &Path, bytes: u64, sha256: &str) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|_| archive_error())?;
    let metadata = file.metadata().map_err(|_| archive_error())?;
    if !metadata.is_file() || metadata.len() != bytes {
        return Err(archive_error());
    }
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|_| archive_error())?;
    (hex::encode(hasher.finalize()) == sha256)
        .then_some(())
        .ok_or_else(archive_error)
}

fn extract_raw(
    entry: &VoiceCatalogEntry,
    archive: &Path,
    destination: &Path,
) -> Result<(), String> {
    if entry.files.len() != 1 || entry.files[0].bytes != entry.archive.bytes {
        return Err(archive_error());
    }
    let target = destination.join(&entry.files[0].path);
    create_parent(&target)?;
    fs::copy(archive, target).map_err(|_| storage_error())?;
    verify_manifest(destination, &entry.files)
}

fn write_entry(
    source: &mut impl Read,
    destination: &Path,
    expected: &VoiceModelFile,
) -> Result<(), String> {
    let target = destination.join(&expected.path);
    create_parent(&target)?;
    let mut output = fs::File::create(target).map_err(|_| storage_error())?;
    let copied = std::io::copy(&mut source.take(expected.bytes + 1), &mut output)
        .map_err(|_| storage_error())?;
    if copied != expected.bytes {
        return Err(archive_error());
    }
    output.flush().map_err(|_| storage_error())
}

fn verify_manifest(root: &Path, files: &[VoiceModelFile]) -> Result<(), String> {
    for file in files {
        verify_file(&root.join(&file.path), file.bytes, &file.sha256)?;
    }
    Ok(())
}

fn matching_file<'a>(path: &Path, files: &'a [VoiceModelFile]) -> Option<&'a VoiceModelFile> {
    files.iter().find(|file| path.ends_with(&file.path))
}

fn validate_archive_path(path: &Path) -> Result<(), String> {
    let valid = path
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir));
    valid.then_some(()).ok_or_else(archive_error)
}

fn create_parent(path: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or_else(storage_error)?;
    fs::create_dir_all(parent).map_err(|_| storage_error())
}

pub(super) fn manifest_paths(root: &Path, files: &[VoiceModelFile]) -> Vec<PathBuf> {
    files.iter().map(|file| root.join(&file.path)).collect()
}

fn archive_error() -> String {
    "model-download-archive-invalid".into()
}

fn storage_error() -> String {
    "model-download-storage-failed".into()
}

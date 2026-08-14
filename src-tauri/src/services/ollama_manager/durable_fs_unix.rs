#![allow(dead_code)]

use super::{io_error_kind, OllamaDurableFs, OllamaFsError, OllamaFsErrorKind};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UnixOllamaDurableFs;

impl OllamaDurableFs for UnixOllamaDurableFs {
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, OllamaFsError> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        if !metadata.is_file() || metadata.len() > max_bytes as u64 {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?
            .take(max_bytes.saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        if bytes.len() > max_bytes {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        Ok(bytes)
    }

    fn create_directory_durable(&self, path: &Path) -> Result<(), OllamaFsError> {
        fs::create_dir_all(path).map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        sync_directory(path)?;
        sync_parent_path(path)
    }

    fn write_new_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        write_atomic(tmp, final_path, bytes, false)
    }

    fn replace_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        write_atomic(tmp, final_path, bytes, true)
    }

    fn rename_durable(&self, source: &Path, destination: &Path) -> Result<(), OllamaFsError> {
        fs::rename(source, destination)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        sync_parent_path(source)?;
        if source.parent() != destination.parent() {
            sync_parent_path(destination)?;
        }
        Ok(())
    }

    fn remove_file_durable(&self, path: &Path) -> Result<(), OllamaFsError> {
        fs::remove_file(path).map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        sync_parent_path(path)
    }

    fn remove_tree(&self, root: &Path) -> Result<(), OllamaFsError> {
        fs::remove_dir_all(root).map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        sync_parent_path(root)
    }

    fn sync_file(&self, path: &Path) -> Result<(), OllamaFsError> {
        File::open(path)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?
            .sync_all()
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))
    }

    fn sync_parent(&self, path: &Path) -> Result<(), OllamaFsError> {
        sync_parent_path(path)
    }
}

fn write_atomic(
    tmp: &Path,
    final_path: &Path,
    bytes: &[u8],
    replace: bool,
) -> Result<(), OllamaFsError> {
    if !replace && fs::symlink_metadata(final_path).is_ok() {
        return Err(OllamaFsError::new(OllamaFsErrorKind::AlreadyExists));
    }
    let mut temp_created = false;
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(tmp)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        temp_created = true;
        file.write_all(bytes)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        file.sync_all()
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        if !replace && fs::symlink_metadata(final_path).is_ok() {
            return Err(OllamaFsError::new(OllamaFsErrorKind::AlreadyExists));
        }
        fs::rename(tmp, final_path).map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        temp_created = false;
        sync_parent_path(final_path)
    })();
    if result.is_err() && temp_created {
        let _ = fs::remove_file(tmp);
    }
    result
}

fn sync_directory(path: &Path) -> Result<(), OllamaFsError> {
    File::open(path)
        .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?
        .sync_all()
        .map_err(|error| OllamaFsError::new(io_error_kind(&error)))
}

fn sync_parent_path(path: &Path) -> Result<(), OllamaFsError> {
    let parent = path
        .parent()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    sync_directory(parent)
}

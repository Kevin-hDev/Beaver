#![allow(dead_code)]

use super::super::path_identity::CanonicalDirectory;
use super::sync_parent_pair;
use super::{OllamaDurableFs, OllamaFsError, OllamaFsErrorKind};
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[path = "durable_fs_unix_verified.rs"]
mod verified;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UnixOllamaDurableFs;

impl OllamaDurableFs for UnixOllamaDurableFs {
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, OllamaFsError> {
        let metadata =
            fs::symlink_metadata(path).map_err(|error| OllamaFsError::from_io(&error))?;
        if !metadata.is_file() || metadata.len() > max_bytes as u64 {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|error| OllamaFsError::from_io(&error))?
            .take(max_bytes.saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| OllamaFsError::from_io(&error))?;
        if bytes.len() > max_bytes {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        Ok(bytes)
    }

    fn create_directory_durable(&self, path: &Path) -> Result<(), OllamaFsError> {
        fs::create_dir_all(path).map_err(|error| OllamaFsError::from_io(&error))?;
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
        fs::rename(source, destination).map_err(|error| OllamaFsError::from_io(&error))?;
        sync_parent_pair(source, destination, sync_directory)
    }

    fn remove_file_durable(&self, path: &Path) -> Result<(), OllamaFsError> {
        fs::remove_file(path).map_err(|error| OllamaFsError::from_io(&error))?;
        sync_parent_path(path)
    }

    fn remove_tree(&self, root: &Path) -> Result<(), OllamaFsError> {
        fs::remove_dir_all(root).map_err(|error| OllamaFsError::from_io(&error))?;
        sync_parent_path(root)
    }

    fn remove_tree_verified(&self, root: &CanonicalDirectory) -> Result<(), OllamaFsError> {
        verified::remove_tree(root)
    }

    fn sync_file(&self, path: &Path) -> Result<(), OllamaFsError> {
        File::open(path)
            .map_err(|error| OllamaFsError::from_io(&error))?
            .sync_all()
            .map_err(|error| OllamaFsError::from_io(&error))
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
    write_atomic_with_hook(tmp, final_path, bytes, replace, || {})
}

#[cfg(test)]
impl UnixOllamaDurableFs {
    pub(crate) fn write_new_atomic_with_hook<F>(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
        before_publish: F,
    ) -> Result<(), OllamaFsError>
    where
        F: FnOnce(),
    {
        write_atomic_with_hook(tmp, final_path, bytes, false, before_publish)
    }
}

fn write_atomic_with_hook<F>(
    tmp: &Path,
    final_path: &Path,
    bytes: &[u8],
    replace: bool,
    before_publish: F,
) -> Result<(), OllamaFsError>
where
    F: FnOnce(),
{
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(tmp)
            .map_err(|error| OllamaFsError::from_io(&error))?;
        file.write_all(bytes)
            .map_err(|error| OllamaFsError::from_io(&error))?;
        file.sync_all()
            .map_err(|error| OllamaFsError::from_io(&error))?;
        before_publish();
        if replace {
            fs::rename(tmp, final_path).map_err(|error| OllamaFsError::from_io(&error))?;
        } else {
            rename_new_no_replace(tmp, final_path)?;
        }
        sync_parent_pair(tmp, final_path, sync_directory)
    })();
    // Le tmp reste volontairement présent tant que la publication n'a pas consommé son nom.
    result
}

fn rename_new_no_replace(source: &Path, destination: &Path) -> Result<(), OllamaFsError> {
    #[cfg(target_os = "macos")]
    {
        let source = c_path(source)?;
        let destination = c_path(destination)?;
        let result = unsafe {
            libc::renameatx_np(
                libc::AT_FDCWD,
                source.as_ptr(),
                libc::AT_FDCWD,
                destination.as_ptr(),
                libc::RENAME_EXCL,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(OllamaFsError::from_io(&std::io::Error::last_os_error()))
        }
    }
    #[cfg(target_os = "linux")]
    {
        let source = c_path(source)?;
        let destination = c_path(destination)?;
        let result = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                source.as_ptr(),
                libc::AT_FDCWD,
                destination.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(OllamaFsError::from_io(&std::io::Error::last_os_error()))
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        fs::hard_link(source, destination).map_err(|error| OllamaFsError::from_io(&error))?;
        fs::remove_file(source).map_err(|error| OllamaFsError::from_io(&error))
    }
}

fn c_path(path: &Path) -> Result<CString, OllamaFsError> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))
}

fn sync_directory(path: &Path) -> Result<(), OllamaFsError> {
    File::open(path)
        .map_err(|error| OllamaFsError::from_io(&error))?
        .sync_all()
        .map_err(|error| OllamaFsError::from_io(&error))
}

fn sync_parent_path(path: &Path) -> Result<(), OllamaFsError> {
    let parent = path
        .parent()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    sync_directory(parent)
}

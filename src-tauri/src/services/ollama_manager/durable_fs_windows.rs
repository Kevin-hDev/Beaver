#![allow(dead_code)]

use super::{
    io_error_kind, retry_windows_sharing, sync_parent_pair, validate_wide_units,
    windows_file_flush_access, OllamaDurableFs, OllamaFsError, OllamaFsErrorKind,
};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS, ERROR_FILE_NOT_FOUND,
    ERROR_INVALID_PARAMETER, ERROR_LOCK_VIOLATION, ERROR_PATH_NOT_FOUND, ERROR_SHARING_VIOLATION,
    GENERIC_WRITE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FlushFileBuffers, MoveFileExW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    OPEN_EXISTING,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct WindowsOllamaDurableFs {
    cancelled: Arc<AtomicBool>,
}

impl WindowsOllamaDurableFs {
    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

impl OllamaDurableFs for WindowsOllamaDurableFs {
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
        write_atomic(tmp, final_path, bytes, false, &self.cancelled)
    }

    fn replace_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        write_atomic(tmp, final_path, bytes, true, &self.cancelled)
    }

    fn rename_durable(&self, source: &Path, destination: &Path) -> Result<(), OllamaFsError> {
        move_file(source, destination, true, &self.cancelled)?;
        sync_parent_pair(source, destination, sync_directory)
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
        flush_path(path, 0)
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
    cancelled: &AtomicBool,
) -> Result<(), OllamaFsError> {
    (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(tmp)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        file.write_all(bytes)
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        file.sync_all()
            .map_err(|error| OllamaFsError::new(io_error_kind(&error)))?;
        move_file(tmp, final_path, replace, cancelled)?;
        sync_parent_pair(tmp, final_path, sync_directory)
    })()
}

fn move_file(
    source: &Path,
    destination: &Path,
    replace: bool,
    cancelled: &AtomicBool,
) -> Result<(), OllamaFsError> {
    let source = wide(source)?;
    let destination = wide(destination)?;
    let flags = MOVEFILE_WRITE_THROUGH
        | if replace {
            MOVEFILE_REPLACE_EXISTING
        } else {
            0
        };
    retry_windows_sharing(
        || unsafe {
            if MoveFileExW(source.as_ptr(), destination.as_ptr(), flags) != 0 {
                Ok(())
            } else {
                Err(win_error(GetLastError()))
            }
        },
        || cancelled.load(Ordering::Acquire),
        thread::sleep,
    )
}

fn sync_directory(path: &Path) -> Result<(), OllamaFsError> {
    flush_path(path, FILE_FLAG_BACKUP_SEMANTICS)
}

fn flush_path(path: &Path, flags: u32) -> Result<(), OllamaFsError> {
    let wide_path = wide(path)?;
    debug_assert_eq!(windows_file_flush_access(), GENERIC_WRITE);
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            flags,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(win_error(unsafe { GetLastError() }));
    }
    let result = if unsafe { FlushFileBuffers(handle) } != 0 {
        Ok(())
    } else {
        Err(win_error(unsafe { GetLastError() }))
    };
    unsafe { CloseHandle(handle) };
    result
}

fn sync_parent_path(path: &Path) -> Result<(), OllamaFsError> {
    let parent = path
        .parent()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    sync_directory(parent)
}

fn win_error(code: u32) -> OllamaFsError {
    let kind = match code {
        ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => OllamaFsErrorKind::NotFound,
        ERROR_ALREADY_EXISTS | ERROR_FILE_EXISTS => OllamaFsErrorKind::AlreadyExists,
        ERROR_SHARING_VIOLATION | ERROR_LOCK_VIOLATION => OllamaFsErrorKind::SharingViolation,
        ERROR_INVALID_PARAMETER => OllamaFsErrorKind::InvalidInput,
        _ => OllamaFsErrorKind::Other,
    };
    OllamaFsError::new(kind)
}

fn wide(path: &Path) -> Result<Vec<u16>, OllamaFsError> {
    validate_wide_units(path.as_os_str().encode_wide()).map_err(OllamaFsError::new)?;
    let units = path.as_os_str().encode_wide().count();
    let mut result = Vec::with_capacity(units + 1);
    result.extend(path.as_os_str().encode_wide());
    result.push(0);
    Ok(result)
}

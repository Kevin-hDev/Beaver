use super::super::super::canonical_executable::{
    CanonicalExecutable, NativeFileIdentity, StableFileHandle,
};
use super::super::{
    CanonicalDirectory, NativeDirectoryIdentity, OllamaError, StableDirectoryHandle,
};
use std::fs::OpenOptions;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::sync::Arc;
use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, DELETE, FILE_ATTRIBUTE_DIRECTORY,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_DELETE_CHILD, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_READ, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES,
    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
};

fn open_handle(
    path: &Path,
    flags: u32,
) -> Result<(std::fs::File, BY_HANDLE_FILE_INFORMATION), OllamaError> {
    let access = if flags & FILE_FLAG_BACKUP_SEMANTICS != 0 {
        DELETE | FILE_DELETE_CHILD | FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES
    } else {
        FILE_GENERIC_READ
    };
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .access_mode(access)
        .custom_flags(flags)
        .open(path)
        .map_err(|_| super::super::super::error::OllamaErrorCode::OllamaStorageUnavailable)?;
    let raw = file.as_raw_handle();
    let non_inheritable =
        unsafe { SetHandleInformation(raw as HANDLE, HANDLE_FLAG_INHERIT, 0) } != 0;
    if !non_inheritable {
        return Err(super::super::super::error::OllamaErrorCode::OllamaStorageUnavailable);
    }
    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    let success = unsafe { GetFileInformationByHandle(raw as _, info.as_mut_ptr()) } != 0;
    if !success {
        return Err(super::super::super::error::OllamaErrorCode::OllamaStorageUnavailable);
    }
    Ok((file, unsafe { info.assume_init() }))
}

fn native_identity(info: &BY_HANDLE_FILE_INFORMATION) -> NativeDirectoryIdentity {
    let file_id = ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64;
    NativeDirectoryIdentity::windows(info.dwVolumeSerialNumber as u64, file_id)
}

pub(super) fn opened(path: &Path) -> Result<CanonicalDirectory, OllamaError> {
    let (file, info) = open_handle(
        path,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
    )?;
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(super::super::super::error::OllamaErrorCode::OllamaModelStoreConflict);
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
        return Err(super::super::super::error::OllamaErrorCode::OllamaStorageUnavailable);
    }
    let canonical = dunce::canonicalize(path)
        .map_err(|_| super::super::super::error::OllamaErrorCode::OllamaStorageUnavailable)?;
    Ok(CanonicalDirectory::from_native(
        canonical,
        Some(native_identity(&info)),
        Some(StableDirectoryHandle(Arc::new(file))),
    ))
}

pub(super) fn canonical_executable(path: &Path) -> Result<CanonicalExecutable, OllamaError> {
    let (file, info) = open_handle(path, FILE_FLAG_OPEN_REPARSE_POINT)?;
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(super::super::super::error::OllamaErrorCode::OllamaModelStoreConflict);
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
        return Err(super::super::super::error::OllamaErrorCode::OllamaModelStoreConflict);
    }
    let file_id = ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64;
    Ok(CanonicalExecutable::from_native(
        path.to_path_buf(),
        NativeFileIdentity::windows(info.dwVolumeSerialNumber as u64, file_id),
        StableFileHandle(Arc::new(file)),
    ))
}

pub(super) fn ancestor_identity(path: &Path) -> Result<NativeDirectoryIdentity, OllamaError> {
    let (file, info) = open_handle(
        path,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
    )?;
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(super::super::super::error::OllamaErrorCode::OllamaModelStoreConflict);
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
        return Err(super::super::super::error::OllamaErrorCode::OllamaStorageUnavailable);
    }
    drop(file);
    Ok(native_identity(&info))
}

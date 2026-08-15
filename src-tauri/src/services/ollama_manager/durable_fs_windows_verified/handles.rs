use super::super::{win_error, OllamaFsError};
use std::path::Path;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, ERROR_NOT_SUPPORTED,
    HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FileDispositionInfo, FileDispositionInfoEx, GetFileInformationByHandle,
    SetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_DISPOSITION_FLAG_DELETE,
    FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE, FILE_DISPOSITION_FLAG_POSIX_SEMANTICS,
    FILE_DISPOSITION_INFO, FILE_DISPOSITION_INFO_EX, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::Storage::FileSystem::{DELETE, FILE_DELETE_CHILD};

pub(super) type FileInfo = BY_HANDLE_FILE_INFORMATION;

pub(super) struct OwnedHandle(HANDLE);

impl OwnedHandle {
    pub(super) fn new(handle: HANDLE) -> Result<Self, OllamaFsError> {
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            Err(win_error(unsafe { GetLastError() }))
        } else {
            Ok(Self(handle))
        }
    }

    pub(super) fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub(super) fn open_path(path: &Path, directory: bool) -> Result<OwnedHandle, OllamaFsError> {
    let wide_path = super::super::wide(path)?;
    let access = if directory {
        DELETE | FILE_LIST_DIRECTORY | FILE_DELETE_CHILD | FILE_READ_ATTRIBUTES
    } else {
        DELETE | FILE_READ_ATTRIBUTES
    };
    let flags = FILE_FLAG_OPEN_REPARSE_POINT
        | if directory {
            FILE_FLAG_BACKUP_SEMANTICS
        } else {
            0
        };
    OwnedHandle::new(unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            flags,
            std::ptr::null_mut(),
        )
    })
}

pub(super) fn file_info(handle: HANDLE) -> Result<FileInfo, OllamaFsError> {
    let mut info = std::mem::MaybeUninit::<FileInfo>::zeroed();
    if unsafe { GetFileInformationByHandle(handle, info.as_mut_ptr()) } == 0 {
        return Err(win_error(unsafe { GetLastError() }));
    }
    Ok(unsafe { info.assume_init() })
}

pub(super) fn file_id(info: &FileInfo) -> u64 {
    ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64
}

pub(super) fn same_identity(left: &FileInfo, right: &FileInfo) -> bool {
    left.dwVolumeSerialNumber == right.dwVolumeSerialNumber && file_id(left) == file_id(right)
}

pub(super) fn matches_identity(info: &FileInfo, expected_id: u64, expected_volume: u32) -> bool {
    info.dwVolumeSerialNumber == expected_volume && file_id(info) == expected_id
}

pub(super) fn mark_deleted(handle: HANDLE) -> Result<(), OllamaFsError> {
    let disposition = FILE_DISPOSITION_INFO_EX {
        Flags: FILE_DISPOSITION_FLAG_DELETE
            | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS
            | FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE,
    };
    if unsafe {
        SetFileInformationByHandle(
            handle,
            FileDispositionInfoEx,
            (&disposition as *const FILE_DISPOSITION_INFO_EX).cast(),
            std::mem::size_of::<FILE_DISPOSITION_INFO_EX>() as u32,
        )
    } != 0
    {
        return Ok(());
    }
    let extended_error = unsafe { GetLastError() };
    if !matches!(
        extended_error,
        ERROR_ACCESS_DENIED | ERROR_INVALID_PARAMETER | ERROR_NOT_SUPPORTED
    ) {
        return Err(win_error(extended_error));
    }
    let legacy = FILE_DISPOSITION_INFO { DeleteFile: true };
    if unsafe {
        SetFileInformationByHandle(
            handle,
            FileDispositionInfo,
            (&legacy as *const FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    } == 0
    {
        return Err(win_error(unsafe { GetLastError() }));
    }
    Ok(())
}

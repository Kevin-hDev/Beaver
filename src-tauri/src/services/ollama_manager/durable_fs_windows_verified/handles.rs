use super::super::{win_error, OllamaFsError, OllamaFsErrorKind};
use std::path::{Component, Path};
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, GENERIC_READ, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FileDispositionInfoEx, FileIdType, GetFileInformationByHandle, OpenFileById,
    SetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_DISPOSITION_FLAG_DELETE,
    FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE, FILE_DISPOSITION_INFO_EX,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_DESCRIPTOR,
    FILE_ID_DESCRIPTOR_0, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE,
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

pub(super) fn open_child(
    volume: HANDLE,
    file_id: i64,
    directory: bool,
) -> Result<OwnedHandle, OllamaFsError> {
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
    let descriptor = FILE_ID_DESCRIPTOR {
        dwSize: std::mem::size_of::<FILE_ID_DESCRIPTOR>() as u32,
        Type: FileIdType,
        Anonymous: FILE_ID_DESCRIPTOR_0 { FileId: file_id },
    };
    OwnedHandle::new(unsafe {
        OpenFileById(
            volume,
            &descriptor,
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            flags,
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

pub(super) fn mark_deleted(handle: HANDLE) -> Result<(), OllamaFsError> {
    let disposition = FILE_DISPOSITION_INFO_EX {
        Flags: FILE_DISPOSITION_FLAG_DELETE | FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE,
    };
    if unsafe {
        SetFileInformationByHandle(
            handle,
            FileDispositionInfoEx,
            (&disposition as *const FILE_DISPOSITION_INFO_EX).cast(),
            std::mem::size_of::<FILE_DISPOSITION_INFO_EX>() as u32,
        )
    } == 0
    {
        return Err(win_error(unsafe { GetLastError() }));
    }
    Ok(())
}

pub(super) fn open_volume(path: &Path) -> Result<OwnedHandle, OllamaFsError> {
    let prefix = match path.components().next() {
        Some(Component::Prefix(prefix)) => prefix.as_os_str().to_string_lossy(),
        _ => return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput)),
    };
    if prefix.len() != 2 || !prefix.as_bytes()[1].eq(&b':') {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let volume = format!(r"\\.\{}", prefix);
    let volume = super::super::wide(Path::new(&volume))?;
    OwnedHandle::new(unsafe {
        CreateFileW(
            volume.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    })
}

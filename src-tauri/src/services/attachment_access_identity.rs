#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct FileIdentity(u128);

#[cfg(unix)]
pub(super) fn from_file(file: &std::fs::File) -> Option<FileIdentity> {
    use std::os::unix::fs::MetadataExt;

    let metadata = file.metadata().ok()?;
    Some(FileIdentity(
        (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino()),
    ))
}

#[cfg(windows)]
pub(super) fn from_file(file: &std::fs::File) -> Option<FileIdentity> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };

    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, info.as_mut_ptr()) } == 0 {
        return None;
    }
    let info = unsafe { info.assume_init() };
    let file_index = (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow);
    (info.dwVolumeSerialNumber != 0 && file_index != 0).then_some(FileIdentity(
        (u128::from(info.dwVolumeSerialNumber) << 64) | u128::from(file_index),
    ))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn from_file(_: &std::fs::File) -> Option<FileIdentity> {
    None
}

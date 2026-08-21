use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use super::BoundedFile;

pub(super) fn read(path: &Path, max_bytes: u64) -> Result<BoundedFile, ()> {
    let mut options = OpenOptions::new();
    options.read(true);
    configure_no_follow(&mut options);
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BoundedFile::Missing);
        }
        Err(_) => return Err(()),
    };
    if !file_is_single_link_regular(&file) {
        return Err(());
    }
    let metadata = file.metadata().map_err(|_| ())?;
    if metadata.len() > max_bytes {
        return Err(());
    }
    let read_limit = max_bytes.checked_add(1).ok_or(())?;
    let mut content = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(&mut file)
        .take(read_limit)
        .read_to_end(&mut content)
        .map_err(|_| ())?;
    if content.len() as u64 > max_bytes {
        return Err(());
    }
    Ok(BoundedFile::Content(content))
}

#[cfg(unix)]
fn configure_no_follow(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
}

#[cfg(windows)]
fn configure_no_follow(options: &mut OpenOptions) {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
}

#[cfg(unix)]
pub(super) fn file_is_single_link_regular(file: &std::fs::File) -> bool {
    use std::os::unix::fs::MetadataExt;

    file.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.nlink() == 1)
}

#[cfg(windows)]
pub(super) fn file_is_single_link_regular(file: &std::fs::File) -> bool {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_REPARSE_POINT,
    };

    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, info.as_mut_ptr()) } == 0 {
        return false;
    }
    let info = unsafe { info.assume_init() };
    info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) == 0
        && info.nNumberOfLinks == 1
}

#[cfg(not(any(unix, windows)))]
fn configure_no_follow(_: &mut OpenOptions) {}

#[cfg(not(any(unix, windows)))]
pub(super) fn file_is_single_link_regular(_: &std::fs::File) -> bool {
    false
}

use super::super::super::process::OllamaProcessError;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

const GATE_LINK_PREFIX: &str = ".beaver-gated-";
const OWNER_FILE: &str = ".owner";
const MAX_STALE_GATE_LINKS: usize = 32;

pub(super) fn stale_gate_links(
    parent_file: &File,
    parent: &Path,
) -> Result<(), OllamaProcessError> {
    let mut stale = Vec::new();
    for entry in std::fs::read_dir(parent).map_err(|_| OllamaProcessError::Identity)? {
        let entry = entry.map_err(|_| OllamaProcessError::Identity)?;
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with(GATE_LINK_PREFIX)
        {
            continue;
        }
        if stale.len() == MAX_STALE_GATE_LINKS {
            return Err(OllamaProcessError::Identity);
        }
        stale.push(entry.file_name());
    }
    for name in stale {
        if let Some(owner) = link_owner(&name) {
            let Some(link) = open_at_file(parent_file, &name) else {
                continue;
            };
            if !link.metadata().is_ok_and(|metadata| metadata.is_file()) || process_is_alive(owner)
            {
                continue;
            }
            let name = CString::new(name.as_bytes()).map_err(|_| OllamaProcessError::Identity)?;
            let removed = unsafe { libc::unlinkat(parent_file.as_raw_fd(), name.as_ptr(), 0) };
            if removed != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOENT)
            {
                return Err(OllamaProcessError::Identity);
            }
            continue;
        }
        let Some(directory) = open_at_directory(parent_file, &name) else {
            continue;
        };
        let Some(owner) = read_owner(&directory) else {
            continue;
        };
        if process_is_alive(owner) {
            continue;
        }
        clear_gate_directory(&directory)?;
        let name = CString::new(name.as_bytes()).map_err(|_| OllamaProcessError::Identity)?;
        let removed =
            unsafe { libc::unlinkat(parent_file.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) };
        if removed != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOENT) {
            return Err(OllamaProcessError::Identity);
        }
    }
    Ok(())
}

fn link_owner(name: &std::ffi::OsStr) -> Option<u32> {
    let value = name.to_str()?.strip_prefix(GATE_LINK_PREFIX)?;
    let (owner, suffix) = value.split_once('-')?;
    (!suffix.is_empty()).then_some(())?;
    owner.parse().ok()
}

fn open_at_file(parent: &File, name: &std::ffi::OsStr) -> Option<File> {
    let name = CString::new(name.as_bytes()).ok()?;
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0,
        )
    };
    (fd >= 0).then(|| unsafe { File::from_raw_fd(fd) })
}

fn clear_gate_directory(directory: &File) -> Result<(), OllamaProcessError> {
    for name in directory_entries(directory)? {
        let name = CString::new(name.as_bytes()).map_err(|_| OllamaProcessError::Identity)?;
        let removed = unsafe { libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), 0) };
        if removed != 0 {
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::EISDIR) {
                return Err(OllamaProcessError::Identity);
            }
            if unsafe { libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) }
                != 0
            {
                return Err(OllamaProcessError::Identity);
            }
        }
    }
    Ok(())
}

fn directory_entries(directory: &File) -> Result<Vec<std::ffi::OsString>, OllamaProcessError> {
    let duplicate = unsafe { libc::dup(directory.as_raw_fd()) };
    if duplicate < 0 {
        return Err(OllamaProcessError::Identity);
    }
    let stream = unsafe { libc::fdopendir(duplicate) };
    if stream.is_null() {
        unsafe { libc::close(duplicate) };
        return Err(OllamaProcessError::Identity);
    }
    let mut entries = Vec::new();
    loop {
        let entry = unsafe { libc::readdir(stream) };
        if entry.is_null() {
            break;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        if entries.len() == 8 {
            unsafe { libc::closedir(stream) };
            return Err(OllamaProcessError::Identity);
        }
        entries.push(std::ffi::OsString::from_vec(bytes.to_vec()));
    }
    unsafe { libc::closedir(stream) };
    Ok(entries)
}

fn open_at_directory(parent: &File, name: &std::ffi::OsStr) -> Option<File> {
    let name = CString::new(name.as_bytes()).ok()?;
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0,
        )
    };
    (fd >= 0).then(|| unsafe { File::from_raw_fd(fd) })
}

fn read_owner(directory: &File) -> Option<u32> {
    let name = CString::new(OWNER_FILE).ok()?;
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0,
        )
    };
    if fd < 0 {
        return None;
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut bytes = [0_u8; 16];
    let length = file.read(&mut bytes).ok()?;
    std::str::from_utf8(&bytes[..length])
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn process_is_alive(pid: u32) -> bool {
    if pid < 2 || pid > i32::MAX as u32 {
        return false;
    }
    (unsafe { libc::kill(pid as libc::pid_t, 0) == 0 })
        || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

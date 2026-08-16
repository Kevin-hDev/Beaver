use super::super::error::OllamaErrorCode;
use super::relative_components;
use std::ffi::{CString, OsStr};
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

const DIRECTORY_FLAGS: i32 = libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
const FILE_FLAGS: i32 = libc::O_WRONLY | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_CLOEXEC;

pub(super) struct PlatformExtractionRoot {
    root: File,
    allow_existing_files: bool,
}

impl PlatformExtractionRoot {
    pub(super) fn open(path: &Path, require_empty: bool) -> Result<Self, OllamaErrorCode> {
        let root = OpenOptions::new()
            .read(true)
            .custom_flags(DIRECTORY_FLAGS)
            .open(path)
            .map_err(map_error)?;
        if !root.metadata().map_err(map_error)?.is_dir() {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        if require_empty && !is_empty(&root)? {
            return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
        }
        Ok(Self {
            root,
            allow_existing_files: !require_empty,
        })
    }

    pub(super) fn create_directory_all(&self, path: &Path) -> Result<(), OllamaErrorCode> {
        let _ = self.open_parent(path)?;
        Ok(())
    }

    pub(super) fn create_file(&self, path: &Path, mode: u32) -> Result<File, OllamaErrorCode> {
        let components = relative_components(path)?;
        let (name, parent_components) = components
            .split_last()
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        let parent = self.open_components(parent_components)?;
        let name = c_name(name.as_os_str())?;
        let flags = FILE_FLAGS
            | if self.allow_existing_files {
                libc::O_TRUNC
            } else {
                libc::O_EXCL
            };
        let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags, mode & 0o777) };
        if fd < 0 {
            let error = io::Error::last_os_error();
            return Err(map_error(error));
        }
        let file = unsafe { File::from_raw_fd(fd) };
        let metadata = file.metadata().map_err(map_error)?;
        if !metadata.is_file() {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        if unsafe { libc::fchmod(file.as_raw_fd(), (mode & 0o777) as libc::mode_t) } != 0 {
            return Err(OllamaErrorCode::OllamaStorageUnavailable);
        }
        Ok(file)
    }

    pub(super) fn create_symlink(&self, path: &Path, target: &Path) -> Result<(), OllamaErrorCode> {
        let components = relative_components(path)?;
        let (name, parent_components) = components
            .split_last()
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        let parent = self.open_components(parent_components)?;
        let name = c_name(name.as_os_str())?;
        let target = c_name(target.as_os_str())?;
        if unsafe { libc::symlinkat(target.as_ptr(), parent.as_raw_fd(), name.as_ptr()) } != 0 {
            return Err(map_error(io::Error::last_os_error()));
        }
        Ok(())
    }

    fn open_parent(&self, path: &Path) -> Result<File, OllamaErrorCode> {
        let components = relative_components(path)?;
        self.open_components(&components)
    }

    fn open_components(&self, components: &[std::path::PathBuf]) -> Result<File, OllamaErrorCode> {
        let mut current = self.root.try_clone().map_err(map_error)?;
        for component in components {
            let name = c_name(component.as_os_str())?;
            let child = match open_directory(&current, &name, true)? {
                Some(child) => child,
                None => {
                    let result =
                        unsafe { libc::mkdirat(current.as_raw_fd(), name.as_ptr(), 0o755) };
                    if result != 0
                        && std::io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST)
                    {
                        return Err(map_error(io::Error::last_os_error()));
                    }
                    open_directory(&current, &name, false)?
                        .ok_or(OllamaErrorCode::OllamaStorageUnavailable)?
                }
            };
            current = child;
        }
        Ok(current)
    }
}

fn c_name(name: &OsStr) -> Result<CString, OllamaErrorCode> {
    CString::new(name.as_bytes()).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)
}

fn open_directory(
    parent: &File,
    name: &CString,
    allow_missing: bool,
) -> Result<Option<File>, OllamaErrorCode> {
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), DIRECTORY_FLAGS, 0) };
    if fd < 0 {
        let error = io::Error::last_os_error();
        if allow_missing && error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(map_error(error));
    }
    Ok(Some(unsafe { File::from_raw_fd(fd) }))
}

fn is_empty(file: &File) -> Result<bool, OllamaErrorCode> {
    let duplicate = unsafe { libc::dup(file.as_raw_fd()) };
    if duplicate < 0 {
        return Err(OllamaErrorCode::OllamaStorageUnavailable);
    }
    let directory = unsafe { libc::fdopendir(duplicate) };
    if directory.is_null() {
        unsafe { libc::close(duplicate) };
        return Err(OllamaErrorCode::OllamaStorageUnavailable);
    }
    loop {
        reset_errno();
        let entry = unsafe { libc::readdir(directory) };
        if entry.is_null() {
            let error = io::Error::last_os_error();
            unsafe { libc::closedir(directory) };
            return if error.raw_os_error().unwrap_or(0) == 0 {
                Ok(true)
            } else {
                Err(OllamaErrorCode::OllamaStorageUnavailable)
            };
        }
        let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) };
        if name.to_bytes() != b"." && name.to_bytes() != b".." {
            unsafe { libc::closedir(directory) };
            return Ok(false);
        }
    }
}

fn reset_errno() {
    #[cfg(target_os = "macos")]
    unsafe {
        *libc::__error() = 0;
    }
    #[cfg(not(target_os = "macos"))]
    unsafe {
        *libc::__errno_location() = 0;
    }
}

fn map_error(error: io::Error) -> OllamaErrorCode {
    match error.raw_os_error() {
        Some(libc::ELOOP) | Some(libc::ENOTDIR) => OllamaErrorCode::OllamaBundleInvalid,
        Some(libc::EEXIST) => OllamaErrorCode::OllamaUpdateRecoveryRequired,
        _ => OllamaErrorCode::OllamaStorageUnavailable,
    }
}

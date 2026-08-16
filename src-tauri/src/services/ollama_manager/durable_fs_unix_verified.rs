use super::super::super::path_identity::{
    CanonicalDirectory, NativePathIdentityResolver, PathIdentityResolver,
};
use super::{sync_parent_path, OllamaFsError, OllamaFsErrorKind};
use std::ffi::{CStr, CString, OsStr};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;

const DIRECTORY_FLAGS: i32 =
    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
const MAX_DELETE_DEPTH: usize = 64;
const MAX_DELETE_ENTRIES: usize = 8_192;

pub(super) fn remove_tree(root: &CanonicalDirectory) -> Result<(), OllamaFsError> {
    let expected = root
        .identity()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let root_handle = root
        .stable_handle()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let current = NativePathIdentityResolver
        .canonical_directory(root.path())
        .map_err(|_| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    if current.identity() != Some(expected) {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let mut removed_entries = 0usize;
    remove_contents(root_handle.as_raw_fd(), 0, &mut removed_entries)?;

    let parent_path = root
        .path()
        .parent()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let parent = NativePathIdentityResolver
        .canonical_directory(parent_path)
        .map_err(|_| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let parent_handle = parent
        .stable_handle()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let leaf = root
        .path()
        .file_name()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let current = NativePathIdentityResolver
        .canonical_directory(root.path())
        .map_err(|_| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    if current.identity() != Some(expected) {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let leaf = c_name(leaf)?;
    let result =
        unsafe { libc::unlinkat(parent_handle.as_raw_fd(), leaf.as_ptr(), libc::AT_REMOVEDIR) };
    if result != 0 {
        return Err(io_error());
    }
    sync_parent_path(root.path())
}

fn remove_contents(
    fd: RawFd,
    depth: usize,
    removed_entries: &mut usize,
) -> Result<(), OllamaFsError> {
    let duplicate = unsafe { libc::dup(fd) };
    if duplicate < 0 {
        return Err(io_error());
    }
    let directory = unsafe { libc::fdopendir(duplicate) };
    if directory.is_null() {
        unsafe { libc::close(duplicate) };
        return Err(io_error());
    }
    loop {
        let entry = unsafe { libc::readdir(directory) };
        if entry.is_null() {
            break;
        }
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
        if name.to_bytes() == b"." || name.to_bytes() == b".." {
            continue;
        }
        if *removed_entries >= MAX_DELETE_ENTRIES {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        *removed_entries += 1;
        remove_entry(fd, name, depth, removed_entries)?;
    }
    unsafe { libc::closedir(directory) };
    Ok(())
}

fn remove_entry(
    parent: RawFd,
    name: &CStr,
    depth: usize,
    removed_entries: &mut usize,
) -> Result<(), OllamaFsError> {
    let mut stat = unsafe { std::mem::zeroed::<libc::stat>() };
    let result =
        unsafe { libc::fstatat(parent, name.as_ptr(), &mut stat, libc::AT_SYMLINK_NOFOLLOW) };
    if result != 0 {
        return Err(io_error());
    }
    if stat.st_mode & libc::S_IFMT == libc::S_IFDIR {
        if depth >= MAX_DELETE_DEPTH {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        let child = unsafe { libc::openat(parent, name.as_ptr(), DIRECTORY_FLAGS) };
        if child < 0 {
            return Err(io_error());
        }
        let result = remove_contents(child, depth + 1, removed_entries);
        unsafe { libc::close(child) };
        result?;
        let result = unsafe { libc::unlinkat(parent, name.as_ptr(), libc::AT_REMOVEDIR) };
        if result != 0 {
            return Err(io_error());
        }
    } else {
        let result = unsafe { libc::unlinkat(parent, name.as_ptr(), 0) };
        if result != 0 {
            return Err(io_error());
        }
    }
    Ok(())
}

fn c_name(name: &OsStr) -> Result<CString, OllamaFsError> {
    CString::new(name.as_bytes()).map_err(|_| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))
}

fn io_error() -> OllamaFsError {
    OllamaFsError::new(OllamaFsErrorKind::Other)
}

use super::super::super::canonical_executable::{
    CanonicalExecutable, NativeFileIdentity, StableFileHandle,
};
use super::super::{
    CanonicalDirectory, NativeDirectoryIdentity, OllamaError, StableDirectoryHandle,
    ValidatedPathComponent, VerifiedDirectoryLocation,
};
use std::ffi::{CString, OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

const DIRECTORY_FLAGS: i32 = libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
const FILE_FLAGS: i32 = libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK;

fn classify(error: std::io::Error) -> OllamaError {
    match error.raw_os_error() {
        Some(libc::ELOOP) => super::super::OllamaErrorCode::OllamaModelStoreConflict,
        _ => super::super::OllamaErrorCode::OllamaStorageUnavailable,
    }
}

fn error_code(
    error: std::io::Error,
    allow_missing: bool,
    final_component: bool,
) -> Result<Option<File>, OllamaError> {
    if allow_missing && error.raw_os_error() == Some(libc::ENOENT) {
        Ok(None)
    } else if final_component && error.raw_os_error() == Some(libc::ENOTDIR) {
        Err(super::super::OllamaErrorCode::OllamaModelStoreConflict)
    } else {
        Err(classify(error))
    }
}

fn symlink_at(parent: &File, component: &OsStr) -> bool {
    let Ok(name) = CString::new(component.as_bytes()) else {
        return false;
    };
    let mut target = [0u8; libc::PATH_MAX as usize];
    unsafe {
        libc::readlinkat(
            parent.as_raw_fd(),
            name.as_ptr(),
            target.as_mut_ptr().cast(),
            target.len(),
        ) >= 0
    }
}

fn open_at(
    parent: &File,
    component: &OsStr,
    flags: i32,
    allow_missing: bool,
    final_component: bool,
) -> Result<Option<File>, OllamaError> {
    let name = CString::new(component.as_bytes())
        .map_err(|_| super::super::OllamaErrorCode::OllamaModelStoreConflict)?;
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags, 0) };
    if fd < 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOTDIR) && symlink_at(parent, component) {
            return Err(super::super::OllamaErrorCode::OllamaModelStoreConflict);
        }
        return error_code(error, allow_missing, final_component);
    }
    Ok(Some(unsafe { File::from_raw_fd(fd) }))
}

fn base(path: &Path) -> Result<(File, PathBuf), OllamaError> {
    if path.is_absolute() {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(DIRECTORY_FLAGS)
            .open("/")
            .map_err(classify)?;
        return Ok((file, PathBuf::from("/")));
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(DIRECTORY_FLAGS)
        .open(".")
        .map_err(classify)?;
    let current = std::env::current_dir()
        .map_err(|_| super::super::OllamaErrorCode::OllamaStorageUnavailable)?;
    Ok((file, current))
}

fn append_component(path: &mut PathBuf, component: Component<'_>) {
    if let Component::Normal(name) = component {
        path.push(name);
    }
}

fn parent_and_leaf(path: &Path) -> Result<(File, PathBuf, OsString), OllamaError> {
    let (mut current, mut display) = base(path)?;
    let mut components = path.components().peekable();
    let mut leaf = None;
    while let Some(component) = components.next() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) if components.peek().is_none() => {
                leaf = Some(name.to_owned());
            }
            Component::Normal(name) => {
                current = open_at(&current, name, DIRECTORY_FLAGS, false, false)?
                    .ok_or(super::super::OllamaErrorCode::OllamaStorageUnavailable)?;
                append_component(&mut display, Component::Normal(name));
            }
            Component::CurDir | Component::ParentDir | Component::Prefix(_) => {
                return Err(super::super::OllamaErrorCode::OllamaModelStoreConflict)
            }
        }
    }
    leaf.map(|leaf| (current, display, leaf))
        .ok_or(super::super::OllamaErrorCode::OllamaModelStoreConflict)
}

fn directory_from_file(path: PathBuf, file: File) -> Result<CanonicalDirectory, OllamaError> {
    let metadata = file
        .metadata()
        .map_err(|_| super::super::OllamaErrorCode::OllamaStorageUnavailable)?;
    if !metadata.is_dir() {
        return Err(super::super::OllamaErrorCode::OllamaStorageUnavailable);
    }
    Ok(CanonicalDirectory::from_native(
        path,
        Some(NativeDirectoryIdentity::unix(
            metadata.dev(),
            metadata.ino(),
        )),
        Some(StableDirectoryHandle(Arc::new(file))),
    ))
}

pub(super) fn canonical_directory(path: &Path) -> Result<CanonicalDirectory, OllamaError> {
    let (mut current, mut display) = base(path)?;
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                current = open_at(&current, name, DIRECTORY_FLAGS, false, false)?
                    .ok_or(super::super::OllamaErrorCode::OllamaStorageUnavailable)?;
                append_component(&mut display, Component::Normal(name));
            }
            Component::CurDir | Component::ParentDir | Component::Prefix(_) => {
                return Err(super::super::OllamaErrorCode::OllamaModelStoreConflict)
            }
        }
    }
    directory_from_file(display, current)
}

pub(super) fn verified_location(path: &Path) -> Result<VerifiedDirectoryLocation, OllamaError> {
    let (parent_file, parent_path, leaf_os) = parent_and_leaf(path)?;
    let leaf = ValidatedPathComponent::from_os(&leaf_os)?;
    let child = open_at(&parent_file, &leaf_os, DIRECTORY_FLAGS, true, true)?;
    let parent = directory_from_file(parent_path, parent_file)?;
    match child {
        Some(child) => {
            let child_path = parent.path().join(&leaf_os);
            let child = directory_from_file(child_path, child)?;
            Ok(VerifiedDirectoryLocation::native_existing(
                parent, leaf, child,
            ))
        }
        None => Ok(VerifiedDirectoryLocation::absent(parent, leaf)),
    }
}

pub(super) fn canonical_executable(path: &Path) -> Result<CanonicalExecutable, OllamaError> {
    let (parent_file, parent_path, leaf) = parent_and_leaf(path)?;
    let file = open_at(&parent_file, &leaf, FILE_FLAGS, false, true)?
        .ok_or(super::super::OllamaErrorCode::OllamaStorageUnavailable)?;
    let metadata = file
        .metadata()
        .map_err(|_| super::super::OllamaErrorCode::OllamaStorageUnavailable)?;
    if !metadata.is_file() || metadata.mode() & 0o111 == 0 {
        return Err(super::super::OllamaErrorCode::OllamaModelStoreConflict);
    }
    Ok(CanonicalExecutable::from_native(
        parent_path.join(leaf),
        NativeFileIdentity::unix(metadata.dev(), metadata.ino()),
        StableFileHandle(Arc::new(file)),
    ))
}

pub(super) fn ancestor_identity(path: &Path) -> Result<NativeDirectoryIdentity, OllamaError> {
    let directory = canonical_directory(path)?;
    directory
        .identity()
        .cloned()
        .ok_or(super::super::OllamaErrorCode::OllamaStorageUnavailable)
}

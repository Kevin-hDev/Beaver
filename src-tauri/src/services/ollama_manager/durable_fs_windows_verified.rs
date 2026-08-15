use super::super::super::path_identity::{
    CanonicalDirectory, NativePathIdentityResolver, PathIdentityResolver,
};
use super::{sync_parent_path, OllamaFsError, OllamaFsErrorKind};
use std::os::windows::io::AsRawHandle;

#[path = "durable_fs_windows_verified/entries.rs"]
mod entries;
#[path = "durable_fs_windows_verified/handles.rs"]
mod handles;

const MAX_DELETE_DEPTH: usize = 64;

pub(super) fn remove_tree(root: &CanonicalDirectory) -> Result<(), OllamaFsError> {
    let expected = root
        .identity()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let stable = root
        .stable_handle()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let stable_info = handles::file_info(stable.as_raw_handle())?;
    revalidate_root(root, expected, &stable_info)?;
    let deletion = handles::reopen_directory(stable.as_raw_handle())?;
    let deletion_info = handles::file_info(deletion.raw())?;
    if !handles::same_identity(&stable_info, &deletion_info) {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let volume = handles::open_volume(root.path())?;
    let mut removed_entries = 0usize;
    entries::remove_contents(
        deletion.raw(),
        volume.raw(),
        stable_info.dwVolumeSerialNumber,
        0,
        &mut removed_entries,
    )?;
    revalidate_root(root, expected, &stable_info)?;
    handles::mark_deleted(deletion.raw())?;
    sync_parent_path(root.path())
}

fn revalidate_root(
    root: &CanonicalDirectory,
    expected: &super::super::super::path_identity::NativeDirectoryIdentity,
    stable_info: &handles::FileInfo,
) -> Result<(), OllamaFsError> {
    let current = NativePathIdentityResolver
        .canonical_directory(root.path())
        .map_err(|_| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let current_handle = current
        .stable_handle()
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let current_info = handles::file_info(current_handle.as_raw_handle())?;
    if current.identity() != Some(expected) || !handles::same_identity(stable_info, &current_info) {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    Ok(())
}

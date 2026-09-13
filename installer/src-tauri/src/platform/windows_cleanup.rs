use crate::error::InstallerError;
use crate::temp_ownership::OwnedTempRun;
#[cfg(target_os = "windows")]
use crate::temp_ownership::OWNER_MARKER;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
const MAX_RUN_ENTRIES: usize = 4_096;

pub fn cleanup_targets(
    run: &OwnedTempRun,
    executable: &Path,
) -> Result<[PathBuf; 2], InstallerError> {
    let root = run
        .path()
        .canonicalize()
        .map_err(|_| InstallerError::CleanupFailed)?;
    let metadata = fs::symlink_metadata(executable).map_err(|_| InstallerError::CleanupFailed)?;
    let executable = executable
        .canonicalize()
        .map_err(|_| InstallerError::CleanupFailed)?;
    if !metadata.is_file()
        || is_link_or_reparse(&metadata)
        || executable.parent() != Some(root.as_path())
    {
        return Err(InstallerError::CleanupFailed);
    }
    Ok([executable, root])
}

#[cfg(target_os = "windows")]
pub fn schedule_self_cleanup(run: &OwnedTempRun, executable: &Path) -> Result<(), InstallerError> {
    let [executable, root] = cleanup_targets(&run, executable)?;
    let entries = validated_children(&root, &executable)?;
    for entry in entries {
        if entry.is_dir() {
            fs::remove_dir_all(entry).map_err(|_| InstallerError::CleanupFailed)?;
        } else {
            fs::remove_file(entry).map_err(|_| InstallerError::CleanupFailed)?;
        }
    }
    fs::remove_file(root.join(OWNER_MARKER)).map_err(|_| InstallerError::CleanupFailed)?;
    schedule_delete(&executable)?;
    schedule_delete(&root)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn validated_children(root: &Path, executable: &Path) -> Result<Vec<PathBuf>, InstallerError> {
    let mut removable = Vec::new();
    for entry in fs::read_dir(root).map_err(|_| InstallerError::CleanupFailed)? {
        if removable.len() >= MAX_RUN_ENTRIES {
            return Err(InstallerError::CleanupFailed);
        }
        let path = entry.map_err(|_| InstallerError::CleanupFailed)?.path();
        if path == executable || path.file_name().is_some_and(|name| name == OWNER_MARKER) {
            continue;
        }
        validate_tree(&path)?;
        removable.push(path);
    }
    Ok(removable)
}

#[cfg(target_os = "windows")]
fn validate_tree(root: &Path) -> Result<(), InstallerError> {
    let mut pending = vec![root.to_path_buf()];
    let mut count = 0;
    while let Some(path) = pending.pop() {
        count += 1;
        if count > MAX_RUN_ENTRIES {
            return Err(InstallerError::CleanupFailed);
        }
        let metadata = fs::symlink_metadata(&path).map_err(|_| InstallerError::CleanupFailed)?;
        if is_link_or_reparse(&metadata) || (!metadata.is_file() && !metadata.is_dir()) {
            return Err(InstallerError::CleanupFailed);
        }
        if metadata.is_dir() {
            for child in fs::read_dir(path).map_err(|_| InstallerError::CleanupFailed)? {
                pending.push(child.map_err(|_| InstallerError::CleanupFailed)?.path());
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn schedule_delete(path: &Path) -> Result<(), InstallerError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT};

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    (unsafe { MoveFileExW(wide.as_ptr(), std::ptr::null(), MOVEFILE_DELAY_UNTIL_REBOOT) } != 0)
        .then_some(())
        .ok_or(InstallerError::CleanupFailed)
}

#[cfg(unix)]
fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
}

use super::{InstallerTrace, MAX_TRACE_BYTES};
use crate::error::InstallerError;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_FAILURE_TRACES: usize = 3;
const MAX_TRACE_ENTRIES: usize = 128;

pub(super) fn preserve(
    trace: &mut InstallerTrace,
    data_root: &Path,
    run_id: &str,
) -> Result<PathBuf, InstallerError> {
    if !valid_run_id(run_id) {
        return Err(InstallerError::CleanupFailed);
    }
    let file = trace.file.take().ok_or(InstallerError::CleanupFailed)?;
    file.sync_all().map_err(|_| InstallerError::CleanupFailed)?;
    drop(file);
    let directory = prepare_trace_directory(data_root)?;
    rotate(&directory)?;
    let destination = directory.join(format!("installer-{run_id}.jsonl"));
    copy_without_follow(&trace.path, &destination)?;
    fs::remove_file(&trace.path).map_err(|_| InstallerError::CleanupFailed)?;
    sync_directory(&directory).map_err(|_| InstallerError::CleanupFailed)?;
    Ok(destination)
}

fn prepare_trace_directory(data_root: &Path) -> Result<PathBuf, InstallerError> {
    verify_directory(data_root, false)?;
    let logs = data_root.join("logs");
    create_directory(&logs, false)?;
    let installer = logs.join("installer");
    create_directory(&installer, true)?;
    Ok(installer)
}

fn create_directory(path: &Path, private: bool) -> Result<(), InstallerError> {
    if fs::symlink_metadata(path).is_err() {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(path)
            .map_err(|_| InstallerError::CleanupFailed)?;
    }
    verify_directory(path, private)
}

fn verify_directory(path: &Path, private: bool) -> Result<(), InstallerError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| InstallerError::CleanupFailed)?;
    if !metadata.is_dir()
        || is_link_or_reparse(&metadata)
        || !owned_by_current_user(&metadata)
        || (private && !has_private_permissions(&metadata))
    {
        return Err(InstallerError::CleanupFailed);
    }
    Ok(())
}

fn rotate(directory: &Path) -> Result<(), InstallerError> {
    let mut traces = Vec::new();
    for entry in fs::read_dir(directory)
        .map_err(|_| InstallerError::CleanupFailed)?
        .take(MAX_TRACE_ENTRIES + 1)
    {
        if traces.len() == MAX_TRACE_ENTRIES {
            return Err(InstallerError::CleanupFailed);
        }
        let path = entry.map_err(|_| InstallerError::CleanupFailed)?.path();
        let metadata = fs::symlink_metadata(&path).map_err(|_| InstallerError::CleanupFailed)?;
        if !valid_trace(&path, &metadata) {
            return Err(InstallerError::CleanupFailed);
        }
        traces.push((metadata.modified().ok(), path));
    }
    traces.sort_by_key(|entry| entry.0);
    while traces.len() >= MAX_FAILURE_TRACES {
        let (_, path) = traces.remove(0);
        open_without_follow(&path)?;
        fs::remove_file(path).map_err(|_| InstallerError::CleanupFailed)?;
    }
    Ok(())
}

fn valid_trace(path: &Path, metadata: &fs::Metadata) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(run_id) = name
        .strip_prefix("installer-")
        .and_then(|name| name.strip_suffix(".jsonl"))
    else {
        return false;
    };
    valid_run_id(run_id)
        && metadata.is_file()
        && metadata.len() <= MAX_TRACE_BYTES as u64
        && !is_link_or_reparse(metadata)
        && owned_by_current_user(metadata)
        && has_private_permissions(metadata)
}

fn valid_run_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn copy_without_follow(source: &Path, destination: &Path) -> Result<(), InstallerError> {
    let source = open_without_follow(source)?;
    let metadata = source
        .metadata()
        .map_err(|_| InstallerError::CleanupFailed)?;
    if !metadata.is_file() || metadata.len() > MAX_TRACE_BYTES as u64 {
        return Err(InstallerError::CleanupFailed);
    }
    let mut destination = create_private_file(destination)?;
    let copied = std::io::copy(
        &mut source.take(MAX_TRACE_BYTES as u64 + 1),
        &mut destination,
    )
    .map_err(|_| InstallerError::CleanupFailed)?;
    if copied > MAX_TRACE_BYTES as u64 {
        return Err(InstallerError::CleanupFailed);
    }
    destination
        .sync_all()
        .map_err(|_| InstallerError::CleanupFailed)
}

pub(super) fn create_private_file(path: &Path) -> Result<File, InstallerError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000);
    }
    options
        .open(path)
        .map_err(|_| InstallerError::CleanupFailed)
}

fn open_without_follow(path: &Path) -> Result<File, InstallerError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000);
    }
    options
        .open(path)
        .map_err(|_| InstallerError::CleanupFailed)
}

#[cfg(unix)]
fn owned_by_current_user(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    metadata.uid() == unsafe { libc::geteuid() }
}

#[cfg(windows)]
fn owned_by_current_user(_metadata: &fs::Metadata) -> bool {
    true
}

#[cfg(unix)]
fn has_private_permissions(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o077 == 0
}

#[cfg(windows)]
fn has_private_permissions(_metadata: &fs::Metadata) -> bool {
    true
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

#[cfg(unix)]
fn sync_directory(path: &Path) -> std::io::Result<()> {
    File::open(path)?.sync_all()
}

#[cfg(windows)]
fn sync_directory(path: &Path) -> std::io::Result<()> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(0x0200_0000)
        .open(path)?
        .sync_all()
}

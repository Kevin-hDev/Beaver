use rand::RngCore;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

fn private_store_error() -> String {
    error_codes::PRIVATE_STORE_UNAVAILABLE.to_string()
}

pub(crate) use cache::{CachedStore, StoreErrorCodes, StoreFailure, StoreLoad};
pub(crate) use private_store_atomic::atomic_write;

#[path = "private_store/cache.rs"]
mod cache;

#[path = "private_store/atomic_write.rs"]
mod private_store_atomic;

#[path = "private_store/temp_cleanup.rs"]
mod temp_cleanup;

#[path = "private_store/error_codes.rs"]
pub(crate) mod error_codes;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BoundedFile {
    Missing,
    Content(Vec<u8>),
}

pub(crate) fn read_bounded_regular(path: &Path, max_bytes: u64) -> Result<BoundedFile, String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BoundedFile::Missing);
        }
        Err(_) => return Err(private_store_error()),
    };
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(private_store_error());
    }
    let read_limit = max_bytes.checked_add(1).ok_or_else(private_store_error)?;
    let mut content = Vec::new();
    File::open(path)
        .and_then(|file| file.take(read_limit).read_to_end(&mut content))
        .map_err(|_| private_store_error())?;
    if content.len() as u64 > max_bytes {
        return Err(private_store_error());
    }
    Ok(BoundedFile::Content(content))
}

pub async fn atomic_write_async(path: PathBuf, bytes: Vec<u8>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || atomic_write(&path, &bytes))
        .await
        .map_err(|_| private_store_error())?
}

pub async fn read_bounded_regular_async(
    path: PathBuf,
    max_bytes: u64,
) -> Result<BoundedFile, String> {
    tokio::task::spawn_blocking(move || read_bounded_regular(&path, max_bytes))
        .await
        .map_err(|_| private_store_error())?
}

pub async fn write_new_async(path: PathBuf, bytes: Vec<u8>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || write_new(&path, &bytes))
        .await
        .map_err(|_| private_store_error())?
}

pub async fn ensure_private_dir_async(path: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || ensure_private_dir(&path))
        .await
        .map_err(|_| private_store_error())?
}

pub fn ensure_private_dir(path: &Path) -> Result<(), String> {
    create_private_dirs(path)
}

pub fn repair_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    set_private_permissions(path)
}

pub fn repair_app_storage() -> Result<(), String> {
    let root = crate::services::paths::data_dir();
    create_private_dirs(&root)?;
    temp_cleanup::purge_stale_atomic_temps_logged(&root);
    for directory in [
        root.join("agent-sessions"),
        root.join("forecast-notes"),
        root.join("logs"),
    ] {
        create_private_dirs(&directory)?;
    }
    for file in [
        root.join("secrets.enc"),
        root.join("configured-providers.json"),
        root.join("mcp-connectors.json"),
        root.join("provider-usage.json"),
        root.join("agent-sessions/gateway-session-map.json"),
        root.join("logs/gateway-audit.jsonl"),
    ] {
        repair_path(&file)?;
    }
    Ok(())
}

fn create_private_dirs(path: &Path) -> Result<(), String> {
    let mut missing = Vec::new();
    let mut current = path;
    while !current.exists() {
        missing.push(current.to_path_buf());
        current = current.parent().ok_or_else(private_store_error)?;
    }
    std::fs::create_dir_all(path).map_err(|_| private_store_error())?;
    for directory in missing.iter().rev() {
        set_private_permissions(directory)?;
    }
    set_private_permissions(path)
}

fn temp_path(path: &Path) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(private_store_error)?;
    let mut random = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut random);
    Ok(path.with_file_name(format!(".{name}.{}.tmp", hex::encode(random))))
}

fn open_private_file(path: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(|_| private_store_error())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(private_store_error)?;
    create_private_dirs(parent)?;
    let mut file = open_private_file(path)?;
    let result = file
        .write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| private_store_error())
        .and_then(|_| repair_path(path))
        .and_then(|_| sync_parent(parent));
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if path.is_dir() { 0o700 } else { 0o600 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .map_err(|_| private_store_error())
}

#[cfg(windows)]
fn set_private_permissions(path: &Path) -> Result<(), String> {
    private_store_windows::secure_acl(path)
}

#[cfg(not(any(unix, windows)))]
fn set_private_permissions(_path: &Path) -> Result<(), String> {
    Err(private_store_error())
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    private_store_windows::replace_file(source, destination)
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::rename(source, destination).map_err(|_| private_store_error())
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<(), String> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| private_store_error())
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
#[path = "private_store/private_store_windows.rs"]
mod private_store_windows;
#[cfg(windows)]
#[path = "private_store/windows_acl.rs"]
mod windows_acl;
#[cfg(windows)]
#[path = "private_store/windows_token.rs"]
mod windows_token;
#[cfg(windows)]
pub(crate) use windows_token::current_user as current_windows_user;

#[cfg(test)]
#[path = "private_store_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "private_store_interrupt_tests.rs"]
mod interrupt_tests;

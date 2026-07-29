use rand::RngCore;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "stockage privé indisponible".to_string())?;
    create_private_dirs(parent)?;
    let temp = temp_path(path)?;
    let result = (|| {
        let mut file = open_private_file(&temp)?;
        file.write_all(bytes)
            .map_err(|_| "stockage privé indisponible".to_string())?;
        file.sync_all()
            .map_err(|_| "stockage privé indisponible".to_string())?;
        replace_file(&temp, path)?;
        repair_path(path)?;
        sync_parent(parent)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

pub async fn atomic_write_async(path: PathBuf, bytes: Vec<u8>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || atomic_write(&path, &bytes))
        .await
        .map_err(|_| "stockage privé indisponible".to_string())?
}

pub async fn write_new_async(path: PathBuf, bytes: Vec<u8>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || write_new(&path, &bytes))
        .await
        .map_err(|_| "stockage privé indisponible".to_string())?
}

pub async fn ensure_private_dir_async(path: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || ensure_private_dir(&path))
        .await
        .map_err(|_| "stockage privé indisponible".to_string())?
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
        current = current
            .parent()
            .ok_or_else(|| "stockage privé indisponible".to_string())?;
    }
    std::fs::create_dir_all(path).map_err(|_| "stockage privé indisponible".to_string())?;
    for directory in missing.iter().rev() {
        set_private_permissions(directory)?;
    }
    set_private_permissions(path)
}

fn temp_path(path: &Path) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "stockage privé indisponible".to_string())?;
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
    options
        .open(path)
        .map_err(|_| "stockage privé indisponible".to_string())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "stockage privé indisponible".to_string())?;
    create_private_dirs(parent)?;
    let mut file = open_private_file(path)?;
    let result = file
        .write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "stockage privé indisponible".to_string())
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
        .map_err(|_| "stockage privé indisponible".to_string())
}

#[cfg(windows)]
fn set_private_permissions(path: &Path) -> Result<(), String> {
    private_store_windows::secure_acl(path)
}

#[cfg(not(any(unix, windows)))]
fn set_private_permissions(_path: &Path) -> Result<(), String> {
    Err("stockage privé indisponible".to_string())
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    private_store_windows::replace_file(source, destination)
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::rename(source, destination).map_err(|_| "stockage privé indisponible".to_string())
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<(), String> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| "stockage privé indisponible".to_string())
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

#[cfg(test)]
#[path = "private_store_tests.rs"]
mod tests;

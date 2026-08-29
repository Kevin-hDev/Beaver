use std::path::{Path, PathBuf};
#[cfg(debug_assertions)]
use std::sync::OnceLock;

const MAX_REPORTS: usize = 64;
const MAX_DIRECTORY_ENTRIES: usize = 256;
#[cfg(debug_assertions)]
const MAX_REPORT_BYTES: usize = 256 * 1024;
#[cfg(debug_assertions)]
static WRITE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[path = "reasoning_fixture_store_id.rs"]
mod fixture_id;
pub(crate) use fixture_id::{derive_fixture_id, derive_fixture_id_with_variant};
use fixture_id::{is_report_name, validate_fixture_id};

#[cfg(debug_assertions)]
pub async fn write_report(
    session_id: &str,
    fixture_id: &str,
    report: Vec<u8>,
) -> Result<(), String> {
    crate::services::agent_local::session_id::validate_session_id(session_id)?;
    validate_fixture_id(fixture_id)?;
    if report.is_empty() || report.len() > MAX_REPORT_BYTES {
        return Err(unavailable());
    }
    let root = crate::services::paths::data_dir().join("reasoning-fixture-reports");
    let _guard = WRITE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;
    let session_id = session_id.to_string();
    let fixture_id = fixture_id.to_string();
    tokio::task::spawn_blocking(move || write_at(&root, &session_id, &fixture_id, &report))
        .await
        .map_err(|_| unavailable())?
}

pub async fn remove_for_session(session_id: &str) -> Result<(), String> {
    crate::services::agent_local::session_id::validate_session_id(session_id)?;
    let path = crate::services::paths::data_dir()
        .join("reasoning-fixture-reports")
        .join(session_id);
    tokio::task::spawn_blocking(move || remove_private_dir(&path))
        .await
        .map_err(|_| unavailable())?
}

#[cfg(any(debug_assertions, test))]
fn write_at(root: &Path, session_id: &str, fixture_id: &str, report: &[u8]) -> Result<(), String> {
    let directory = root.join(session_id);
    crate::services::private_store::ensure_private_dir(&directory).map_err(|_| unavailable())?;
    prune(&directory)?;
    crate::services::private_store::atomic_write(
        &directory.join(format!("{fixture_id}.json")),
        report,
    )
    .map_err(|_| unavailable())
}

#[cfg(any(debug_assertions, test))]
fn prune(directory: &Path) -> Result<(), String> {
    let mut reports = valid_reports(directory)?;
    if reports.len() < MAX_REPORTS {
        return Ok(());
    }
    reports.sort_by_key(|entry| entry.1);
    let path = reports
        .first()
        .map(|entry| entry.0.clone())
        .ok_or_else(unavailable)?;
    std::fs::remove_file(path).map_err(|_| unavailable())
}

fn remove_private_dir(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(unavailable()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unavailable());
    }
    for (index, entry) in std::fs::read_dir(path)
        .map_err(|_| unavailable())?
        .enumerate()
    {
        if index >= MAX_DIRECTORY_ENTRIES {
            return Err(unavailable());
        }
        let entry = entry.map_err(|_| unavailable())?;
        let file_type = entry.file_type().map_err(|_| unavailable())?;
        if file_type.is_symlink() || !file_type.is_file() {
            return Err(unavailable());
        }
        let name = entry.file_name();
        if name.to_str().is_some_and(is_report_name) {
            std::fs::remove_file(entry.path()).map_err(|_| unavailable())?;
        }
    }
    match std::fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => Ok(()),
        Err(_) => Err(unavailable()),
    }
}

fn valid_reports(directory: &Path) -> Result<Vec<(PathBuf, std::time::SystemTime)>, String> {
    let mut reports = Vec::new();
    for (index, entry) in std::fs::read_dir(directory)
        .map_err(|_| unavailable())?
        .enumerate()
    {
        if index >= MAX_DIRECTORY_ENTRIES {
            return Err(unavailable());
        }
        let entry = entry.map_err(|_| unavailable())?;
        if reports.len() >= MAX_REPORTS {
            return Err(unavailable());
        }
        let file_type = entry.file_type().map_err(|_| unavailable())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(unavailable)?;
        if file_type.is_symlink() || !file_type.is_file() {
            return Err(unavailable());
        }
        if !is_report_name(name) {
            continue;
        }
        let modified = entry
            .metadata()
            .map_err(|_| unavailable())?
            .modified()
            .map_err(|_| unavailable())?;
        reports.push((entry.path(), modified));
    }
    Ok(reports)
}

fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}

#[cfg(test)]
#[path = "reasoning_fixture_store_tests.rs"]
mod tests;

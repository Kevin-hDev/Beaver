use std::path::{Path, PathBuf};

const MAX_REPORTS: usize = 64;
#[cfg(debug_assertions)]
const MAX_REPORT_BYTES: usize = 256 * 1024;

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
    for (entry, _) in valid_reports(path)? {
        std::fs::remove_file(entry).map_err(|_| unavailable())?;
    }
    std::fs::remove_dir(path).map_err(|_| unavailable())
}

fn valid_reports(directory: &Path) -> Result<Vec<(PathBuf, std::time::SystemTime)>, String> {
    let mut reports = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(|_| unavailable())? {
        let entry = entry.map_err(|_| unavailable())?;
        if reports.len() >= MAX_REPORTS {
            return Err(unavailable());
        }
        let file_type = entry.file_type().map_err(|_| unavailable())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(unavailable)?;
        if file_type.is_symlink() || !file_type.is_file() || !is_report_name(name) {
            return Err(unavailable());
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

#[cfg(debug_assertions)]
fn validate_fixture_id(fixture_id: &str) -> Result<(), String> {
    uuid::Uuid::parse_str(fixture_id)
        .map(|_| ())
        .map_err(|_| unavailable())
}

fn is_report_name(name: &str) -> bool {
    name.strip_suffix(".json")
        .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
}

fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_evicts_the_oldest_report_without_following_links() {
        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary.path().join("reports");
        let session = "11111111-1111-4111-8111-111111111111";
        for _ in 0..MAX_REPORTS {
            write_at(
                &directory,
                session,
                &uuid::Uuid::new_v4().to_string(),
                b"{}",
            )
            .unwrap();
        }
        write_at(
            &directory,
            session,
            &uuid::Uuid::new_v4().to_string(),
            b"{}",
        )
        .unwrap();
        assert_eq!(
            valid_reports(&directory.join(session)).unwrap().len(),
            MAX_REPORTS
        );
    }

    #[test]
    fn rejects_noncanonical_report_names() {
        assert!(!is_report_name("report.json"));
        assert!(!is_report_name("../report.json"));
    }
}

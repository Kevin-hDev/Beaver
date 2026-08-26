use std::path::PathBuf;

const MAX_REPORTS: usize = 64;

pub async fn remove_for_session(session_id: &str) -> Result<(), String> {
    crate::services::agent_local::session_id::validate_session_id(session_id)?;
    let path = crate::services::paths::data_dir()
        .join("reasoning-fixture-reports")
        .join(session_id);
    tokio::task::spawn_blocking(move || remove_private_dir(path))
        .await
        .map_err(|_| unavailable())?
}

fn remove_private_dir(path: PathBuf) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(unavailable()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unavailable());
    }
    let entries = std::fs::read_dir(&path)
        .map_err(|_| unavailable())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| unavailable())?;
    if entries.len() > MAX_REPORTS
        || entries.iter().any(|entry| {
            entry
                .file_type()
                .map_or(true, |kind| kind.is_symlink() || !kind.is_file())
        })
    {
        return Err(unavailable());
    }
    std::fs::remove_dir_all(path).map_err(|_| unavailable())
}

fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}

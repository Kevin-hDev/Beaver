use std::path::{Path, PathBuf};

const QUARANTINE_DIR: &str = "quarantine";

pub(crate) async fn session_paths(session_id: &str) -> Result<Vec<PathBuf>, String> {
    super::session_store::validate_session_id(session_id)?;
    let directory = super::stream_recovery_store::root().join(session_id);
    tokio::task::spawn_blocking(move || list_logs(&directory))
        .await
        .map_err(|_| error())?
}

pub(crate) async fn all_session_ids() -> Result<Vec<String>, String> {
    let root = super::stream_recovery_store::root();
    tokio::task::spawn_blocking(move || list_session_ids(&root))
    .await
    .map_err(|_| error())?
}

pub(super) fn list_session_ids(root: &Path) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();
    let Some(entries) = read_dir_if_present(root)? else {
        return Ok(ids);
    };
    for (inspected, entry) in entries.enumerate() {
        if inspected >= super::stream_recovery_store::MAX_DISCOVERY_ENTRIES {
            return Err(error());
        }
        let entry = entry.map_err(|_| error())?;
        if !entry.file_type().map_err(|_| error())?.is_dir()
            || entry.file_name() == QUARANTINE_DIR
        {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if uuid::Uuid::parse_str(&id).is_ok() {
            ids.push(id);
        }
    }
    Ok(ids)
}

pub(crate) async fn claim(path: PathBuf) -> Result<PathBuf, String> {
    tokio::task::spawn_blocking(move || {
        let name = path.file_name().and_then(|name| name.to_str()).ok_or_else(error)?;
        if name.ends_with(".recovering") {
            return Ok(path);
        }
        if !name.ends_with(".jsonl") {
            return Err(error());
        }
        let claimed = path.with_extension("jsonl.recovering");
        std::fs::rename(&path, &claimed).map_err(|_| error())?;
        Ok(claimed)
    })
    .await
    .map_err(|_| error())?
}

pub(crate) async fn quarantine(path: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let session_id = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|value| value.to_str())
            .filter(|value| uuid::Uuid::parse_str(value).is_ok())
            .ok_or_else(error)?;
        let directory = super::stream_recovery_store::root()
            .join(QUARANTINE_DIR)
            .join(session_id);
        crate::services::private_store::ensure_private_dir(&directory).map_err(|_| error())?;
        let mut existing = std::fs::read_dir(&directory)
            .map_err(|_| error())?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .collect::<Vec<_>>();
        existing.sort_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok());
        while existing.len() >= super::stream_recovery_store::MAX_LOGS_PER_SESSION {
            let oldest = existing.remove(0);
            std::fs::remove_file(oldest.path()).map_err(|_| error())?;
        }
        let name = format!("{}.bad", uuid::Uuid::new_v4());
        std::fs::rename(path, directory.join(name)).map_err(|_| error())
    })
    .await
    .map_err(|_| error())?
}

pub(crate) async fn remove_session(session_id: &str) -> Result<(), String> {
    super::session_store::validate_session_id(session_id)?;
    let root = super::stream_recovery_store::root();
    let directories = [root.join(session_id), root.join(QUARANTINE_DIR).join(session_id)];
    tokio::task::spawn_blocking(move || {
        for directory in directories {
            match std::fs::remove_dir_all(directory) {
                Ok(()) => {}
                Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(error()),
            }
        }
        Ok(())
    })
    .await
    .map_err(|_| error())?
}

fn list_logs(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let Some(entries) = read_dir_if_present(directory)? else {
        return Ok(Vec::new());
    };
    let mut paths = Vec::new();
    for (inspected, entry) in entries.enumerate() {
        if inspected >= super::stream_recovery_store::MAX_DISCOVERY_ENTRIES {
            return Err(error());
        }
        let entry = entry.map_err(|_| error())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if entry.file_type().map_err(|_| error())?.is_file()
            && (name.ends_with(".jsonl") || name.ends_with(".jsonl.recovering"))
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}

fn read_dir_if_present(path: &Path) -> Result<Option<std::fs::ReadDir>, String> {
    match std::fs::read_dir(path) {
        Ok(entries) => Ok(Some(entries)),
        Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(error()),
    }
}

fn error() -> String {
    "stream_recovery_unavailable".into()
}

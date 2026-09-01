use super::data_profiles::{
    is_profile_file, legacy_profile_path_for_read, legacy_profile_path_for_write,
    profile_directory, profile_path_for_read, profile_path_for_write, PROFILE_LOCK,
};
use super::data_profiles_load::{read_stored, DataProfileLoadError, StoredDataProfile};
use crate::services::workspace_scope::WorkspaceScope;
use std::collections::BTreeSet;
use std::path::Path;

const MAX_RELEASE_SCAN: usize = 1_000;

pub(super) async fn claim_legacy(
    workspace: &WorkspaceScope,
    id: &str,
) -> Result<StoredDataProfile, DataProfileLoadError> {
    let _guard = PROFILE_LOCK.lock().await;
    if let Ok(path) = profile_path_for_read(workspace, id).await {
        let stored = read_for_workspace(&path, id, workspace).await?;
        purge_completed_marker(workspace, id).await?;
        return Ok(stored);
    }
    let legacy_path = legacy_profile_path_for_read(id)
        .await
        .map_err(classify_io)?;
    let mut stored = read_stored(&legacy_path, id).await?;
    if stored.workspace == WorkspaceScope::Legacy {
        stored.workspace = workspace.clone();
        write_stored(&legacy_path, &stored).await?;
    } else if stored.workspace != *workspace {
        return Err(DataProfileLoadError::NotFound);
    }
    let target = profile_path_for_write(workspace, id)
        .await
        .map_err(|_| DataProfileLoadError::Unavailable)?;
    write_stored(&target, &stored).await?;
    // La copie isolée est durable avant cette suppression : une interruption
    // laisse donc toujours une source complète et la reprise reste idempotente.
    purge_completed_marker(workspace, id).await?;
    Ok(stored)
}

async fn purge_completed_marker(
    workspace: &WorkspaceScope,
    id: &str,
) -> Result<(), DataProfileLoadError> {
    let marker = match legacy_profile_path_for_read(id).await {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(classify_io(error)),
    };
    let stored = read_stored(&marker, id).await?;
    if stored.workspace != *workspace {
        return Ok(());
    }
    tokio::fs::remove_file(marker).await.map_err(classify_io)
}

pub(super) async fn stage_release(workspace: &WorkspaceScope) -> Result<Vec<String>, String> {
    let _guard = PROFILE_LOCK.lock().await;
    let mut ids = markers_for(workspace).await?;
    let directory =
        profile_directory(workspace).map_err(|_| "Migration des profils impossible".to_string())?;
    let path = crate::services::paths::data_dir().join(directory);
    for source in profile_files(&path).await? {
        let Some(id) = file_id(&source) else { continue };
        let stored = read_stored(&source, &id)
            .await
            .map_err(|_| "Migration des profils impossible".to_string())?;
        if stored.workspace != *workspace {
            return Err("Migration des profils impossible".into());
        }
        if let Ok(existing) = legacy_profile_path_for_read(&id).await {
            let marker = read_stored(&existing, &id)
                .await
                .map_err(|_| "Migration des profils impossible".to_string())?;
            if marker.workspace != *workspace {
                return Err("Migration des profils impossible".into());
            }
        }
        let marker = legacy_profile_path_for_write(&id).await?;
        write_stored_string(&marker, &stored).await?;
        ids.insert(id);
    }
    Ok(ids.into_iter().collect())
}

pub(super) async fn commit_release(
    workspace: &WorkspaceScope,
    ids: &[String],
) -> Result<(), String> {
    let _guard = PROFILE_LOCK.lock().await;
    for id in ids {
        let marker = legacy_profile_path_for_read(id)
            .await
            .map_err(|_| "Migration des profils impossible".to_string())?;
        let mut stored = read_stored(&marker, id)
            .await
            .map_err(|_| "Migration des profils impossible".to_string())?;
        if stored.workspace != *workspace && stored.workspace != WorkspaceScope::Legacy {
            return Err("Migration des profils impossible".into());
        }
        if let Ok(source) = profile_path_for_read(workspace, id).await {
            tokio::fs::remove_file(source)
                .await
                .map_err(|_| "Migration des profils impossible".to_string())?;
        }
        stored.workspace = WorkspaceScope::Legacy;
        write_stored_string(&marker, &stored).await?;
    }
    Ok(())
}

async fn markers_for(workspace: &WorkspaceScope) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    let directory = crate::services::paths::data_dir().join("forecast-data-profiles");
    for path in profile_files(&directory).await? {
        let Some(id) = file_id(&path) else { continue };
        if read_stored(&path, &id)
            .await
            .is_ok_and(|stored| stored.workspace == *workspace)
        {
            ids.insert(id);
        }
    }
    Ok(ids)
}

async fn profile_files(directory: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut entries = match tokio::fs::read_dir(directory).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err("Migration des profils impossible".into()),
    };
    let mut files = Vec::new();
    let mut scanned = 0usize;
    while scanned < MAX_RELEASE_SCAN {
        let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|_| "Migration des profils impossible".to_string())?
        else {
            return Ok(files);
        };
        scanned += 1;
        let path = entry.path();
        if is_profile_file(&path) {
            files.push(path);
        }
    }
    if entries
        .next_entry()
        .await
        .map_err(|_| "Migration des profils impossible".to_string())?
        .is_some()
    {
        return Err("Migration des profils impossible".into());
    }
    Ok(files)
}

fn file_id(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| uuid::Uuid::parse_str(value).is_ok())
        .map(str::to_string)
}

async fn read_for_workspace(
    path: &Path,
    id: &str,
    workspace: &WorkspaceScope,
) -> Result<StoredDataProfile, DataProfileLoadError> {
    let stored = read_stored(path, id).await?;
    (stored.workspace == *workspace)
        .then_some(stored)
        .ok_or(DataProfileLoadError::Corrupt)
}

async fn write_stored(path: &Path, stored: &StoredDataProfile) -> Result<(), DataProfileLoadError> {
    let json = serde_json::to_vec_pretty(stored).map_err(|_| DataProfileLoadError::Corrupt)?;
    crate::services::private_store::atomic_write_async(path.to_path_buf(), json)
        .await
        .map_err(|_| DataProfileLoadError::Unavailable)
}

async fn write_stored_string(path: &Path, stored: &StoredDataProfile) -> Result<(), String> {
    write_stored(path, stored)
        .await
        .map_err(|_| "Migration des profils impossible".to_string())
}

fn classify_io(error: std::io::Error) -> DataProfileLoadError {
    if error.kind() == std::io::ErrorKind::NotFound {
        DataProfileLoadError::NotFound
    } else {
        DataProfileLoadError::Unavailable
    }
}

#[cfg(test)]
#[path = "data_profiles_lifecycle_tests.rs"]
mod tests;

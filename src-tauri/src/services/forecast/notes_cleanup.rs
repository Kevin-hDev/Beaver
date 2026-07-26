use super::{notes_paths, notes_transaction, notes_validation, storage};
use std::path::{Path, PathBuf};

const PENDING_PREFIX: &str = ".delete-";
const MAX_PENDING_DELETIONS: usize = 64;
const MAX_DIRECTORY_ENTRIES: usize = 1024;

pub async fn delete_analysis(analysis_id: &str) -> Result<(), String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    let _guard = notes_transaction::lock().await;
    recover_pending_locked().await?;
    let staged = stage_notes(analysis_id).await?;
    if let Err(error) = storage::delete(analysis_id).await {
        if restore_staged(analysis_id, staged.as_deref())
            .await
            .is_err()
        {
            return Err("Suppression échouée".into());
        }
        return Err(error);
    }
    if let Some(path) = staged {
        if tokio::fs::remove_dir_all(path).await.is_err() {
            eprintln!("[forecast] nettoyage différé des notes");
        }
    }
    Ok(())
}

pub async fn recover_pending_deletions() -> Result<(), String> {
    let _guard = notes_transaction::lock().await;
    recover_pending_locked().await
}

async fn recover_pending_locked() -> Result<(), String> {
    let root = notes_paths::root_for_write().await?;
    let mut entries = tokio::fs::read_dir(&root)
        .await
        .map_err(|_| cleanup_error())?;
    let mut inspected = 0;
    let mut scanned = 0;
    while let Some(entry) = entries.next_entry().await.map_err(|_| cleanup_error())? {
        scanned += 1;
        if scanned > MAX_DIRECTORY_ENTRIES {
            return Err(cleanup_error());
        }
        if inspected >= MAX_PENDING_DELETIONS {
            break;
        }
        let Some(analysis_id) = pending_analysis_id(&entry.file_name()) else {
            continue;
        };
        inspected += 1;
        let pending = notes_paths::verify_child_directory(&root, &entry.path()).await?;
        if storage::exists(&analysis_id).await? {
            restore_staged(&analysis_id, Some(&pending)).await?;
        } else {
            tokio::fs::remove_dir_all(pending)
                .await
                .map_err(|_| cleanup_error())?;
        }
    }
    Ok(())
}

async fn stage_notes(analysis_id: &str) -> Result<Option<PathBuf>, String> {
    let Some(live) = notes_paths::directory_if_exists(analysis_id).await? else {
        return Ok(None);
    };
    let root = notes_paths::root_for_write().await?;
    let pending = root.join(format!("{PENDING_PREFIX}{analysis_id}"));
    if tokio::fs::try_exists(&pending)
        .await
        .map_err(|_| cleanup_error())?
    {
        return Err(cleanup_error());
    }
    tokio::fs::rename(live, &pending)
        .await
        .map_err(|_| cleanup_error())?;
    notes_paths::verify_child_directory(&root, &pending)
        .await
        .map(Some)
}

async fn restore_staged(analysis_id: &str, staged: Option<&Path>) -> Result<(), String> {
    let Some(staged) = staged else {
        return Ok(());
    };
    let root = notes_paths::root_for_write().await?;
    let live = root.join(analysis_id);
    if tokio::fs::try_exists(&live)
        .await
        .map_err(|_| cleanup_error())?
    {
        tokio::fs::remove_dir_all(staged)
            .await
            .map_err(|_| cleanup_error())?;
        return Ok(());
    }
    tokio::fs::rename(staged, &live)
        .await
        .map_err(|_| cleanup_error())?;
    notes_paths::verify_child_directory(&root, &live)
        .await
        .map(|_| ())
}

fn pending_analysis_id(name: &std::ffi::OsStr) -> Option<String> {
    let value = name.to_str()?.strip_prefix(PENDING_PREFIX)?;
    notes_validation::id(value, "Identifiant d'analyse invalide").ok()?;
    uuid::Uuid::parse_str(value).ok().map(|_| value.to_string())
}

fn cleanup_error() -> String {
    "Nettoyage des notes impossible".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_names_require_a_valid_analysis_uuid() {
        assert!(pending_analysis_id(std::ffi::OsStr::new(".delete-deadbeef")).is_none());
        assert!(pending_analysis_id(std::ffi::OsStr::new(".delete-../escape")).is_none());
        assert!(pending_analysis_id(std::ffi::OsStr::new(
            ".delete-550e8400-e29b-41d4-a716-446655440000"
        ))
        .is_some());
    }

    #[tokio::test]
    async fn staged_notes_can_be_restored() {
        let analysis_id = uuid::Uuid::new_v4().to_string();
        let live = notes_paths::directory_for_write(&analysis_id)
            .await
            .unwrap();
        tokio::fs::write(live.join("marker"), b"note")
            .await
            .unwrap();

        let staged = stage_notes(&analysis_id).await.unwrap().unwrap();
        assert!(!live.exists());
        restore_staged(&analysis_id, Some(&staged)).await.unwrap();

        assert!(live.join("marker").exists());
        tokio::fs::remove_dir_all(live).await.unwrap();
    }

    #[tokio::test]
    async fn recovery_removes_notes_after_a_completed_analysis_deletion() {
        let analysis_id = uuid::Uuid::new_v4().to_string();
        let root = notes_paths::root_for_write().await.unwrap();
        let pending = root.join(format!("{PENDING_PREFIX}{analysis_id}"));
        crate::services::private_store::ensure_private_dir_async(pending.clone())
            .await
            .unwrap();

        recover_pending_deletions().await.unwrap();

        assert!(!pending.exists());
    }
}

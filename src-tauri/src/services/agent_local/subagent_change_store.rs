use super::types_subagent_change::{SubagentChangeMeta, SubagentChangeStatus};
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;

const MAX_STORED_CHANGES: usize = 256;
const MAX_CHANGE_FILE_BYTES: u64 = 128 * 1024;
static STORE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn path(child_id: &str) -> Result<PathBuf, String> {
    super::types_subagent_change::validate_uuid(child_id)?;
    Ok(crate::services::paths::data_dir()
        .join("subagent-changes")
        .join(format!("{child_id}.json")))
}

pub async fn load(child_id: &str) -> Result<SubagentChangeMeta, String> {
    load_optional(child_id)
        .await?
        .ok_or_else(|| "Changement sous-agent indisponible".to_string())
}

pub async fn load_optional(child_id: &str) -> Result<Option<SubagentChangeMeta>, String> {
    let data = match tokio::fs::read_to_string(path(child_id)?).await {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("Changement sous-agent indisponible".to_string()),
    };
    let meta: SubagentChangeMeta = serde_json::from_str(&data)
        .map_err(|_| "Changement sous-agent indisponible".to_string())?;
    meta.validate()?;
    if meta.child_session_id != child_id {
        return Err("Changement sous-agent indisponible".into());
    }
    Ok(Some(meta))
}

pub async fn save(meta: &SubagentChangeMeta) -> Result<(), String> {
    save_in_dir(meta, &changes_dir(), MAX_STORED_CHANGES).await
}

fn changes_dir() -> PathBuf {
    crate::services::paths::data_dir().join("subagent-changes")
}

async fn save_in_dir(
    meta: &SubagentChangeMeta,
    dir: &std::path::Path,
    limit: usize,
) -> Result<(), String> {
    let _guard = STORE_LOCK.lock().await;
    meta.validate()?;
    let target = dir.join(format!("{}.json", meta.child_session_id));
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())?;
    make_room(dir, &target, limit).await?;
    let tmp = dir.join(format!(".{}.{}.tmp", meta.child_session_id, uuid::Uuid::new_v4()));
    let data = serde_json::to_vec_pretty(meta)
        .map_err(|_| "Persistance du changement impossible".to_string())?;
    tokio::fs::write(&tmp, data)
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())?;
    tokio::fs::rename(&tmp, target)
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())
}

async fn make_room(dir: &std::path::Path, target: &std::path::Path, limit: usize) -> Result<(), String> {
    if target.exists() {
        return Ok(());
    }
    if limit == 0 {
        return Err("Limite de changements atteinte".into());
    }
    let mut entries = tokio::fs::read_dir(dir)
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())?;
    let mut count = 0usize;
    let mut terminal = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        count = count.saturating_add(1);
        if count > limit.saturating_mul(2) {
            return Err("Limite de changements atteinte".into());
        }
        if let Some(updated_at) = terminal_updated_at(&entry.path()).await {
            terminal.push((updated_at, entry.path()));
        }
    }
    let required = count.saturating_add(1).saturating_sub(limit);
    if required == 0 {
        return Ok(());
    }
    terminal.sort_by_key(|(updated_at, _)| *updated_at);
    if terminal.len() < required {
        return Err("Limite de changements atteinte".into());
    }
    for (_, path) in terminal.into_iter().take(required) {
        tokio::fs::remove_file(path)
            .await
            .map_err(|_| "Persistance du changement impossible".to_string())?;
    }
    Ok(())
}

async fn terminal_updated_at(path: &std::path::Path) -> Option<chrono::DateTime<chrono::Utc>> {
    let metadata = tokio::fs::metadata(path).await.ok()?;
    if metadata.len() > MAX_CHANGE_FILE_BYTES {
        return None;
    }
    let data = tokio::fs::read(path).await.ok()?;
    let meta = serde_json::from_slice::<SubagentChangeMeta>(&data).ok()?;
    meta.validate().ok()?;
    matches!(meta.status, SubagentChangeStatus::Applied | SubagentChangeStatus::Discarded)
        .then_some(meta.updated_at)
}

#[cfg(test)]
pub async fn save_in_dir_for_test(
    meta: &SubagentChangeMeta,
    dir: &std::path::Path,
    limit: usize,
) -> Result<(), String> {
    save_in_dir(meta, dir, limit).await
}

pub async fn remove(child_id: &str) -> Result<(), String> {
    match tokio::fs::remove_file(path(child_id)?).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Suppression du changement impossible".into()),
    }
}

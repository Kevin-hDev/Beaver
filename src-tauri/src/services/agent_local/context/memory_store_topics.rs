use super::super::memory_paths::MemoryScope;
use super::super::memory_types::MAX_TOPIC_BYTES;
use super::{
    validate_topic_target, write_topic_locked, MemoryEditError, MemoryWriteError, MEMORY_WRITE_LOCK,
};
use std::path::Path;

pub struct ArchiveTopicResult {
    pub changed: Vec<String>,
    pub content: String,
    pub index_updated: bool,
}

pub async fn read_topic(
    scope: &MemoryScope,
    path: &Path,
) -> Result<(super::memory_format::ParsedTopic, String), MemoryEditError> {
    if !scope.root.exists() {
        return Err(MemoryEditError::NotFound);
    }
    validate_topic_target(scope, path)
        .map_err(MemoryWriteError::TargetInvalid)
        .map_err(MemoryEditError::Failed)?;
    let content = read_existing(path).await?;
    let parsed = super::super::memory_format::parse(&content, path, super::scope_kind(scope))
        .map_err(MemoryWriteError::ContentInvalid)
        .map_err(MemoryEditError::Failed)?;
    Ok((parsed, content))
}

pub async fn edit_topic(
    scope: &MemoryScope,
    path: &Path,
    old: &str,
    new: &str,
) -> Result<Vec<String>, MemoryEditError> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    let (_, current) = read_topic(scope, path).await?;
    if current.matches(old).count() != 1 {
        return Err(MemoryEditError::Stale);
    }
    write_topic_locked(scope, path, &current.replacen(old, new, 1))
        .await
        .map_err(MemoryEditError::Failed)
}

pub async fn replace_topic(
    scope: &MemoryScope,
    path: &Path,
    expected_updated_at: &str,
    content: &str,
) -> Result<Vec<String>, MemoryEditError> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    let (current, _) = read_topic(scope, path).await?;
    if current.topic.updated_at != expected_updated_at {
        return Err(MemoryEditError::Stale);
    }
    write_topic_locked(scope, path, content)
        .await
        .map_err(MemoryEditError::Failed)
}

pub async fn archive_topic(
    scope: &MemoryScope,
    path: &Path,
) -> Result<Vec<String>, MemoryWriteError> {
    archive_topic_result(scope, path)
        .await
        .map(|result| result.changed)
}

pub async fn archive_topic_result(
    scope: &MemoryScope,
    path: &Path,
) -> Result<ArchiveTopicResult, MemoryWriteError> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    validate_topic_target(scope, path).map_err(MemoryWriteError::TargetInvalid)?;
    let current = read_existing(path).await.map_err(|error| match error {
        MemoryEditError::Failed(error) => error,
        MemoryEditError::NotFound | MemoryEditError::Stale => {
            MemoryWriteError::SourceUnavailable("Sujet mémoire introuvable.".into())
        }
    })?;
    let archived = super::super::memory_format_update::archive(&current)
        .map_err(MemoryWriteError::ContentInvalid)?;
    match write_topic_locked(scope, path, &archived).await {
        Ok(changed) => Ok(ArchiveTopicResult {
            changed,
            content: archived,
            index_updated: true,
        }),
        Err(MemoryWriteError::AppliedButIndexFailed(_)) => Ok(ArchiveTopicResult {
            changed: Vec::new(),
            content: archived,
            index_updated: false,
        }),
        Err(error) => Err(error),
    }
}

async fn read_existing(path: &Path) -> Result<String, MemoryEditError> {
    match tokio::fs::try_exists(path).await {
        Ok(true) => {}
        Ok(false) => return Err(MemoryEditError::NotFound),
        Err(error) => {
            return Err(MemoryEditError::Failed(
                MemoryWriteError::SourceUnavailable(super::super::memory_io::storage_error(
                    "topic existence check",
                    error,
                )),
            ));
        }
    }
    super::super::memory_io::read_bounded(path, MAX_TOPIC_BYTES as u64)
        .await
        .map_err(MemoryWriteError::SourceUnavailable)
        .map_err(MemoryEditError::Failed)
}

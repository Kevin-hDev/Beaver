use super::memory_format::{self, ParsedTopic};
use super::memory_paths::MemoryScope;
use super::memory_types::MAX_TOPICS_PER_SCOPE;
use std::path::Path;
use tokio::sync::Mutex;

pub use super::memory_io::write_if_missing;

static MEMORY_WRITE_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Debug, PartialEq, Eq)]
pub enum MemoryEditError {
    Stale,
    NotFound,
    Failed(MemoryWriteError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MemoryWriteError {
    SetupUnavailable(String),
    TargetInvalid(String),
    LimitReached,
    ContentInvalid(String),
    SourceUnavailable(String),
    StorageFailed(String),
    AppliedButIndexFailed(String),
}

impl MemoryWriteError {
    pub fn message(&self) -> &str {
        match self {
            Self::SetupUnavailable(message)
            | Self::TargetInvalid(message)
            | Self::ContentInvalid(message)
            | Self::SourceUnavailable(message)
            | Self::StorageFailed(message)
            | Self::AppliedButIndexFailed(message) => message,
            Self::LimitReached => "La limite de sujets mémoire est atteinte.",
        }
    }
}

pub async fn write_topic(
    scope: &MemoryScope,
    path: &Path,
    content: &str,
) -> Result<Vec<String>, MemoryWriteError> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    write_topic_locked(scope, path, content).await
}

async fn write_topic_locked(
    scope: &MemoryScope,
    path: &Path,
    content: &str,
) -> Result<Vec<String>, MemoryWriteError> {
    scope
        .ensure()
        .await
        .map_err(MemoryWriteError::SetupUnavailable)?;
    validate_topic_target(scope, path).map_err(MemoryWriteError::TargetInvalid)?;
    if !path.exists() && list_topics(scope).await.len() >= MAX_TOPICS_PER_SCOPE {
        return Err(MemoryWriteError::LimitReached);
    }
    let parsed = memory_format::parse(content, path, scope_kind(scope))
        .map_err(MemoryWriteError::ContentInvalid)?;
    if parsed.topic.status == "archived" {
        return super::memory_archive::store(scope, path, content)
            .await
            .map_err(MemoryWriteError::StorageFailed);
    }
    super::memory_io::write_atomic(path, content.as_bytes())
        .await
        .map_err(MemoryWriteError::StorageFailed)?;
    let mut changed = vec![path.to_string_lossy().into_owned()];
    changed.extend(
        super::memory_index::rebuild(scope)
            .await
            .map_err(MemoryWriteError::AppliedButIndexFailed)?,
    );
    Ok(changed)
}

pub async fn edit_topic(
    scope: &MemoryScope,
    path: &Path,
    old: &str,
    new: &str,
) -> Result<Vec<String>, MemoryEditError> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    validate_topic_target(scope, path)
        .map_err(MemoryWriteError::TargetInvalid)
        .map_err(MemoryEditError::Failed)?;
    match tokio::fs::try_exists(path).await {
        Ok(true) => {}
        Ok(false) => return Err(MemoryEditError::NotFound),
        Err(error) => {
            return Err(MemoryEditError::Failed(
                MemoryWriteError::SourceUnavailable(super::memory_io::storage_error(
                    "topic existence check",
                    error,
                )),
            ))
        }
    }
    let current = super::memory_io::read_bounded(path, 64 * 1024)
        .await
        .map_err(MemoryWriteError::SourceUnavailable)
        .map_err(MemoryEditError::Failed)?;
    if current.matches(old).count() != 1 {
        return Err(MemoryEditError::Stale);
    }
    write_topic_locked(scope, path, &current.replacen(old, new, 1))
        .await
        .map_err(MemoryEditError::Failed)
}

pub async fn archive_topic(
    scope: &MemoryScope,
    path: &Path,
) -> Result<Vec<String>, String> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    validate_topic_target(scope, path)?;
    let current = super::memory_io::read_bounded(path, 64 * 1024).await?;
    let archived = super::memory_format_update::archive(&current)?;
    write_topic_locked(scope, path, &archived)
        .await
        .map_err(|error| error.message().to_string())
}

pub async fn load_summary(scope: &MemoryScope) -> String {
    if !scope.root.exists() {
        return String::new();
    }
    if !scope.summary_path().exists() && super::memory_index::rebuild(scope).await.is_err() {
        return String::new();
    }
    super::memory_io::read_bounded(
        &scope.summary_path(),
        super::memory_types::MAX_SUMMARY_BYTES as u64,
    )
    .await
    .unwrap_or_default()
}

pub async fn list_topics(scope: &MemoryScope) -> Vec<ParsedTopic> {
    let mut output = Vec::new();
    let topics_dir = match super::memory_paths::validate_in_scope(scope, &scope.topics_dir()) {
        Ok(path) => path,
        Err(_) => return output,
    };
    let mut entries = match tokio::fs::read_dir(&topics_dir).await {
        Ok(entries) => entries,
        Err(_) => return output,
    };
    while output.len() < MAX_TOPICS_PER_SCOPE {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            _ => break,
        };
        let path = match super::memory_paths::validate_in_scope(scope, &entry.path()) {
            Ok(path) if path.starts_with(&topics_dir) => path,
            _ => continue,
        };
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let content = match super::memory_io::read_bounded(&path, 64 * 1024).await {
            Ok(content) => content,
            Err(_) => continue,
        };
        if let Ok(parsed) = memory_format::parse(&content, &path, scope_kind(scope)) {
            if parsed.topic.status != "archived" {
                output.push(parsed);
            }
        }
    }
    output
}

fn validate_topic_target(scope: &MemoryScope, path: &Path) -> Result<(), String> {
    let expected_parent = scope
        .topics_dir()
        .canonicalize()
        .map_err(|_| "Chemin du sujet mémoire invalide.".to_string())?;
    let actual_parent = path
        .parent()
        .and_then(|parent| parent.canonicalize().ok());
    if actual_parent.as_deref() != Some(expected_parent.as_path())
        || path.extension().and_then(|value| value.to_str()) != Some("md")
        || path
            .file_stem()
            .and_then(|value| value.to_str())
            .is_none_or(|value| uuid::Uuid::parse_str(value).is_err())
    {
        return Err("Chemin du sujet mémoire invalide.".into());
    }
    if matches!(std::fs::symlink_metadata(path), Ok(metadata) if metadata.file_type().is_symlink()) {
        return Err("Lien symbolique mémoire interdit.".into());
    }
    Ok(())
}

pub fn scope_kind(scope: &MemoryScope) -> &str {
    if scope.id == "global" {
        "global"
    } else {
        "project"
    }
}

#[cfg(test)]
#[path = "memory_store_tests.rs"]
mod tests;

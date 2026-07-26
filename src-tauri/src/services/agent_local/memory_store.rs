use super::memory_format::{self, ParsedTopic};
use super::memory_paths::MemoryScope;
use super::memory_types::MAX_TOPICS_PER_SCOPE;
use std::path::Path;
use tokio::sync::Mutex;

pub use super::memory_io::write_if_missing;

static MEMORY_WRITE_LOCK: Mutex<()> = Mutex::const_new(());

pub async fn write_topic(
    scope: &MemoryScope,
    path: &Path,
    content: &str,
) -> Result<Vec<String>, String> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    write_topic_locked(scope, path, content).await
}

async fn write_topic_locked(
    scope: &MemoryScope,
    path: &Path,
    content: &str,
) -> Result<Vec<String>, String> {
    scope.ensure().await?;
    validate_topic_target(scope, path)?;
    if !path.exists() && list_topics(scope).await.len() >= MAX_TOPICS_PER_SCOPE {
        return Err("La limite de sujets mémoire est atteinte.".into());
    }
    let parsed = memory_format::parse(content, path, scope_kind(scope))?;
    if parsed.topic.status == "archived" {
        return super::memory_archive::store(scope, path, content).await;
    }
    super::memory_io::write_atomic(path, content.as_bytes()).await?;
    let mut changed = vec![path.to_string_lossy().into_owned()];
    changed.extend(super::memory_index::rebuild(scope).await?);
    Ok(changed)
}

pub async fn edit_topic(
    scope: &MemoryScope,
    path: &Path,
    old: &str,
    new: &str,
) -> Result<Vec<String>, String> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    validate_topic_target(scope, path)?;
    let current = super::memory_io::read_bounded(path, 64 * 1024).await?;
    if current.matches(old).count() != 1 {
        return Err("Le sujet mémoire a changé. Relisez-le avant de le modifier.".into());
    }
    write_topic_locked(scope, path, &current.replacen(old, new, 1)).await
}

pub async fn archive_topic(
    scope: &MemoryScope,
    path: &Path,
) -> Result<Vec<String>, String> {
    let _guard = MEMORY_WRITE_LOCK.lock().await;
    validate_topic_target(scope, path)?;
    let current = super::memory_io::read_bounded(path, 64 * 1024).await?;
    let archived = super::memory_format_update::archive(&current)?;
    write_topic_locked(scope, path, &archived).await
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

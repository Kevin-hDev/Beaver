use super::memory_paths::{MemoryLayout, MemoryScope};
use super::memory_types::{MemoryOverview, MemoryScopeOverview};
use std::path::Path;

pub async fn load(working_dir: Option<&Path>) -> MemoryOverview {
    let settings = super::memory_settings::load().await;
    let layout = MemoryLayout::production();
    let global = scope(&layout.global_scope()).await;
    let active_scope = match working_dir {
        Some(path) => layout.project_scope_ready(path).await.ok(),
        None => None,
    };
    let active_id = active_scope.as_ref().map(|scope| scope.id.clone());
    let active_project = match active_scope {
        Some(active) => Some(scope(&active).await),
        None => None,
    };
    let project_labels = super::memory_project_labels::load(&layout).await;
    MemoryOverview {
        settings,
        global,
        active_project,
        other_projects: other_projects(&layout, active_id.as_deref(), &project_labels).await,
        legacy_detected: legacy_detected(layout.root()),
    }
}

async fn scope(memory_scope: &MemoryScope) -> MemoryScopeOverview {
    let mut parsed = super::memory_store::list_topics(memory_scope).await;
    parsed.sort_by(|left, right| right.topic.updated_at.cmp(&left.topic.updated_at));
    let mut total_bytes = 0u64;
    for topic in &parsed {
        if let Ok(metadata) = tokio::fs::metadata(&topic.topic.path).await {
            total_bytes = total_bytes.saturating_add(metadata.len());
        }
    }
    MemoryScopeOverview {
        id: memory_scope.id.clone(),
        label: memory_scope.label.clone(),
        topic_count: parsed.len(),
        total_bytes,
        last_updated: parsed
            .iter()
            .map(|topic| topic.topic.updated_at.clone())
            .max(),
        topics: parsed.into_iter().map(|topic| topic.topic).collect(),
        topics_loaded: true,
    }
}

pub async fn load_project(project_id: &str) -> Result<MemoryScopeOverview, String> {
    if !super::memory_paths::valid_project_id(project_id) {
        return Err("Projet mémoire invalide.".into());
    }
    let layout = MemoryLayout::production();
    let labels = super::memory_project_labels::load(&layout).await;
    let root = layout.root().join("projects").join(project_id);
    let metadata = tokio::fs::symlink_metadata(&root)
        .await
        .map_err(|_| "Projet mémoire inaccessible.".to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Projet mémoire inaccessible.".into());
    }
    let label = labels
        .iter()
        .find(|(id, _)| id == project_id)
        .map(|(_, name)| name.clone())
        .unwrap_or_else(|| project_id.to_string());
    Ok(scope(&MemoryScope {
        id: project_id.to_string(),
        label,
        root,
    })
    .await)
}

async fn other_projects(
    layout: &MemoryLayout,
    active_id: Option<&str>,
    labels: &[(String, String)],
) -> Vec<MemoryScopeOverview> {
    let mut output = Vec::new();
    let mut entries = match tokio::fs::read_dir(layout.root().join("projects")).await {
        Ok(entries) => entries,
        Err(_) => return output,
    };
    while output.len() < 128 {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            _ => break,
        };
        let id = entry.file_name().to_string_lossy().into_owned();
        if Some(id.as_str()) == active_id
            || !super::memory_paths::valid_project_id(&id)
        {
            continue;
        }
        let label = labels
            .iter()
            .find(|(project_id, _)| project_id == &id)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| id.clone());
        if let Some(overview) = scope_metadata(&MemoryScope {
                id: id.clone(),
                label,
                root: entry.path(),
            })
            .await
        {
            output.push(overview);
        }
    }
    output.sort_by(|left, right| left.label.cmp(&right.label));
    output
}

async fn scope_metadata(memory_scope: &MemoryScope) -> Option<MemoryScopeOverview> {
    let topics_dir =
        super::memory_paths::validate_in_scope(memory_scope, &memory_scope.topics_dir()).ok()?;
    let mut entries = match tokio::fs::read_dir(&topics_dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Some(empty_scope(memory_scope));
        }
        Err(_) => return None,
    };
    let mut topic_count = 0usize;
    let mut total_bytes = 0u64;
    let mut last_updated = None;
    while topic_count < super::memory_types::MAX_TOPICS_PER_SCOPE {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(_) => return None,
        };
        if entry.path().extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let metadata = match tokio::fs::symlink_metadata(entry.path()).await {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => metadata,
            _ => continue,
        };
        topic_count += 1;
        total_bytes = total_bytes.saturating_add(metadata.len());
        if let Ok(modified) = metadata.modified() {
            let timestamp = chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339();
            if last_updated.as_ref().is_none_or(|current| &timestamp > current) {
                last_updated = Some(timestamp);
            }
        }
    }
    Some(MemoryScopeOverview {
        id: memory_scope.id.clone(),
        label: memory_scope.label.clone(),
        topic_count,
        total_bytes,
        last_updated,
        topics: Vec::new(),
        topics_loaded: false,
    })
}

fn empty_scope(memory_scope: &MemoryScope) -> MemoryScopeOverview {
    MemoryScopeOverview {
        id: memory_scope.id.clone(),
        label: memory_scope.label.clone(),
        topic_count: 0,
        total_bytes: 0,
        last_updated: None,
        topics: Vec::new(),
        topics_loaded: false,
    }
}

fn legacy_detected(root: &Path) -> bool {
    ["archive", "episodes", "hypotheses", "knowledge", "procedures"]
        .iter()
        .any(|name| root.join(name).exists())
}

#[cfg(test)]
#[path = "memory_overview_tests.rs"]
mod tests;

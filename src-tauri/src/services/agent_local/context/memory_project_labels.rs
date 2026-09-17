use super::memory_paths::MemoryLayout;
use std::path::Path;

pub async fn load(layout: &MemoryLayout) -> Vec<(String, String)> {
    let projects = super::project_store::list().await.unwrap_or_default();
    let mut labels = Vec::new();
    for project in projects.into_iter().take(128) {
        if let Ok(scope) = layout.project_scope_ready(Path::new(&project.path)).await {
            labels.push((scope.id, project.name.chars().take(80).collect()));
        }
    }
    labels
}

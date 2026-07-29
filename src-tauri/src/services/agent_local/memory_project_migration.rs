use super::memory_path_classification::{classify_memory_path, MemoryArea};
use super::memory_paths::{lexical_path, validate_in_scope, MemoryLayout, MemoryScope};
use super::memory_project_id::project_identity;
use std::io::ErrorKind;
use std::path::Path;
use tokio::sync::Mutex;

const PENDING_MARKER: &str = ".project-folder-migration-v2";
static PROJECT_MIGRATION_LOCK: Mutex<()> = Mutex::const_new(());

pub async fn resolve(
    layout: &MemoryLayout,
    working_dir: &Path,
) -> Result<MemoryScope, String> {
    let identity = project_identity(working_dir)?;
    let scope = MemoryScope {
        id: identity.id.clone(),
        label: identity.label,
        root: layout.root().join("projects").join(&identity.id),
    };
    migrate_legacy_scope(layout, &scope, &identity.legacy_id).await?;
    Ok(scope)
}

pub async fn scope_for_tool_path(
    layout: &MemoryLayout,
    raw_path: &str,
    working_dir: &Path,
) -> Result<Option<MemoryScope>, String> {
    let Some(classification) =
        classify_memory_path(raw_path, Some(working_dir), layout.root())?
    else {
        return Ok(None);
    };
    lexical_path(raw_path, working_dir)?;
    let global = layout.global_scope();
    if classification.area == MemoryArea::Global {
        return Ok(Some(global));
    }
    let active = resolve(layout, working_dir).await?;
    if classification.belongs_exclusively_to(&active.root) {
        Ok(Some(active))
    } else {
        Err("Mémoire d'un autre projet inaccessible.".into())
    }
}

async fn migrate_legacy_scope(
    layout: &MemoryLayout,
    scope: &MemoryScope,
    legacy_id: &str,
) -> Result<(), String> {
    let _guard = PROJECT_MIGRATION_LOCK.lock().await;
    let legacy = MemoryScope {
        id: legacy_id.to_string(),
        label: scope.label.clone(),
        root: layout.root().join("projects").join(legacy_id),
    };
    let legacy_exists = checked_scope_exists(&legacy).await?;
    let current_exists = checked_scope_exists(scope).await?;
    if legacy_exists && current_exists {
        return Err(migration_error("project folder collision"));
    }
    if legacy_exists {
        ensure_pending_marker(&legacy).await?;
        tokio::fs::rename(&legacy.root, &scope.root)
            .await
            .map_err(|error| migration_io_error("project folder rename", error))?;
        return finish_pending_migration(scope).await;
    }
    if current_exists {
        finish_pending_migration(scope).await?;
    }
    Ok(())
}

async fn checked_scope_exists(scope: &MemoryScope) -> Result<bool, String> {
    match tokio::fs::symlink_metadata(&scope.root).await {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            validate_in_scope(scope, &scope.root)?;
            Ok(true)
        }
        Ok(_) => Err(migration_error("project folder is not a safe directory")),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(migration_io_error("project folder metadata", error)),
    }
}

async fn ensure_pending_marker(scope: &MemoryScope) -> Result<(), String> {
    let marker = scope.root.join(PENDING_MARKER);
    match tokio::fs::symlink_metadata(&marker).await {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(migration_error("migration marker is not a safe file")),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            crate::services::private_store::write_new_async(marker, Vec::new())
                .await
                .map_err(|error| migration_error(&error))
        }
        Err(error) => Err(migration_io_error("migration marker metadata", error)),
    }
}

async fn finish_pending_migration(scope: &MemoryScope) -> Result<(), String> {
    let marker = scope.root.join(PENDING_MARKER);
    match tokio::fs::symlink_metadata(&marker).await {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            super::memory_index::rebuild(scope).await?;
            tokio::fs::remove_file(marker)
                .await
                .map_err(|error| migration_io_error("migration marker cleanup", error))
        }
        Ok(_) => Err(migration_error("migration marker is not a safe file")),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(migration_io_error("migration marker metadata", error)),
    }
}

fn migration_io_error(operation: &str, error: std::io::Error) -> String {
    eprintln!("[memory] {operation}: {error}");
    "Mémoire indisponible.".to_string()
}

fn migration_error(detail: &str) -> String {
    eprintln!("[memory] migration: {detail}");
    "Mémoire indisponible.".to_string()
}

#[cfg(test)]
#[path = "memory_project_migration_tests.rs"]
mod tests;

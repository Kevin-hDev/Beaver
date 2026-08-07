use super::memory_paths::MemoryScope;
use std::path::{Path, PathBuf};

pub async fn ensure_scope_dir(scope: &MemoryScope) -> Result<(), String> {
    reject_existing_symlinks(scope)?;
    crate::services::private_store::ensure_private_dir_async(scope.topics_dir())
        .await
        .map_err(|error| {
            ::log::warn!("[memory] scope create: {error}");
            "Mémoire indisponible.".to_string()
        })?;
    canonical_scope_root(scope)?;
    Ok(())
}

pub fn validate_in_scope(
    scope: &MemoryScope,
    candidate: &Path,
) -> Result<PathBuf, String> {
    let root = canonical_scope_root(scope)?;
    let resolved = if candidate.exists() {
        candidate
            .canonicalize()
            .map_err(|_| "Fichier mémoire inaccessible.".to_string())?
    } else {
        let parent = candidate
            .parent()
            .ok_or_else(|| "Chemin mémoire invalide.".to_string())?
            .canonicalize()
            .map_err(|_| "Chemin mémoire inaccessible.".to_string())?;
        parent.join(
            candidate
                .file_name()
                .ok_or_else(|| "Chemin mémoire invalide.".to_string())?,
        )
    };
    if resolved.starts_with(root) {
        Ok(resolved)
    } else {
        Err("Chemin mémoire interdit.".into())
    }
}

fn canonical_scope_root(scope: &MemoryScope) -> Result<PathBuf, String> {
    reject_existing_symlinks(scope)?;
    let layout = layout_root(scope)?;
    let layout = layout
        .canonicalize()
        .map_err(|_| "Mémoire indisponible.".to_string())?;
    let root = scope
        .root
        .canonicalize()
        .map_err(|_| "Mémoire indisponible.".to_string())?;
    if root.starts_with(layout) {
        Ok(root)
    } else {
        Err("Chemin mémoire interdit.".into())
    }
}

fn reject_existing_symlinks(scope: &MemoryScope) -> Result<(), String> {
    let layout = layout_root(scope)?;
    let mut current = scope.root.as_path();
    loop {
        if std::fs::symlink_metadata(current)
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err("Lien symbolique mémoire interdit.".into());
        }
        if current == layout {
            return Ok(());
        }
        current = current
            .parent()
            .ok_or_else(|| "Chemin mémoire invalide.".to_string())?;
    }
}

fn layout_root(scope: &MemoryScope) -> Result<&Path, String> {
    let parent = scope
        .root
        .parent()
        .ok_or_else(|| "Chemin mémoire invalide.".to_string())?;
    if scope.id == "global" {
        Ok(parent)
    } else {
        parent
            .parent()
            .ok_or_else(|| "Chemin mémoire invalide.".to_string())
    }
}

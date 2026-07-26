use std::path::{Component, Path};

use crate::models::file_tree::FileEntry;

// Sync with src/lib/file-tree-hidden.ts
pub const HIDDEN_ENTRIES: &[&str] = &[
    ".git",
    ".DS_Store",
    ".next",
    ".turbo",
    "__pycache__",
    "dist",
    "target",
    "build",
    ".cache",
];

const MAX_PATH_LEN: usize = 4096;
const MAX_ENTRIES: usize = 5000;

pub(crate) fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.len() > MAX_PATH_LEN || path.contains('\0') {
        return Err("Chemin invalide".into());
    }
    if Path::new(path)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err("Chemin invalide".into());
    }
    Ok(())
}

fn extract_extension(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy();
    // Dotfile sans extension réelle : ".env", ".gitignore"
    if name.starts_with('.') && !name[1..].contains('.') {
        return None;
    }
    path.extension().map(|e| e.to_string_lossy().to_lowercase())
}

#[tauri::command]
pub async fn list_directory(
    path: String,
    show_hidden: bool,
    project_root: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    validate_path(&path)?;

    let canonical = std::fs::canonicalize(&path).map_err(|_| "Dossier introuvable".to_string())?;

    if !canonical.is_dir() {
        return Err("Dossier introuvable".into());
    }

    if let Some(ref root) = project_root {
        let canonical_root =
            std::fs::canonicalize(root).map_err(|_| "Dossier introuvable".to_string())?;
        if !canonical.starts_with(&canonical_root) {
            return Err("Chemin invalide".into());
        }
    } else {
        let roots = crate::services::agent_local::security::allowed_read_roots();
        if !roots.iter().any(|r| canonical.starts_with(r)) {
            return Err("Chemin invalide".into());
        }
    }

    let read_dir =
        std::fs::read_dir(&canonical).map_err(|_| "Impossible de lire ce dossier".to_string())?;

    let mut dirs: Vec<FileEntry> = Vec::new();
    let mut files: Vec<FileEntry> = Vec::new();
    let mut count = 0usize;

    for entry_result in read_dir {
        if count >= MAX_ENTRIES {
            break;
        }
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();

        if !show_hidden && HIDDEN_ENTRIES.contains(&name.as_str()) {
            continue;
        }

        let entry_path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = meta.is_dir();
        let extension = if is_dir {
            None
        } else {
            extract_extension(&entry_path)
        };

        let fe = FileEntry {
            name: name.clone(),
            path: entry_path.to_string_lossy().to_string(),
            is_dir,
            extension,
        };

        if is_dir {
            dirs.push(fe);
        } else {
            files.push(fe);
        }
        count += 1;
    }

    dirs.sort_by_key(|a| a.name.to_lowercase());
    files.sort_by_key(|a| a.name.to_lowercase());

    dirs.extend(files);
    Ok(dirs)
}

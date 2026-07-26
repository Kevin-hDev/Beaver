use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub(crate) fn safe_target_path(
    canonical_dest: &Path,
    dest: &Path,
    raw_path: &Path,
) -> Result<PathBuf, String> {
    let target = canonical_dest.join(raw_path);
    let target_canonical = if target.exists() {
        std::fs::canonicalize(&target).map_err(|e| {
            eprintln!("[ollama-extract] canonicalize entry: {e}");
            "ollama-extract-error".to_string()
        })?
    } else {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                eprintln!("[ollama-extract] mkdir: {e}");
                "ollama-extract-error".to_string()
            })?;
        }
        let parent_canonical =
            std::fs::canonicalize(target.parent().unwrap_or(dest)).map_err(|e| {
                eprintln!("[ollama-extract] canonicalize parent: {e}");
                "ollama-extract-error".to_string()
            })?;
        parent_canonical.join(target.file_name().unwrap_or_default())
    };

    if !target_canonical.starts_with(canonical_dest) {
        eprintln!(
            "[ollama-extract] path traversal: {} vs {}",
            raw_path.display(),
            canonical_dest.display()
        );
        return Err("ollama-extract-path-traversal".into());
    }
    Ok(target_canonical)
}

pub(crate) fn unpack_safe_symlink<R: Read>(
    entry: &tar::Entry<R>,
    canonical_dest: &Path,
    target: &Path,
) -> Result<(), String> {
    let link = entry
        .link_name()
        .map_err(|e| {
            eprintln!("[ollama-extract] symlink target: {e}");
            "ollama-extract-error".to_string()
        })?
        .ok_or_else(|| "symlink sans cible — extraction refusée".to_string())?;

    if link.is_absolute() || has_parent_dir(&link) {
        return Err("symlink non sûr — extraction refusée".into());
    }

    let parent = target
        .parent()
        .ok_or_else(|| "symlink parent invalide".to_string())?;
    let resolved = normalize_path(&parent.join(&link));
    if !resolved.starts_with(canonical_dest) {
        return Err("symlink hors dossier — extraction refusée".into());
    }

    if let Ok(meta) = std::fs::symlink_metadata(target) {
        if meta.is_dir() {
            return Err("symlink cible un dossier existant — extraction refusée".into());
        }
        std::fs::remove_file(target).map_err(|e| {
            eprintln!("[ollama-extract] remove existing symlink target: {e}");
            "ollama-extract-error".to_string()
        })?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&link, target).map_err(|e| {
            eprintln!("[ollama-extract] symlink create: {e}");
            "ollama-extract-error".to_string()
        })?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = link;
        Err("symlink non supporté sur cette plateforme".into())
    }
}

pub(crate) fn has_parent_dir(path: &Path) -> bool {
    path.components().any(|c| c == Component::ParentDir)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            _ => out.push(component.as_os_str()),
        }
    }
    out
}

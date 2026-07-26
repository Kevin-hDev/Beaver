use std::path::{Path, PathBuf};

pub fn for_source(source: &Path) -> Option<PathBuf> {
    source
        .parent()
        .map(|p| p.join("wheels"))
        .filter(|p| usable(p))
}

pub fn sync_from_archive_parent(archive: &Path) -> Result<(), String> {
    let Some(parent) = archive.parent() else {
        return Ok(());
    };
    let bundled = parent.join("wheels");
    if !usable(&bundled) {
        return Ok(());
    }

    let dest = super::paths::sidecar_dir().join("wheels");
    let tmp = super::paths::sidecar_dir().join("wheels.tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|_| "SearXNG: wheels indisponibles".to_string())?;

    for entry in
        std::fs::read_dir(&bundled).map_err(|_| "SearXNG: wheels indisponibles".to_string())?
    {
        let entry = entry.map_err(|_| "SearXNG: wheels indisponibles".to_string())?;
        let path = entry.path();
        if path.is_file() && is_allowed_wheelhouse_file(&path) {
            std::fs::copy(&path, tmp.join(entry.file_name()))
                .map_err(|_| "SearXNG: wheels indisponibles".to_string())?;
        }
    }

    let _ = std::fs::remove_dir_all(&dest);
    std::fs::rename(&tmp, &dest).map_err(|_| "SearXNG: wheels indisponibles".to_string())?;
    Ok(())
}

pub fn usable(path: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };
    entries.flatten().any(|entry| is_wheel(&entry.path()))
}

fn is_allowed_wheelhouse_file(path: &Path) -> bool {
    is_wheel(path) || path.file_name().and_then(|n| n.to_str()) == Some(".requirements.sha256")
}

fn is_wheel(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("whl"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheelhouse_requires_at_least_one_wheel() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!usable(dir.path()));
        std::fs::write(dir.path().join("a.whl"), b"wheel").unwrap();
        assert!(usable(dir.path()));
    }
}

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const MAX_CACHE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const TARGET_CACHE_BYTES: u64 = 3 * 1024 * 1024 * 1024;
const MAX_CACHE_FILES: usize = 20_000;
const MAX_CACHE_ENTRIES: usize = 25_000;
const MAX_CACHE_DEPTH: usize = 16;

struct CacheFile {
    path: PathBuf,
    bytes: u64,
    modified: u64,
}

pub(super) fn prepare(sidecar_dir: &Path) -> Result<PathBuf, String> {
    let cache = sidecar_dir.join(".runtime-cache").join("pip");
    reject_symlink(&cache)?;
    std::fs::create_dir_all(&cache)
        .map_err(|_| "Préparation du cache Forecast impossible".to_string())?;
    prune(&cache)?;
    Ok(cache)
}

fn reject_symlink(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err("Préparation du cache Forecast impossible".to_string())
        }
        Ok(metadata) if !metadata.is_dir() => {
            Err("Préparation du cache Forecast impossible".to_string())
        }
        Ok(_) | Err(_) => Ok(()),
    }
}

fn prune(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    let mut total = 0u64;
    let mut visited = 0usize;
    collect(root, 0, &mut visited, &mut files, &mut total)?;
    if total <= MAX_CACHE_BYTES {
        return Ok(());
    }
    files.sort_by_key(|entry| entry.modified);
    for entry in files {
        if total <= TARGET_CACHE_BYTES {
            break;
        }
        std::fs::remove_file(&entry.path)
            .map_err(|_| "Nettoyage du cache Forecast impossible".to_string())?;
        total = total.saturating_sub(entry.bytes);
    }
    Ok(())
}

fn collect(
    directory: &Path,
    depth: usize,
    visited: &mut usize,
    files: &mut Vec<CacheFile>,
    total: &mut u64,
) -> Result<(), String> {
    if depth > MAX_CACHE_DEPTH {
        return Err("Cache Forecast invalide".to_string());
    }
    let entries = std::fs::read_dir(directory)
        .map_err(|_| "Lecture du cache Forecast impossible".to_string())?;
    for entry in entries {
        *visited = visited.saturating_add(1);
        if *visited > MAX_CACHE_ENTRIES || files.len() >= MAX_CACHE_FILES {
            return Err("Cache Forecast trop volumineux".to_string());
        }
        let entry = entry.map_err(|_| "Lecture du cache Forecast impossible".to_string())?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| "Lecture du cache Forecast impossible".to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("Cache Forecast invalide".to_string());
        }
        if metadata.is_dir() {
            collect(&path, depth + 1, visited, files, total)?;
            continue;
        }
        if !metadata.is_file() {
            return Err("Cache Forecast invalide".to_string());
        }
        *total = total
            .checked_add(metadata.len())
            .ok_or_else(|| "Cache Forecast trop volumineux".to_string())?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |value| value.as_secs());
        files.push(CacheFile {
            path,
            bytes: metadata.len(),
            modified,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::prepare;

    #[test]
    fn creates_an_app_owned_cache() {
        let root = tempfile::tempdir().unwrap();
        let cache = prepare(root.path()).unwrap();
        assert!(cache.is_dir());
        assert!(cache.starts_with(root.path()));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_cache_symlink() {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(".runtime-cache")).unwrap();
        std::os::unix::fs::symlink(
            target.path(),
            root.path().join(".runtime-cache").join("pip"),
        )
        .unwrap();
        assert!(prepare(root.path()).is_err());
    }
}

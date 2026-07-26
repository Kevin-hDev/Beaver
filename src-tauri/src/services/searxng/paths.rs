use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tauri::Manager;

pub fn sidecar_dir() -> PathBuf {
    crate::services::paths::data_dir().join("searxng-sidecar")
}

pub fn venv_dir() -> PathBuf {
    sidecar_dir().join(".venv")
}

pub fn settings_path() -> PathBuf {
    sidecar_dir().join("settings.yml")
}

pub fn source_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if let Some(source) = bundled_source_dir(app) {
        return Ok(source);
    }
    let archive = source_archive(app)?;
    extract_source_archive(&archive)
}

fn bundled_source_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    let resource_source = app
        .path()
        .resource_dir()
        .ok()
        .map(|dir| dir.join("resources").join("searxng-sidecar").join("source"));
    resource_source
        .into_iter()
        .chain(std::iter::once(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("searxng-sidecar")
                .join("source"),
        ))
        .find(|source| valid_source(source))
}

fn source_archive(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let resource_archive = app.path().resource_dir().ok().map(|dir| {
        dir.join("resources")
            .join("searxng-sidecar")
            .join("source.tar.gz")
    });
    resource_archive
        .into_iter()
        .chain(std::iter::once(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("searxng-sidecar")
                .join("source.tar.gz"),
        ))
        .find(|archive| archive.exists())
        .ok_or_else(|| "SearXNG: source introuvable".to_string())
}

fn extract_source_archive(archive: &Path) -> Result<PathBuf, String> {
    let final_dir = sidecar_dir().join("source");
    let stamp = archive_hash(archive)?;
    let stamp_path = sidecar_dir().join(".source-archive.sha256");
    let installed = std::fs::read_to_string(&stamp_path).unwrap_or_default();
    if installed == stamp
        && valid_source(&final_dir)
        && super::source_filter::is_clean_source(&final_dir)
    {
        super::wheels::sync_from_archive_parent(archive)?;
        return Ok(final_dir);
    }
    let tmp_dir = sidecar_dir().join("source.tmp");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("SearXNG: extraction impossible ({e})"))?;

    let file =
        std::fs::File::open(archive).map_err(|_| "SearXNG: source introuvable".to_string())?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut tar_archive = tar::Archive::new(decoder);
    for entry in tar_archive
        .entries()
        .map_err(|_| "SearXNG: archive invalide".to_string())?
    {
        let mut entry = entry.map_err(|_| "SearXNG: archive invalide".to_string())?;
        if should_skip_entry(entry.header().entry_type()) {
            continue;
        }
        let path = safe_archive_path(&entry)?;
        if super::source_filter::should_skip_archive_path(&path) {
            continue;
        }
        // The tar crate's Entry::unpack does NOT create missing parent
        // directories. Some archives (including the bundled SearXNG one) omit
        // top-level directory entries and start directly with files like
        // `source/.coveragerc`, so we must materialize the parent ourselves
        // or unpack fails with ENOENT.
        let dst = tmp_dir.join(&path);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("SearXNG: extraction impossible ({e})"))?;
        }
        entry
            .unpack(&dst)
            .map_err(|e| format!("SearXNG: extraction impossible ({e})"))?;
    }

    let extracted = tmp_dir.join("source");
    if !valid_source(&extracted) {
        return Err("SearXNG: bundle incomplet".to_string());
    }
    let _ = std::fs::remove_dir_all(&final_dir);
    std::fs::rename(&extracted, &final_dir)
        .map_err(|e| format!("SearXNG: extraction impossible ({e})"))?;
    std::fs::write(stamp_path, stamp)
        .map_err(|e| format!("SearXNG: extraction impossible ({e})"))?;
    let _ = std::fs::remove_dir_all(&tmp_dir);
    super::wheels::sync_from_archive_parent(archive)?;
    Ok(final_dir)
}

fn safe_archive_path<R: std::io::Read>(entry: &tar::Entry<'_, R>) -> Result<PathBuf, String> {
    let path = entry
        .path()
        .map_err(|_| "SearXNG: archive invalide".to_string())?;
    if path
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)))
    {
        return Ok(path.into_owned());
    }
    Err("SearXNG: archive invalide".to_string())
}

fn archive_hash(path: &Path) -> Result<String, String> {
    let body = std::fs::read(path).map_err(|_| "SearXNG: source introuvable".to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(body);
    Ok(hex::encode(hasher.finalize()))
}

fn valid_source(source: &Path) -> bool {
    source.join("setup.py").exists()
        && source.join("requirements.txt").exists()
        && source.join("LICENSE").exists()
        && source.join("searx").join("webapp.py").exists()
}

fn should_skip_entry(entry_type: tar::EntryType) -> bool {
    entry_type.is_symlink() || entry_type.is_hard_link()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_archive_contains_required_source_files() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("searxng-sidecar")
            .join("source.tar.gz");
        let file = std::fs::File::open(path).unwrap();
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        let names: Vec<String> = archive
            .entries()
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.path().ok().map(|p| p.to_string_lossy().to_string()))
            .collect();

        assert!(names.iter().any(|name| name == "source/LICENSE"));
        assert!(names.iter().any(|name| name == "source/requirements.txt"));
        assert!(names.iter().any(|name| name == "source/searx/webapp.py"));
    }

    #[test]
    fn skips_links_from_upstream_archive() {
        assert!(should_skip_entry(tar::EntryType::Symlink));
        assert!(should_skip_entry(tar::EntryType::Link));
        assert!(!should_skip_entry(tar::EntryType::Regular));
    }
}

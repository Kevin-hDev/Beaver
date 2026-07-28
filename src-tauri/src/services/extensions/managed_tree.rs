use std::path::{Path, PathBuf};

pub(super) const MAX_ENTRIES: usize = 50_000;
const MAX_DEPTH: usize = 64;
pub(super) const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
pub(super) const MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;

pub fn validate(root: &Path) -> Result<(), String> {
    let root = root
        .canonicalize()
        .map_err(|_| "Installation d'extension invalide.".to_string())?;
    let mut pending = vec![(root, 0_usize)];
    let mut entries = 0_usize;
    let mut total_bytes = 0_u64;
    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DEPTH {
            return Err("Installation d'extension trop profonde.".to_string());
        }
        let children = std::fs::read_dir(directory)
            .map_err(|_| "Installation d'extension illisible.".to_string())?;
        for child in children {
            entries = entries
                .checked_add(1)
                .filter(|count| *count <= MAX_ENTRIES)
                .ok_or_else(|| "Installation d'extension trop volumineuse.".to_string())?;
            let path = child
                .map_err(|_| "Installation d'extension illisible.".to_string())?
                .path();
            inspect_entry(&path, depth, &mut total_bytes, &mut pending)?;
        }
    }
    Ok(())
}

fn inspect_entry(
    path: &Path,
    depth: usize,
    total_bytes: &mut u64,
    pending: &mut Vec<(PathBuf, usize)>,
) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Installation d'extension illisible.".to_string())?;
    let kind = metadata.file_type();
    if kind.is_symlink() {
        return Err("Lien symbolique d'extension non pris en charge.".to_string());
    }
    if kind.is_dir() {
        if pending.len() >= MAX_ENTRIES {
            return Err("Installation d'extension trop volumineuse.".to_string());
        }
        pending.push((path.to_path_buf(), depth + 1));
        return Ok(());
    }
    if !kind.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err("Fichier d'extension invalide.".to_string());
    }
    *total_bytes = total_bytes
        .checked_add(metadata.len())
        .filter(|size| *size <= MAX_TOTAL_BYTES)
        .ok_or_else(|| "Installation d'extension trop volumineuse.".to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;

    #[test]
    fn accepts_a_small_regular_tree() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("lib")).unwrap();
        std::fs::write(directory.path().join("lib/index.js"), "export default {}").unwrap();

        assert!(validate(directory.path()).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symbolic_links() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        symlink("/tmp", directory.path().join("outside")).unwrap();

        assert!(validate(directory.path()).is_err());
    }
}

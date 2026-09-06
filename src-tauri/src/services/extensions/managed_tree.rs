use std::path::{Path, PathBuf};

use super::OperationFailure;

pub(super) const MAX_ENTRIES: usize = 50_000;
const MAX_DEPTH: usize = 64;
pub(super) const MAX_TOTAL_BYTES: u64 = super::install_jobs::DEFAULT_STORAGE_BYTES;

#[cfg(test)]
pub fn validate(root: &Path) -> Result<(), OperationFailure> {
    measure(root).map(|_| ())
}

#[cfg(test)]
pub(super) fn measure(root: &Path) -> Result<u64, OperationFailure> {
    measure_with_budget(root, MAX_TOTAL_BYTES)
}

pub(super) fn measure_with_budget(root: &Path, budget: u64) -> Result<u64, OperationFailure> {
    let root = root
        .canonicalize()
        .map_err(|_| OperationFailure::ManifestInvalid)?;
    let mut pending = vec![(root, 0_usize)];
    let mut entries = 0_usize;
    let mut total_bytes = 0_u64;
    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DEPTH {
            return Err(OperationFailure::ManifestInvalid);
        }
        let children =
            std::fs::read_dir(directory).map_err(|_| OperationFailure::ManifestInvalid)?;
        for child in children {
            entries = entries
                .checked_add(1)
                .filter(|count| *count <= MAX_ENTRIES)
                .ok_or(OperationFailure::ManifestInvalid)?;
            let path = child.map_err(|_| OperationFailure::ManifestInvalid)?.path();
            inspect_entry(&path, depth, &mut total_bytes, &mut pending, budget)?;
        }
    }
    Ok(total_bytes)
}

fn inspect_entry(
    path: &Path,
    depth: usize,
    total_bytes: &mut u64,
    pending: &mut Vec<(PathBuf, usize)>,
    budget: u64,
) -> Result<(), OperationFailure> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| OperationFailure::ManifestInvalid)?;
    let kind = metadata.file_type();
    if kind.is_symlink() {
        return Err(OperationFailure::SymlinkUnsupported);
    }
    if kind.is_dir() {
        if pending.len() >= MAX_ENTRIES {
            return Err(OperationFailure::ManifestInvalid);
        }
        pending.push((path.to_path_buf(), depth + 1));
        return Ok(());
    }
    // The consented total budget also bounds each file: a hidden 256 MiB cap
    // would reject an otherwise approved large extension. Structural limits remain.
    if !kind.is_file() || metadata.len() > budget {
        return Err(OperationFailure::ManifestInvalid);
    }
    *total_bytes = total_bytes
        .checked_add(metadata.len())
        .filter(|size| *size <= budget)
        .ok_or(OperationFailure::ManifestInvalid)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;
    #[cfg(unix)]
    use super::OperationFailure;

    #[test]
    fn a_large_file_uses_the_approved_total_budget_without_a_hidden_cap() {
        let directory = tempfile::tempdir().unwrap();
        let bytes = 256 * 1024 * 1024 + 1;
        std::fs::File::create(directory.path().join("large"))
            .unwrap()
            .set_len(bytes)
            .unwrap();
        assert_eq!(
            super::measure_with_budget(directory.path(), bytes).unwrap(),
            bytes
        );
        assert!(super::measure_with_budget(directory.path(), bytes - 1).is_err());
    }

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

        assert_eq!(
            validate(directory.path()),
            Err(OperationFailure::SymlinkUnsupported)
        );
    }
}

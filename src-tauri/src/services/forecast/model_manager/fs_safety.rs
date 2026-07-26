use std::io;
use std::path::{Path, PathBuf};

const MAX_WALK_ENTRIES: usize = 20_000;
const MAX_WALK_DEPTH: u8 = 16;
const MAX_WALK_BYTES: u64 = 64 * 1024 * 1024 * 1024;

pub(super) fn is_real_directory(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
}

pub(super) fn is_regular_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

pub(super) async fn remove_path(path: &Path) -> io::Result<()> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        tokio::fs::remove_dir_all(path).await
    } else {
        tokio::fs::remove_file(path).await
    }
}

pub(super) fn bounded_directory_size(root: &Path) -> Option<u64> {
    if !is_real_directory(root) {
        return None;
    }
    let mut stack: Vec<(PathBuf, u8)> = vec![(root.to_path_buf(), 0)];
    let mut entries_seen = 0usize;
    let mut total = 0u64;
    while let Some((directory, depth)) = stack.pop() {
        let entries = std::fs::read_dir(directory).ok()?;
        for entry in entries {
            let entry = entry.ok()?;
            entries_seen = entries_seen.checked_add(1)?;
            if entries_seen > MAX_WALK_ENTRIES {
                return None;
            }
            let metadata = std::fs::symlink_metadata(entry.path()).ok()?;
            if metadata.file_type().is_symlink() {
                return None;
            }
            if metadata.is_dir() {
                let next_depth = depth.checked_add(1)?;
                if next_depth > MAX_WALK_DEPTH || stack.len() >= MAX_WALK_ENTRIES {
                    return None;
                }
                stack.push((entry.path(), next_depth));
            } else if metadata.is_file() {
                total = total.checked_add(metadata.len())?;
                if total > MAX_WALK_BYTES {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::{bounded_directory_size, remove_path};

    #[test]
    fn directory_size_counts_only_regular_files() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("nested")).unwrap();
        std::fs::write(root.path().join("one"), [1u8; 3]).unwrap();
        std::fs::write(root.path().join("nested/two"), [2u8; 4]).unwrap();
        assert_eq!(bounded_directory_size(root.path()), Some(7));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn symlink_is_removed_without_touching_its_target() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        let link = root.path().join("link");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep"), b"safe").unwrap();
        symlink(&target, &link).unwrap();
        remove_path(&link).await.unwrap();
        assert!(target.join("keep").is_file());
        assert!(!link.exists());
    }
}

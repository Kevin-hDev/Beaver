use std::path::{Component, Path};

const MAX_SCAN_ENTRIES: usize = 20_000;

pub fn is_clean_source(source: &Path) -> bool {
    let mut stack = vec![source.to_path_buf()];
    let mut visited = 0usize;

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return false;
        };
        for entry in entries.flatten() {
            visited += 1;
            if visited > MAX_SCAN_ENTRIES {
                return false;
            }
            let path = entry.path();
            if is_metadata_name(&path) {
                return false;
            }
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    true
}

pub fn should_skip_archive_path(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(name) => is_metadata_str(&name.to_string_lossy()),
        _ => false,
    })
}

fn is_metadata_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(is_metadata_str)
}

fn is_metadata_str(name: &str) -> bool {
    name.starts_with("._") || name == ".DS_Store" || name == ".AppleDouble" || name == ".py"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_macos_metadata_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(is_clean_source(dir.path()));
        std::fs::write(dir.path().join("._random.py"), b"metadata").unwrap();
        assert!(!is_clean_source(dir.path()));
    }

    #[test]
    fn skips_metadata_inside_archive_paths() {
        assert!(should_skip_archive_path(&PathBuf::from(
            "source/searx/answerers/._random.py"
        )));
        assert!(should_skip_archive_path(&PathBuf::from(
            "source/searx/answerers/.py"
        )));
        assert!(!should_skip_archive_path(&PathBuf::from(
            "source/searx/answerers/random.py"
        )));
    }
}

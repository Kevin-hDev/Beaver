use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};

pub(crate) const MAX_ENTRIES: usize = 512;
const EXCLUDED_NAMES: [&str; 2] = ["secrets.enc", "shell-sandboxes"];

#[derive(Default)]
pub struct PrivateDataAccess {
    pub root: Option<PathBuf>,
    pub directories: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
    pub limit_reached: bool,
}

pub fn current() -> PrivateDataAccess {
    collect(&crate::services::paths::data_dir())
}

fn collect(data_dir: &Path) -> PrivateDataAccess {
    let Some(root) = stable_canonical(data_dir).filter(|path| path.is_dir()) else {
        return PrivateDataAccess::default();
    };
    let Ok(entries) = std::fs::read_dir(&root) else {
        return PrivateDataAccess {
            root: Some(root),
            ..PrivateDataAccess::default()
        };
    };
    let mut access = PrivateDataAccess {
        root: Some(root.clone()),
        ..PrivateDataAccess::default()
    };
    let mut candidates = BinaryHeap::with_capacity(MAX_ENTRIES);
    for entry in entries.flatten() {
        let name = entry.file_name();
        if EXCLUDED_NAMES.iter().any(|excluded| name == *excluded) {
            continue;
        }
        let path = entry.path();
        let Ok(metadata) = path.symlink_metadata() else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if candidates.len() < MAX_ENTRIES {
            candidates.push(path);
        } else {
            access.limit_reached = true;
            if candidates.peek().is_some_and(|largest| path < *largest) {
                candidates.pop();
                candidates.push(path);
            }
        }
    }
    for path in candidates.into_sorted_vec() {
        let Ok(metadata) = path.symlink_metadata() else {
            continue;
        };
        let Some(path) = stable_canonical(&path).filter(|path| path.starts_with(&root)) else {
            continue;
        };
        if metadata.is_dir() {
            access.directories.push(path);
        } else if metadata.is_file() {
            access.files.push(path);
        }
    }
    access.directories.sort();
    access.files.sort();
    access
}

fn stable_canonical(path: &Path) -> Option<PathBuf> {
    let first = dunce::canonicalize(path).ok()?;
    let second = dunce::canonicalize(path).ok()?;
    (first == second).then_some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_store_excludes_the_vault_and_sandbox_temporaries() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("config.json"), "{}").expect("config");
        std::fs::write(temp.path().join("secrets.enc"), "encrypted").expect("vault");
        std::fs::create_dir(temp.path().join("agent-sessions")).expect("sessions");
        std::fs::create_dir(temp.path().join("shell-sandboxes")).expect("sandboxes");

        let access = collect(temp.path());

        assert!(access.files.iter().any(|path| path.ends_with("config.json")));
        assert!(access.directories.iter().any(|path| path.ends_with("agent-sessions")));
        assert!(!access.files.iter().any(|path| path.ends_with("secrets.enc")));
        assert!(!access.directories.iter().any(|path| path.ends_with("shell-sandboxes")));
        assert!(!access.limit_reached);
    }

    #[test]
    fn private_store_limit_is_bounded_and_deterministic() {
        let temp = tempfile::tempdir().expect("tempdir");
        for index in (0..=MAX_ENTRIES).rev() {
            std::fs::write(temp.path().join(format!("entry-{index:04}.json")), "{}")
                .expect("entry");
        }

        let access = collect(temp.path());

        assert!(access.limit_reached);
        assert_eq!(access.files.len(), MAX_ENTRIES);
        assert!(access.files.first().is_some_and(|path| path.ends_with("entry-0000.json")));
        assert!(access.files.last().is_some_and(|path| path.ends_with("entry-0511.json")));
    }
}

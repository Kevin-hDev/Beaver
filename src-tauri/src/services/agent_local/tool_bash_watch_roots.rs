use notify::{RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(any(target_os = "linux", test))]
use std::collections::VecDeque;
use std::path::Path;
#[cfg(any(target_os = "linux", test))]
use std::path::PathBuf;

#[cfg(any(target_os = "linux", test))]
const MAX_WATCH_DIRECTORIES: usize = 4_096;

pub fn attach(watcher: &mut RecommendedWatcher, root: &Path) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        let (directories, incomplete) = collect_directories(root);
        for directory in directories {
            watcher
                .watch(&directory, RecursiveMode::NonRecursive)
                .map_err(|_| "Suivi des fichiers indisponible.".to_string())?;
        }
        Ok(incomplete)
    }
    #[cfg(not(target_os = "linux"))]
    {
        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(|_| "Suivi des fichiers indisponible.".to_string())?;
        Ok(false)
    }
}

#[cfg(any(target_os = "linux", test))]
fn collect_directories(root: &Path) -> (Vec<PathBuf>, bool) {
    let mut directories = Vec::new();
    let mut pending = VecDeque::from([root.to_path_buf()]);
    let mut incomplete = false;
    while let Some(directory) = pending.pop_front() {
        if directories.len() >= MAX_WATCH_DIRECTORIES {
            incomplete = true;
            break;
        }
        directories.push(directory.clone());
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                incomplete = true;
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_directory = entry
                .file_type()
                .is_ok_and(|file_type| file_type.is_dir() && !file_type.is_symlink());
            if is_directory && super::tool_bash_change_hub::is_trackable(root, &path) {
                if pending.len().saturating_add(directories.len()) >= MAX_WATCH_DIRECTORIES {
                    incomplete = true;
                    continue;
                }
                pending.push_back(path);
            }
        }
    }
    (directories, incomplete)
}

#[cfg(test)]
mod tests {
    #[test]
    fn skipped_dependency_directories_are_never_selected_for_watching() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("src/nested")).expect("source dirs");
        std::fs::create_dir_all(root.path().join("node_modules/pkg/deep")).expect("deps");

        let (directories, incomplete) = super::collect_directories(root.path());

        assert!(!incomplete);
        assert!(directories.iter().any(|path| path.ends_with("src/nested")));
        assert!(directories
            .iter()
            .all(|path| !path.to_string_lossy().contains("node_modules")));
    }
}

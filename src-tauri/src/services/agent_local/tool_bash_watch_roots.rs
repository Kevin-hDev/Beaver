use notify::{RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(any(target_os = "linux", test))]
use std::collections::VecDeque;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

#[cfg(any(target_os = "linux", test))]
const MAX_WATCH_DIRECTORIES: usize = 4_096;
pub(super) const MAX_WATCH_ROOTS: usize = 64;

static SHARED_WATCHER: LazyLock<Mutex<Option<SharedWatcher>>> =
    LazyLock::new(|| Mutex::new(None));

struct SharedWatcher {
    watcher: RecommendedWatcher,
    roots: BTreeMap<PathBuf, Vec<PathBuf>>,
}

pub fn attach(root: &Path) -> Result<bool, String> {
    let mut shared = SHARED_WATCHER
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if shared.is_none() {
        *shared = Some(SharedWatcher::create()?);
    }
    shared
        .as_mut()
        .ok_or_else(tracking_unavailable)?
        .attach(root)
}

pub fn detach(root: &Path) -> Result<(), String> {
    let mut shared = SHARED_WATCHER
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let Some(shared) = shared.as_mut() else {
        return Ok(());
    };
    shared.detach(root)
}

impl SharedWatcher {
    fn create() -> Result<Self, String> {
        let watcher = notify::recommended_watcher(|result| {
            super::tool_bash_change_hub::handle_notify_event(result);
        })
        .map_err(|_| tracking_unavailable())?;
        Ok(Self {
            watcher,
            roots: BTreeMap::new(),
        })
    }

    fn attach(&mut self, root: &Path) -> Result<bool, String> {
        if self.roots.contains_key(root) {
            return Ok(false);
        }
        if self.roots.len() >= MAX_WATCH_ROOTS {
            return Err(tracking_unavailable());
        }
        let (paths, mode, incomplete) = watch_paths(root);
        let mut attached = Vec::with_capacity(paths.len());
        for path in paths {
            if self.watcher.watch(&path, mode).is_err() {
                self.rollback(&attached);
                return Err(tracking_unavailable());
            }
            attached.push(path);
        }
        self.roots.insert(root.to_path_buf(), attached);
        Ok(incomplete)
    }

    fn detach(&mut self, root: &Path) -> Result<(), String> {
        let Some(paths) = self.roots.remove(root) else {
            return Ok(());
        };
        let mut failed = false;
        for path in paths {
            failed |= self.watcher.unwatch(&path).is_err();
        }
        if failed {
            return Err(tracking_unavailable());
        }
        Ok(())
    }

    fn rollback(&mut self, paths: &[PathBuf]) {
        for path in paths {
            let _ = self.watcher.unwatch(path);
        }
    }
}

#[cfg(target_os = "linux")]
fn watch_paths(root: &Path) -> (Vec<PathBuf>, RecursiveMode, bool) {
    let (paths, incomplete) = collect_directories(root);
    (paths, RecursiveMode::NonRecursive, incomplete)
}

#[cfg(not(target_os = "linux"))]
fn watch_paths(root: &Path) -> (Vec<PathBuf>, RecursiveMode, bool) {
    (vec![root.to_path_buf()], RecursiveMode::Recursive, false)
}

fn tracking_unavailable() -> String {
    "Suivi des fichiers indisponible.".to_string()
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

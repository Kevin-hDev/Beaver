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
        Ok(Self {
            watcher: create_watcher()?,
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

    fn rollback(&mut self, paths: &[PathBuf]) {
        if any_unwatch_failed(paths, |path| self.watcher.unwatch(path).is_err()) {
            let _ = self.rebuild();
        }
    }

    fn detach(&mut self, root: &Path) -> Result<(), String> {
        let (watcher, roots) = (&mut self.watcher, &mut self.roots);
        let Some(failed) = unregister_root(roots, root, |path| watcher.unwatch(path).is_err())
        else {
            return Ok(());
        };
        if failed {
            self.rebuild()?;
        }
        Ok(())
    }

    fn rebuild(&mut self) -> Result<(), String> {
        // Replacing the watcher drops inconsistent OS state after an unwatch failure.
        let mut watcher = create_watcher()?;
        let roots = self.roots.keys().cloned().collect::<Vec<_>>();
        let mut rebuilt = BTreeMap::new();
        for root in roots {
            let (paths, mode, _) = watch_paths(&root);
            for path in &paths {
                watcher
                    .watch(path, mode)
                    .map_err(|_| tracking_unavailable())?;
            }
            rebuilt.insert(root, paths);
        }
        self.watcher = watcher;
        self.roots = rebuilt;
        super::tool_bash_change_hub::mark_all_overflow();
        Ok(())
    }
}

fn create_watcher() -> Result<RecommendedWatcher, String> {
    notify::recommended_watcher(super::tool_bash_change_hub::handle_notify_event)
        .map_err(|_| tracking_unavailable())
}

fn unregister_root(
    roots: &mut BTreeMap<PathBuf, Vec<PathBuf>>,
    root: &Path,
    unwatch: impl FnMut(&Path) -> bool,
) -> Option<bool> {
    let paths = roots.remove(root)?;
    Some(any_unwatch_failed(&paths, unwatch))
}

fn any_unwatch_failed(paths: &[PathBuf], mut failed: impl FnMut(&Path) -> bool) -> bool {
    let mut any_failed = false;
    for path in paths {
        any_failed |= failed(path);
    }
    any_failed
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
    use std::collections::BTreeMap;

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

    #[test]
    fn failed_unwatch_never_retains_a_root_slot() {
        let root = std::path::PathBuf::from("workspace");
        let mut roots = BTreeMap::from([(root.clone(), vec![root.join("deleted")])]);

        let failed = super::unregister_root(&mut roots, &root, |_| true);

        assert_eq!(failed, Some(true));
        assert!(!roots.contains_key(&root));
    }

    #[test]
    fn failed_rebuild_preserves_remaining_root_registry() {
        let directory = tempfile::tempdir().expect("tempdir");
        let missing = directory.path().join("missing");
        let mut shared = super::SharedWatcher::create().expect("watcher");
        shared.roots.insert(missing.clone(), vec![missing.clone()]);

        assert!(shared.rebuild().is_err());
        assert!(shared.roots.contains_key(&missing));
    }
}

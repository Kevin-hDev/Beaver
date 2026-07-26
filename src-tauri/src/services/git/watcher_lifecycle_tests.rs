use super::watcher::{stop_matching_watcher, ActiveWatcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[test]
fn stops_only_the_watcher_for_the_expected_project() {
    let stop = Arc::new(AtomicBool::new(false));
    let mut slot = Some(ActiveWatcher {
        repo_path: PathBuf::from("/project/one"),
        stop: Arc::clone(&stop),
    });

    stop_matching_watcher(&mut slot, Path::new("/project/two"));
    assert!(slot.is_some());
    assert!(!stop.load(Ordering::Relaxed));

    stop_matching_watcher(&mut slot, Path::new("/project/one"));
    assert!(slot.is_none());
    assert!(stop.load(Ordering::Relaxed));
}

use super::ChangeTracker;
use std::time::{Duration, Instant};

fn wait_for_change(tracker: &mut ChangeTracker) -> Vec<super::ToolFileChange> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let changes = tracker.changes();
        if !changes.is_empty() || Instant::now() >= deadline {
            return changes;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

async fn start_tracker(path: &std::path::Path) -> ChangeTracker {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if let Ok(tracker) = ChangeTracker::start(path).await {
            return tracker;
        }
        assert!(Instant::now() < deadline, "watcher did not initialize");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[test]
fn ignores_generated_dependency_directories() {
    let root = std::path::Path::new("/project");

    assert!(!super::super::tool_bash_change_hub::is_trackable(
        root,
        std::path::Path::new("/project/target/debug/app"),
    ));
    assert!(super::super::tool_bash_change_hub::is_trackable(
        root,
        std::path::Path::new("/project/src/main.rs"),
    ));
}

#[tokio::test]
async fn records_created_files_without_scanning_the_workspace() {
    let directory = tempfile::tempdir().expect("tempdir");
    let mut tracker = start_tracker(directory.path()).await;
    let created = directory.path().join("created.txt");

    std::fs::write(&created, "hello").expect("write");
    let changes = wait_for_change(&mut tracker);
    let expected = created.canonicalize().expect("canonicalize");

    assert!(changes.iter().any(|change| change.path == expected.to_string_lossy()));
}

#[tokio::test]
async fn ignores_paths_outside_the_watched_root() {
    let directory = tempfile::tempdir().expect("tempdir");
    let mut tracker = start_tracker(directory.path()).await;
    let outside = tempfile::NamedTempFile::new().expect("outside");

    tracker.record(
        outside.path().to_path_buf(),
        super::ToolFileChangeStatus::Modified,
    );

    assert!(tracker.changes().is_empty());
}

#[tokio::test]
async fn reuses_one_watcher_for_parallel_commands_in_the_same_workspace() {
    let directory = tempfile::tempdir().expect("tempdir");
    let first = start_tracker(directory.path()).await;
    let second = start_tracker(directory.path()).await;

    assert!(std::sync::Arc::ptr_eq(&first.hub, &second.hub));
}

use super::ChangeTracker;
use std::time::{Duration, Instant};

const EVENT_TIMEOUT: Duration = Duration::from_secs(5);
const EVENT_RETRY_INTERVAL: Duration = Duration::from_millis(50);

fn write_until_path_is_tracked(tracker: &mut ChangeTracker, path: &std::path::Path) -> bool {
    let deadline = Instant::now() + EVENT_TIMEOUT;
    let mut attempt = 0_u64;
    while Instant::now() < deadline {
        std::fs::write(path, attempt.to_string()).expect("write tracked file");
        let expected = path.canonicalize().expect("canonical tracked file");
        if tracker
            .changes()
            .iter()
            .any(|change| change.path == expected.to_string_lossy())
        {
            return true;
        }
        attempt = attempt.saturating_add(1);
        std::thread::sleep(EVENT_RETRY_INTERVAL);
    }
    false
}

async fn start_tracker(path: &std::path::Path) -> ChangeTracker {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if let Ok(tracker) = ChangeTracker::start(path).await {
            if tracker.hub.is_some() {
                return tracker;
            }
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

    assert!(write_until_path_is_tracked(&mut tracker, &created));
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

    assert!(std::sync::Arc::ptr_eq(
        first.hub.as_ref().expect("first hub"),
        second.hub.as_ref().expect("second hub"),
    ));
}

#[tokio::test]
async fn directory_baseline_avoids_the_event_settle_delay() {
    let directory = tempfile::tempdir().expect("tempdir");
    let tracker = start_tracker(directory.path()).await;

    assert!(tracker.directory_baseline.is_some());
    assert!(!tracker.requires_event_settle());
}

#[tokio::test]
async fn parallel_workspaces_keep_complete_final_changes_when_watchers_are_busy() {
    let directories = (0..20)
        .map(|_| tempfile::tempdir().expect("tempdir"))
        .collect::<Vec<_>>();
    let starts = directories
        .iter()
        .map(|directory| ChangeTracker::start(directory.path()));
    let trackers = futures_util::future::join_all(starts).await;

    for (index, (directory, tracker)) in directories.iter().zip(trackers).enumerate() {
        let mut tracker = tracker.expect("tracker");
        let created = directory.path().join(format!("created-{index}.txt"));
        std::fs::write(&created, "content").expect("created file");
        let expected = created.canonicalize().expect("canonical file");

        let (changes, incomplete) = tracker.finish_changes();

        assert!(!incomplete);
        assert!(changes
            .iter()
            .any(|change| change.path == expected.to_string_lossy()));
    }
}

use super::collect_final_changes;
use crate::services::agent_local::tool_bash_changes::ChangeTracker;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

async fn collect_with_counter(
    shutdown_cancelled: bool,
    scans: Arc<AtomicUsize>,
) -> (
    Vec<crate::services::agent_local::types_tools::ToolFileChange>,
    bool,
) {
    let root = tempfile::tempdir().expect("workspace");
    let tracker = ChangeTracker::start(root.path()).await.expect("tracker");
    collect_final_changes(tracker, shutdown_cancelled, move |tracker| {
        scans.fetch_add(1, Ordering::SeqCst);
        tracker.finish_changes()
    })
    .await
}

#[tokio::test]
async fn global_shutdown_skips_the_final_scan_and_marks_changes_incomplete() {
    let scans = Arc::new(AtomicUsize::new(0));
    let (_, incomplete) = collect_with_counter(true, Arc::clone(&scans)).await;

    assert_eq!(scans.load(Ordering::SeqCst), 0);
    assert!(incomplete);
}

#[tokio::test]
async fn normal_completion_keeps_the_final_scan() {
    let scans = Arc::new(AtomicUsize::new(0));
    let _ = collect_with_counter(false, Arc::clone(&scans)).await;

    assert_eq!(scans.load(Ordering::SeqCst), 1);
}

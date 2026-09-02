use super::*;
use serde_json::json;
use std::time::{Duration, Instant};

fn extension_work() -> super::super::work_supervision::ExtensionWorkServices {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor())
}

#[tokio::test]
async fn load_tracker_accepts_one_ordered_load_and_clears_it() {
    let tracker = HostLoadTracker::default();

    tracker.arm("com.beaver.first").await.unwrap();
    assert!(tracker.arm("com.beaver.second").await.is_err());
    assert_eq!(tracker.advance("import").await.unwrap(), "com.beaver.first");
    assert_eq!(
        tracker.advance("activate").await.unwrap(),
        "com.beaver.first"
    );
    assert_eq!(
        tracker.advance("register").await.unwrap(),
        "com.beaver.first"
    );
    tracker.clear().await;
    assert!(tracker.arm("com.beaver.second").await.is_ok());
}

#[tokio::test]
async fn load_tracker_rejects_notifications_outside_or_out_of_order() {
    let tracker = HostLoadTracker::default();

    assert!(tracker.advance("import").await.is_err());
    tracker.arm("com.beaver.first").await.unwrap();
    assert!(tracker.advance("activate").await.is_err());
    assert!(tracker.advance("unknown").await.is_err());
}

#[test]
fn reader_admission_precedes_host_process_creation() {
    let source = include_str!("host_process.rs");
    let admission = source
        .find("try_admit_reader")
        .expect("reader admission boundary");
    let spawn = source
        .find("OwnedProcess::spawn_tokio")
        .expect("host process spawn");

    assert!(admission < spawn);
}

#[tokio::test]
async fn reader_timeout_preserves_the_signal_for_a_retry() {
    let (finished_tx, finished_rx) = tokio::sync::oneshot::channel();
    let reader_done = tokio::sync::Mutex::new(Some(finished_rx));

    assert!(!super::wait_reader_done(&reader_done, Instant::now()).await);
    finished_tx.send(()).unwrap();
    assert!(super::wait_reader_done(&reader_done, Instant::now() + Duration::from_secs(1)).await);
    assert!(reader_done.lock().await.is_none());
}

fn bundled_paths() -> HostPaths {
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("extension-host");
    HostPaths {
        node: which::which("node").unwrap().canonicalize().unwrap(),
        script: directory.join("host.mjs"),
        directory,
    }
}

#[tokio::test]
async fn matches_concurrent_out_of_order_responses_by_id() {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("host.mjs");
    std::fs::write(
        &script,
        r#"import readline from "node:readline";
const lines = readline.createInterface({ input: process.stdin });
lines.on("line", (line) => {
  const message = JSON.parse(line);
  setTimeout(() => process.stdout.write(JSON.stringify({
    jsonrpc: "2.0", id: message.id, result: message.params.value
  }) + "\n"), message.params.delay);
});"#,
    )
    .unwrap();
    let paths = HostPaths {
        node: which::which("node").unwrap(),
        script,
        directory: directory.path().to_path_buf(),
    };
    let work = extension_work();
    let host = Arc::new(HostProcess::spawn(&paths, &work).await.unwrap());
    let slow_host = host.clone();
    let fast_host = host.clone();
    let (slow, fast) = tokio::join!(
        slow_host.request("test", json!({"value": "slow", "delay": 50})),
        fast_host.request("test", json!({"value": "fast", "delay": 1})),
    );

    assert_eq!(slow.unwrap(), json!("slow"));
    assert_eq!(fast.unwrap(), json!("fast"));
    assert!(host.kill(super::stop_deadline()).await);
    assert!(host.request("test", json!({})).await.is_err());
}

#[tokio::test]
async fn bundled_extension_host_answers_hello() {
    let work = extension_work();
    let host = HostProcess::spawn(&bundled_paths(), &work).await.unwrap();

    let hello = host.request("host.hello", json!({})).await.unwrap();

    assert_eq!(hello["apiVersion"], "1");
    assert!(hello["nodeVersion"].as_str().is_some());
    assert!(host.kill(super::stop_deadline()).await);
}

#[tokio::test]
async fn closed_reader_admission_refuses_and_reaps_new_host() {
    let work = extension_work();
    work.begin_closing();

    let result = HostProcess::spawn(&bundled_paths(), &work).await;

    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), error_codes::HOST_UNAVAILABLE);
}

#[tokio::test]
async fn repeated_host_restarts_reuse_the_single_reader_slot() {
    let paths = bundled_paths();
    let work = extension_work();

    for _ in 0..16 {
        let host = HostProcess::spawn(&paths, &work)
            .await
            .expect("reader slot must be reusable after kill");
        assert!(host.kill(super::stop_deadline()).await);
    }
}

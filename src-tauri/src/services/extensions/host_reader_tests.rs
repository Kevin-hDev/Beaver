use super::super::host_identity::HostIdentity;
use super::super::types::ExtensionApiLevel;
use super::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

fn extension_work() -> super::super::work_supervision::ExtensionWorkServices {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor())
}

async fn echo_host() -> (Child, SharedWriter, BufReader<ChildStdout>) {
    let mut child = Command::new(which::which("node").unwrap())
        .args(["-e", "process.stdin.pipe(process.stdout)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let writer = Arc::new(tokio::sync::Mutex::new(child.stdin.take().unwrap()));
    let reader = BufReader::new(child.stdout.take().unwrap());
    (child, writer, reader)
}

#[tokio::test]
async fn ignores_response_for_expired_request() {
    let (mut child, writer, _reader) = echo_host().await;
    let pending = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let tracker = super::super::host_load_tracker::HostLoadTracker::default();
    let work = extension_work();
    let message = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": "expired",
        "result": {"late": true}
    }))
    .unwrap();

    assert!(receive(&message, &writer, &pending, &tracker, &work)
        .await
        .is_ok());

    let _ = child.start_kill();
    let _ = child.wait().await;
}

#[tokio::test]
async fn rejects_saturated_core_without_stopping_reader() {
    let (mut child, writer, mut reader) = echo_host().await;
    let pending = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let tracker = super::super::host_load_tracker::HostLoadTracker::default();
    let work = extension_work();
    let admissions = (0..super::super::work_supervision::MAX_EXTENSION_CORE_CALLS)
        .map(|_| work.try_admit_core_call().unwrap())
        .collect::<Vec<_>>();
    let message = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": "busy",
        "method": "sessions.list",
        "params": {}
    }))
    .unwrap();

    assert!(receive(&message, &writer, &pending, &tracker, &work)
        .await
        .is_ok());
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    let response: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["id"], "busy");
    assert_eq!(response["error"]["code"], -32000);
    drop(admissions);

    let _ = child.start_kill();
    let _ = child.wait().await;
}

#[tokio::test]
async fn accepts_ordered_load_notifications_without_responding() {
    let (mut child, writer, mut reader) = echo_host().await;
    let pending = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let tracker = super::super::host_load_tracker::HostLoadTracker::default();
    let work = extension_work();
    tracker.arm("com.beaver.loading").await.unwrap();

    for stage in ["import", "activate", "register"] {
        let message = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "method": "host.load.stage",
            "params": {"stage": stage}
        }))
        .unwrap();
        assert!(receive(&message, &writer, &pending, &tracker, &work)
            .await
            .is_ok());
    }
    let mut response = String::new();
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(20),
        reader.read_line(&mut response)
    )
    .await
    .is_err());

    let _ = child.start_kill();
    let _ = child.wait().await;
}

#[tokio::test]
async fn rejects_load_notification_outside_the_active_order() {
    let (mut child, writer, _reader) = echo_host().await;
    let pending = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let tracker = super::super::host_load_tracker::HostLoadTracker::default();
    let work = extension_work();
    let message = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "method": "host.load.stage",
        "params": {"stage": "activate"}
    }))
    .unwrap();

    assert!(receive(&message, &writer, &pending, &tracker, &work)
        .await
        .is_err());

    let _ = child.start_kill();
    let _ = child.wait().await;
}

#[tokio::test]
async fn revoked_channel_cannot_start_a_new_core_call() {
    let (mut child, writer, mut reader) = echo_host().await;
    let pending = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let tracker = super::super::host_load_tracker::HostLoadTracker::default();
    let work = extension_work();
    let alive = AtomicBool::new(false);
    let channel_cancel = tokio_util::sync::CancellationToken::new();
    channel_cancel.cancel();
    let message = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": "revoked",
        "method": "app.info",
        "params": {}
    }))
    .unwrap();

    let context = HostReaderContext {
        writer: &writer,
        pending: &pending,
        load_tracker: &tracker,
        alive: &alive,
        channel_cancel: &channel_cancel,
    };
    let authority = HostAuthority {
        identity: HostIdentity::ThirdParty("com.example.revoked".to_string()),
        api_level: ExtensionApiLevel::Stable,
    };
    assert!(receive_bound(&message, &context, &work, &authority)
        .await
        .is_err());
    let mut response = String::new();
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(20),
        reader.read_line(&mut response)
    )
    .await
    .is_err());

    let _ = child.start_kill();
    let _ = child.wait().await;
}

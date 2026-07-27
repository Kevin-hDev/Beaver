use super::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

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
    let limit = Arc::new(Semaphore::new(1));
    let message = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": "expired",
        "result": {"late": true}
    }))
    .unwrap();

    assert!(receive(&message, &writer, &pending, &limit).await.is_ok());

    let _ = child.start_kill();
    let _ = child.wait().await;
}

#[tokio::test]
async fn rejects_saturated_core_without_stopping_reader() {
    let (mut child, writer, mut reader) = echo_host().await;
    let pending = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let limit = Arc::new(Semaphore::new(0));
    let message = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": "busy",
        "method": "sessions.list",
        "params": {}
    }))
    .unwrap();

    assert!(receive(&message, &writer, &pending, &limit).await.is_ok());
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    let response: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["id"], "busy");
    assert_eq!(response["error"]["code"], -32000);

    let _ = child.start_kill();
    let _ = child.wait().await;
}

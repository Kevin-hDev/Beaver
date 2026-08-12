use super::*;
use serde_json::json;

fn extension_work() -> super::super::work_supervision::ExtensionWorkServices {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor())
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
    host.kill().await;
    assert!(host.request("test", json!({})).await.is_err());
}

#[tokio::test]
async fn bundled_extension_host_answers_hello() {
    let work = extension_work();
    let host = HostProcess::spawn(&bundled_paths(), &work).await.unwrap();

    let hello = host.request("host.hello", json!({})).await.unwrap();

    assert_eq!(hello["apiVersion"], "1");
    assert!(hello["nodeVersion"].as_str().is_some());
    host.kill().await;
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
        host.kill().await;
    }
}

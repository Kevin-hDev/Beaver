use super::super::host_paths::HostPaths;
use super::*;
use serde_json::json;

fn prepared_runtime_paths() -> HostPaths {
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("extension-host");
    let node = directory
        .join("runtime")
        .join(if cfg!(windows) { "node.exe" } else { "node" });
    HostPaths {
        node,
        script: directory.join("host.mjs"),
        directory,
    }
}

fn standard_ui_specification() -> super::super::protocol::HostExtensionSpec {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("scripts")
        .join("extensions")
        .join("fixtures")
        .join("ui")
        .join("standard-complete");
    let manifest =
        serde_json::from_slice(&std::fs::read(fixture.join("beaver-extension.json")).unwrap())
            .unwrap();
    super::super::protocol::HostExtensionSpec {
        id: "acceptance.standard.complete".to_string(),
        main_path: fixture.join("index.mjs").to_string_lossy().into_owned(),
        manifest,
    }
}

#[tokio::test]
#[ignore = "requires scripts/extensions/prepare-extension-host.mjs --dev"]
async fn prepared_runtime_answers_hello_through_owned_process() {
    let work = test_extension_work();
    let host = HostProcess::spawn(&prepared_runtime_paths(), &work)
        .await
        .unwrap();

    let hello = host.request("host.hello", json!({})).await.unwrap();

    assert_eq!(hello["apiVersion"], "1");
    assert!(hello["nodeVersion"].as_str().is_some());
    assert!(
        host.kill(super::super::runtime_lifecycle::new_stop_deadline())
            .await
    );
}

#[tokio::test]
#[ignore = "requires scripts/extensions/prepare-extension-host.mjs --dev"]
async fn prepared_runtime_loads_a_real_ui_extension() {
    let _marker_lock = super::super::loading_marker::test_lock().await;
    let _ = super::super::loading_marker::discard();
    let work = test_extension_work();
    let host = HostProcess::spawn(&prepared_runtime_paths(), &work)
        .await
        .unwrap();
    let specification = standard_ui_specification();

    let loaded = host.load(&specification, 1).await;
    let stopped = host
        .kill(super::super::runtime_lifecycle::new_stop_deadline())
        .await;
    let marker_discarded = super::super::loading_marker::discard();

    let loaded = loaded.unwrap();
    assert_eq!(loaded["id"], specification.id);
    assert_eq!(loaded["contributions"]["ui"].as_array().unwrap().len(), 4);
    assert!(stopped);
    marker_discarded.unwrap();
}

#[tokio::test]
#[ignore = "requires scripts/extensions/prepare-extension-host.mjs --dev"]
async fn prepared_runtime_restarts_after_confirmed_stop() {
    let paths = prepared_runtime_paths();
    let work = test_extension_work();
    let first = HostProcess::spawn(&paths, &work).await.unwrap();
    first.request("host.hello", json!({})).await.unwrap();
    assert!(
        first
            .kill(super::super::runtime_lifecycle::new_stop_deadline())
            .await
    );

    let second = HostProcess::spawn(&paths, &work).await.unwrap();
    let hello = second.request("host.hello", json!({})).await.unwrap();
    assert_eq!(hello["apiVersion"], "1");
    assert!(
        second
            .kill(super::super::runtime_lifecycle::new_stop_deadline())
            .await
    );
}

#[tokio::test]
#[ignore = "requires scripts/extensions/prepare-extension-host.mjs --dev"]
async fn prepared_runtime_runs_two_isolated_hosts_concurrently() {
    let paths = prepared_runtime_paths();
    let work = test_extension_work();
    let first = HostProcess::spawn(&paths, &work).await.unwrap();
    let second = HostProcess::spawn(&paths, &work).await.unwrap();

    let (first_hello, second_hello) = tokio::join!(
        first.request("host.hello", json!({})),
        second.request("host.hello", json!({})),
    );
    assert_eq!(first_hello.unwrap()["apiVersion"], "1");
    assert_eq!(second_hello.unwrap()["apiVersion"], "1");

    assert!(
        first
            .kill(super::super::runtime_lifecycle::new_stop_deadline())
            .await
    );
    assert!(
        second
            .kill(super::super::runtime_lifecycle::new_stop_deadline())
            .await
    );
}

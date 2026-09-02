use super::*;
use crate::services::extensions::host_paths::HostPaths;
use crate::services::extensions::protocol::HostExtensionSpec;
use crate::services::extensions::types::{ExtensionApiLevel, ExtensionManifest};

fn specification(id: &str) -> HostExtensionSpec {
    HostExtensionSpec {
        id: id.to_string(),
        main_path: format!("/{id}/index.mjs"),
        manifest: ExtensionManifest {
            id: id.to_string(),
            name: id.to_string(),
            version: "1.0.0".to_string(),
            beaver_api: "1".to_string(),
            runtime: "node".to_string(),
            main: Some("index.mjs".to_string()),
            ui: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
    }
}

#[tokio::test]
async fn a_shared_host_failure_discards_its_partial_load_results() {
    let _marker_lock = super::super::loading_marker::test_lock().await;
    let _ = super::super::loading_marker::discard();
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("host.mjs");
    std::fs::write(
        &script,
        r#"import readline from "node:readline";
let loads = 0;
readline.createInterface({ input: process.stdin }).on("line", (line) => {
  const message = JSON.parse(line);
  if (message.method === "host.load" && ++loads === 2) process.exit(23);
  const result = message.method === "host.load"
    ? { id: message.params.extension.id, contributions: { tools: [], events: [] } }
    : {};
  process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id: message.id, result }) + "\n");
});"#,
    )
    .unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work = crate::services::extensions::work_supervision::ExtensionWorkServices::new(
        coordinator.work_supervisor(),
    );
    let process = HostProcess::spawn(
        &HostPaths {
            node: which::which("node").unwrap().canonicalize().unwrap(),
            script,
            directory: directory.path().to_path_buf(),
        },
        &work,
    )
    .await
    .unwrap();
    let mut responses = Vec::new();

    assert!(load_specs(
        &process,
        &[specification("first"), specification("second")],
        &mut responses,
        &super::super::runtime_sync::RecoveryPreflight::Normal,
    )
    .await
    .is_err());
    assert!(responses.is_empty());
    let super::super::loading_marker::MarkerRead::Valid(marker) =
        super::super::loading_marker::read()
    else {
        panic!("interrupted extension marker expected");
    };
    assert_eq!(marker.extension_id, "second");
    assert_eq!(marker.stage, "import");
    super::super::loading_marker::discard().unwrap();
    assert!(
        process
            .kill(crate::services::extensions::runtime_lifecycle::new_stop_deadline())
            .await
    );
}

#[tokio::test]
async fn crashes_attribute_the_real_extension_and_last_host_stage() {
    let _marker_lock = super::super::loading_marker::test_lock().await;
    for (index, expected_stage) in ["import", "activate", "register"].into_iter().enumerate() {
        let _ = super::super::loading_marker::discard();
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("host.mjs");
        std::fs::write(
            &script,
            format!(
                r#"import readline from "node:readline";
readline.createInterface({{ input: process.stdin }}).on("line", (line) => {{
  const message = JSON.parse(line);
  if (message.method !== "host.load") {{
    process.stdout.write(JSON.stringify({{jsonrpc:"2.0", id:message.id, result:{{}}}}) + "\n");
    return;
  }}
  for (const stage of ["import", "activate", "register"]) {{
    process.stdout.write(JSON.stringify({{jsonrpc:"2.0", method:"host.load.stage", params:{{stage}}}}) + "\n");
    if (stage === "{expected_stage}") process.exit(23);
  }}
}});"#
            ),
        )
        .unwrap();
        let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
        let work = crate::services::extensions::work_supervision::ExtensionWorkServices::new(
            coordinator.work_supervisor(),
        );
        let process = HostProcess::spawn(
            &HostPaths {
                node: which::which("node").unwrap().canonicalize().unwrap(),
                script,
                directory: directory.path().to_path_buf(),
            },
            &work,
        )
        .await
        .unwrap();
        let id = format!("com.example.stage{index}");
        let mut responses = Vec::new();

        assert!(load_specs(
            &process,
            &[specification(&id)],
            &mut responses,
            &super::super::runtime_sync::RecoveryPreflight::Normal,
        )
        .await
        .is_err());
        let super::super::loading_marker::MarkerRead::Valid(marker) =
            super::super::loading_marker::read()
        else {
            panic!("marker expected for {id} at {expected_stage}");
        };
        assert_eq!(marker.extension_id, id);
        assert_eq!(marker.stage, expected_stage);
    }
    super::super::loading_marker::discard().unwrap();
}

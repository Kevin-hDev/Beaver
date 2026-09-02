use super::host_identity::HostIdentity;
use super::host_paths::HostPaths;
use super::host_process::HostProcess;
use super::runtime_hosts::RuntimeHosts;
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionKind, ExtensionManifest, ExtensionRecord,
    ExtensionStatus,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

fn record(id: &str, kind: ExtensionKind) -> ExtensionRecord {
    ExtensionRecord {
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
        kind,
        source: "test".to_string(),
        origin: None,
        enabled: true,
        trusted: true,
        show_in_chat: true,
        status: ExtensionStatus::Inactive,
        last_error: None,
        last_activated_at: None,
        contributions: ExtensionContributions::default(),
    }
}

#[test]
fn rust_registry_assigns_one_official_identity_and_distinct_third_party_identities() {
    let first_builtin =
        HostIdentity::from_record(&record("com.beaver.office.word", ExtensionKind::Builtin))
            .unwrap();
    let second_builtin =
        HostIdentity::from_record(&record("com.beaver.office.excel", ExtensionKind::Builtin))
            .unwrap();
    let first_local =
        HostIdentity::from_record(&record("com.example.first", ExtensionKind::Local)).unwrap();
    let second_local =
        HostIdentity::from_record(&record("com.example.second", ExtensionKind::Local)).unwrap();

    assert_eq!(first_builtin, HostIdentity::Official);
    assert_eq!(second_builtin, HostIdentity::Official);
    assert_eq!(
        first_local,
        HostIdentity::ThirdParty("com.example.first".to_string())
    );
    assert_ne!(first_local, second_local);
}

#[test]
fn official_host_reserves_capacity_and_the_thirty_second_third_party_is_refused() {
    let mut records = vec![record("com.beaver.official", ExtensionKind::Builtin)];
    records.extend(
        (0..32).map(|index| record(&format!("com.example.local{index}"), ExtensionKind::Local)),
    );

    let plan = super::runtime_sync::plan_records(records);

    assert_eq!(plan.official.len(), 1);
    assert_eq!(plan.third_party.len(), 31);
    assert_eq!(
        plan.failures.get("com.example.local31").map(String::as_str),
        Some(super::error_codes::LIMIT_REACHED)
    );
}

#[test]
fn the_thirty_second_third_party_is_refused_even_without_an_active_builtin() {
    let records = (0..32)
        .map(|index| {
            record(
                &format!("com.example.only-local{index}"),
                ExtensionKind::Local,
            )
        })
        .collect();

    let plan = super::runtime_sync::plan_records(records);

    assert!(plan.official.is_empty());
    assert_eq!(plan.third_party.len(), 31);
    assert_eq!(
        plan.failures
            .get("com.example.only-local31")
            .map(String::as_str),
        Some(super::error_codes::LIMIT_REACHED)
    );
}

fn extension_work() -> super::work_supervision::ExtensionWorkServices {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor())
}

async fn bind_fixture(
    hosts: &mut RuntimeHosts,
    root: &std::path::Path,
    id: &str,
    behavior: &str,
    work: &super::work_supervision::ExtensionWorkServices,
) -> Arc<HostProcess> {
    bind_fixture_identity(
        hosts,
        root,
        id,
        behavior,
        HostIdentity::ThirdParty(id.to_string()),
        work,
    )
    .await
}

async fn bind_fixture_identity(
    hosts: &mut RuntimeHosts,
    root: &std::path::Path,
    id: &str,
    behavior: &str,
    identity: HostIdentity,
    work: &super::work_supervision::ExtensionWorkServices,
) -> Arc<HostProcess> {
    let directory = root.join(id);
    std::fs::create_dir(&directory).unwrap();
    let script = directory.join("host.mjs");
    std::fs::write(
        &script,
        format!(
            r#"import readline from "node:readline";
const lines = readline.createInterface({{ input: process.stdin }});
lines.on("line", (line) => {{
  const message = JSON.parse(line);
  if ("{behavior}" === "crash") process.exit(23);
  if ("{behavior}" === "block") while (true) {{}}
  process.stdout.write(JSON.stringify({{jsonrpc:"2.0", id:message.id, result:{{id:"{id}"}}}}) + "\n");
}});"#
        ),
    )
    .unwrap();
    let reservation = hosts.reserve(identity.clone()).unwrap();
    let process = Arc::new(
        HostProcess::spawn_bound(
            &HostPaths {
                node: which::which("node").unwrap().canonicalize().unwrap(),
                script,
                directory,
            },
            work,
            identity,
            ExtensionApiLevel::Stable,
            reservation.temporary_directory(),
        )
        .await
        .unwrap(),
    );
    hosts
        .bind(reservation, ExtensionApiLevel::Stable, Arc::clone(&process))
        .unwrap();
    process
}

#[tokio::test]
async fn unconfirmed_stop_keeps_the_current_channel_bound() {
    let workspace = tempfile::tempdir().unwrap();
    let mut hosts = RuntimeHosts::new(workspace.path().join("channels")).unwrap();
    let work = extension_work();
    let identity = HostIdentity::ThirdParty("current".to_string());
    let process = bind_fixture_identity(
        &mut hosts,
        workspace.path(),
        "current",
        "reply",
        identity.clone(),
        &work,
    )
    .await;
    let (_, generation, _) = hosts.snapshot(&identity).unwrap();

    assert!(!hosts.remove_stopped(&identity, generation, false));
    assert!(hosts.snapshot(&identity).is_some());

    assert!(process.kill(super::host_process::stop_deadline()).await);
}

#[tokio::test]
async fn a_stale_generation_never_removes_the_newer_channel() {
    let workspace = tempfile::tempdir().unwrap();
    let mut hosts = RuntimeHosts::new(workspace.path().join("channels")).unwrap();
    let work = extension_work();
    let identity = HostIdentity::ThirdParty("current".to_string());
    let stale = bind_fixture_identity(
        &mut hosts,
        workspace.path(),
        "stale",
        "reply",
        identity.clone(),
        &work,
    )
    .await;
    let (_, stale_generation, _) = hosts.snapshot(&identity).unwrap();
    assert!(stale.kill(super::host_process::stop_deadline()).await);
    assert!(hosts.remove_current(&identity, stale_generation));

    let current = bind_fixture_identity(
        &mut hosts,
        workspace.path(),
        "current",
        "reply",
        identity.clone(),
        &work,
    )
    .await;

    assert!(!hosts.remove_current(&identity, stale_generation));
    assert_eq!(
        current.request("fixture", json!({})).await.unwrap()["id"],
        "current"
    );
    assert!(current.kill(super::host_process::stop_deadline()).await);
}

#[tokio::test]
async fn a_process_exit_in_one_third_party_host_does_not_stop_another() {
    let workspace = tempfile::tempdir().unwrap();
    let mut hosts = RuntimeHosts::new(workspace.path().join("channels")).unwrap();
    let work = extension_work();
    let crashing = bind_fixture(&mut hosts, workspace.path(), "crashing", "crash", &work).await;
    let healthy = bind_fixture(&mut hosts, workspace.path(), "healthy", "reply", &work).await;

    assert!(crashing.request("fixture", json!({})).await.is_err());
    assert_eq!(
        healthy.request("fixture", json!({})).await.unwrap()["id"],
        "healthy"
    );

    assert!(crashing.kill(super::host_process::stop_deadline()).await);
    assert!(healthy.kill(super::host_process::stop_deadline()).await);
}

#[tokio::test]
async fn a_blocked_third_party_event_loop_does_not_delay_another_host() {
    let workspace = tempfile::tempdir().unwrap();
    let mut hosts = RuntimeHosts::new(workspace.path().join("channels")).unwrap();
    let work = extension_work();
    let blocked = bind_fixture(&mut hosts, workspace.path(), "blocked", "block", &work).await;
    let healthy = bind_fixture(&mut hosts, workspace.path(), "responsive", "reply", &work).await;
    let blocked_request = {
        let blocked = Arc::clone(&blocked);
        tokio::spawn(async move { blocked.request("fixture", json!({})).await })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;

    let response = tokio::time::timeout(
        Duration::from_secs(2),
        healthy.request("fixture", json!({})),
    )
    .await
    .expect("healthy host must not wait for the blocked host")
    .unwrap();
    assert_eq!(response["id"], "responsive");

    assert!(blocked.kill(super::host_process::stop_deadline()).await);
    assert!(blocked_request.await.unwrap().is_err());
    assert!(healthy.kill(super::host_process::stop_deadline()).await);
}

#[tokio::test]
async fn targeted_update_or_uninstall_stop_preserves_official_and_other_third_party_hosts() {
    let workspace = tempfile::tempdir().unwrap();
    let mut hosts = RuntimeHosts::new(workspace.path().join("channels")).unwrap();
    let work = extension_work();
    let official = bind_fixture_identity(
        &mut hosts,
        workspace.path(),
        "official",
        "reply",
        HostIdentity::Official,
        &work,
    )
    .await;
    let target = bind_fixture(&mut hosts, workspace.path(), "target", "reply", &work).await;
    let other = bind_fixture(&mut hosts, workspace.path(), "other", "reply", &work).await;
    let runtime = super::runtime::ExtensionRuntime {
        paths: None,
        hosts: tokio::sync::Mutex::new(hosts),
        sync: tokio::sync::Mutex::new(()),
        status: std::sync::RwLock::new(super::types::ExtensionHostStatus::default()),
        auto_restarts: super::runtime_restart::RestartBudget::default(),
        work,
    };

    assert!(
        runtime
            .stop_channel(
                &HostIdentity::ThirdParty("target".to_string()),
                Some(&target),
            )
            .await
    );
    assert!(target.request("fixture", json!({})).await.is_err());
    assert_eq!(
        official.request("fixture", json!({})).await.unwrap()["id"],
        "official"
    );
    assert_eq!(
        other.request("fixture", json!({})).await.unwrap()["id"],
        "other"
    );

    assert!(official.kill(super::host_process::stop_deadline()).await);
    assert!(other.kill(super::host_process::stop_deadline()).await);
}

#[tokio::test]
async fn real_host_table_refuses_the_thirty_second_third_party_without_reader_starvation() {
    let workspace = tempfile::tempdir().unwrap();
    let mut hosts = RuntimeHosts::new(workspace.path().join("channels")).unwrap();
    let work = extension_work();
    let mut processes = Vec::new();
    processes.push(
        bind_fixture_identity(
            &mut hosts,
            workspace.path(),
            "official-capacity",
            "reply",
            HostIdentity::Official,
            &work,
        )
        .await,
    );
    for index in 0..31 {
        let id = format!("local-capacity-{index}");
        processes.push(bind_fixture(&mut hosts, workspace.path(), &id, "reply", &work).await);
    }

    assert_eq!(hosts.len(), super::types::MAX_HOST_PROCESSES);
    assert_eq!(
        hosts
            .reserve(HostIdentity::ThirdParty("local-capacity-31".to_string()))
            .err(),
        Some(super::error_codes::LIMIT_REACHED.to_string())
    );
    assert_eq!(
        processes[1].request("fixture", json!({})).await.unwrap()["id"],
        "local-capacity-0"
    );

    for process in processes {
        assert!(process.kill(super::host_process::stop_deadline()).await);
    }
}

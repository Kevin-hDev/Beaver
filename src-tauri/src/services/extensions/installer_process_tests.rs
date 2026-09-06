use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

fn fixture(script: &str) -> (tempfile::TempDir, std::process::Command) {
    let root = tempfile::tempdir().unwrap();
    let script_path = root.path().join("producer.mjs");
    std::fs::write(&script_path, script).unwrap();
    let mut command =
        std::process::Command::new(which::which("node").unwrap().canonicalize().unwrap());
    command.arg(script_path).current_dir(root.path());
    (root, command)
}

#[test]
fn identity_must_be_persisted_before_node_can_write() {
    let (root, command) = fixture("import fs from 'node:fs';if(!fs.existsSync('identity'))process.exit(2);fs.writeFileSync('result','ok');");
    let output = run(
        command,
        Duration::from_secs(5),
        || false,
        |identity| {
            assert!(crate::services::owned_process::OwnedProcess::process_exists(identity.pid));
            std::fs::write(
                root.path().join("identity"),
                serde_json::to_vec(&identity).unwrap(),
            )
            .map_err(|_| ())
        },
        || Ok(()),
    );
    assert!(output.is_ok(), "{output:?}");
    assert_eq!(
        std::fs::read_to_string(root.path().join("result")).unwrap(),
        "ok"
    );
}

#[test]
fn failed_identity_persistence_never_opens_producer_gate() {
    let (root, command) = fixture("import fs from 'node:fs';fs.writeFileSync('result','unsafe');");
    let stopped = AtomicBool::new(false);
    let output = run(
        command,
        Duration::from_secs(5),
        || false,
        |_| Err(()),
        || {
            stopped.store(true, Ordering::SeqCst);
            Ok(())
        },
    );
    assert!(output.is_err());
    assert!(stopped.load(Ordering::SeqCst));
    assert!(!root.path().join("result").exists());
}

#[tokio::test]
async fn parent_crash_before_identity_closes_gate_without_loading_producer() {
    let (root, command) = fixture("import fs from 'node:fs';fs.writeFileSync('result','unsafe');");
    let mut command = tokio::process::Command::from(gated(command).unwrap());
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let kind = process_tree::ProcessKind::ExtensionInstaller;
    let (mut child, scope) = OwnedProcess::spawn_tokio_scoped_with_owner_pipe(&mut command, kind)
        .await
        .unwrap();
    let pid = child.id().unwrap();
    // Simulate the exact parent-crash effect before identity persistence: pipe EOF.
    drop(child.stdin.take());
    let status = tokio::time::timeout(Duration::from_secs(6), child.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(!status.success());
    assert!(
        process_tree::terminate_tokio_scoped(
            &mut child,
            kind,
            &scope,
            pid,
            Instant::now() + Duration::from_secs(2)
        )
        .await
    );
    assert!(!root.path().join("result").exists());
}

#[test]
fn root_success_still_terminates_descendant_writer_and_pipe_holder() {
    let (root, command) = fixture("import{spawn}from'node:child_process';const child=spawn(process.execPath,['-e',\"setInterval(()=>require('fs').appendFileSync('writes','x'),5)\"],{stdio:'inherit'});child.unref();process.exit(0);");
    let output = run(
        command,
        Duration::from_secs(5),
        || false,
        |_| Ok(()),
        || Ok(()),
    );
    assert!(output.is_ok(), "{output:?}");
    let before = std::fs::read(root.path().join("writes")).unwrap_or_default();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        std::fs::read(root.path().join("writes")).unwrap_or_default(),
        before
    );
}

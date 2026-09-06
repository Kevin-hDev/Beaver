//! Minimal producer isolates the launch barrier from npm, network and UI code.
use super::*;

#[tokio::test]
async fn launch_gate_imports_a_minimal_producer_after_owner_acknowledgement() {
    let root = tempfile::tempdir().unwrap();
    let script = root.path().join("probe.mjs");
    std::fs::write(&script, "process.stdout.write('producer-loaded');").unwrap();
    let mut command = tokio::process::Command::new(which::which("node").unwrap());
    command
        .args(["--eval", GATE, "--"])
        .arg(&script)
        .current_dir(root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let kind = process_tree::ProcessKind::ExtensionInstaller;
    let (mut child, scope) = OwnedProcess::spawn_tokio_scoped(&mut command, kind)
        .await
        .unwrap();
    let pid = child.id().unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    scope.identity(pid).unwrap();
    input.write_all(&[1]).await.unwrap();
    let status = tokio::time::timeout(Duration::from_secs(8), child.wait()).await;
    assert!(
        process_tree::terminate_tokio_scoped(
            &mut child,
            kind,
            &scope,
            pid,
            Instant::now() + STOP_BUDGET
        )
        .await
    );
    drop(input);
    let mut diagnostics = Vec::new();
    tokio::time::timeout(STOP_BUDGET, stderr.take(4096).read_to_end(&mut diagnostics))
        .await
        .unwrap()
        .unwrap();
    assert!(
        status.is_ok_and(|result| result.is_ok_and(|exit| exit.success())),
        "minimal gate failed before producer completion"
    );
    let mut output = [0_u8; 15];
    tokio::time::timeout(STOP_BUDGET, stdout.read_exact(&mut output))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&output, b"producer-loaded");
}

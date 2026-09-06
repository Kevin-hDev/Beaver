//! Minimal producer isolates the launch barrier from npm, network and UI code.
use super::*;

#[tokio::test]
async fn launch_gate_imports_a_minimal_producer_after_owner_acknowledgement() {
    let root = tempfile::tempdir().unwrap();
    let script = root.path().join("probe.mjs");
    std::fs::write(&script, "process.stdout.write('producer-loaded');").unwrap();
    // Temporary CI probe: remove instrumentation once the Windows gate fault is
    // identified. Only fixed markers and an allowlist of error codes are emitted.
    let probe = GATE
        .replace("const { Worker }", "process.on('uncaughtException', () => { console.error('gate:uncaught'); process.exit(1); }); const { Worker }")
        .replace("process.stdin.pause();", "console.error('gate:ack'); process.stdin.pause();")
        .replace("process.stdin.unref();", "console.error('gate:before-unref'); process.stdin.unref(); console.error('gate:after-unref');")
        .replace("const watcher = new Worker", "console.error('gate:before-worker'); const watcher = new Worker")
        .replace("watcher.on('error', abortGroup);", "watcher.on('error', (error) => { const code = ['EBUSY','EINVAL','EBADF','ENOTSUP','ERR_INVALID_FD_TYPE'].includes(error.code) ? error.code : 'other'; console.error('gate:worker-error:' + code); abortGroup(); });")
        .replace("clearTimeout(launchTimeout);", "console.error('gate:ready'); clearTimeout(launchTimeout);");
    let mut command = tokio::process::Command::new(which::which("node").unwrap());
    command
        .args(["--eval", &probe, "--"])
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
    OwnedProcess::identity(pid).unwrap();
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
    let markers: Vec<_> = String::from_utf8_lossy(&diagnostics)
        .lines()
        .filter(|line| {
            matches!(
                *line,
                "gate:uncaught"
                    | "gate:ack"
                    | "gate:before-unref"
                    | "gate:after-unref"
                    | "gate:before-worker"
                    | "gate:ready"
                    | "gate:worker-error:EBUSY"
                    | "gate:worker-error:EINVAL"
                    | "gate:worker-error:EBADF"
                    | "gate:worker-error:ENOTSUP"
                    | "gate:worker-error:ERR_INVALID_FD_TYPE"
                    | "gate:worker-error:other"
            )
        })
        .map(str::to_owned)
        .collect();
    assert!(
        status.is_ok_and(|result| result.is_ok_and(|exit| exit.success())),
        "minimal gate failed: {markers:?}"
    );
    let mut output = [0_u8; 15];
    tokio::time::timeout(STOP_BUDGET, stdout.read_exact(&mut output))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&output, b"producer-loaded");
}

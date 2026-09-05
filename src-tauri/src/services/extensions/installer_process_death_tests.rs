use super::*;

const FIXTURE_ROOT: &str = "BEAVER_INSTALL_PARENT_DEATH_FIXTURE";

#[test]
#[ignore = "subprocess fixture invoked by the parent-death test"]
fn parent_fixture() {
    let Some(root) = std::env::var_os(FIXTURE_ROOT).map(std::path::PathBuf::from) else {
        return;
    };
    let mut command =
        std::process::Command::new(which::which("node").unwrap().canonicalize().unwrap());
    command.arg(root.join("blocked.mjs")).current_dir(&root);
    let result = run(
        command,
        Duration::from_secs(20),
        || false,
        |identity| {
            crate::services::private_store::atomic_write(
                &root.join("identity.json"),
                &serde_json::to_vec(&identity).unwrap(),
            )
            .map_err(|_| ())
        },
        || Ok(()),
    );
    assert!(result.is_ok(), "{result:?}");
}

#[cfg(unix)]
#[tokio::test]
async fn parent_death_stops_blocked_node_and_descendant_without_main_loop_cooperation() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("blocked.mjs"), "import fs from 'node:fs';import{spawn}from'node:child_process';const child=spawn(process.execPath,['-e',\"require('fs').writeFileSync('writer.pid',String(process.pid));setInterval(()=>require('fs').appendFileSync('writes','x'),5)\"],{stdio:'ignore'});child.unref();fs.writeFileSync('blocked',String(process.pid));for(;;){};").unwrap();
    let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--ignored",
            "--exact",
            "services::extensions::installer_process::death_tests::parent_fixture",
            "--nocapture",
        ])
        .env(FIXTURE_ROOT, root.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let kind = process_tree::ProcessKind::ExtensionInstaller;
    let (mut parent, scope) = OwnedProcess::spawn_tokio_scoped(&mut command, kind)
        .await
        .unwrap();
    let parent_pid = parent.id().unwrap();
    let writer_pid = tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let writer = std::fs::read_to_string(root.path().join("writer.pid"))
                .unwrap_or_default()
                .parse::<u32>();
            if root.path().join("blocked").exists()
                && std::fs::read(root.path().join("writes")).is_ok_and(|bytes| !bytes.is_empty())
            {
                if let Ok(pid) = writer {
                    break pid;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    // Kill ONLY the owning application process, not the child installer group.
    parent.start_kill().unwrap();
    parent.wait().await.unwrap();
    let writer_pid = writer_pid.expect("blocked producer and descendant started");
    let identity: crate::services::owned_process::OwnedProcessIdentity =
        serde_json::from_slice(&std::fs::read(root.path().join("identity.json")).unwrap()).unwrap();
    let absent = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if !OwnedProcess::process_exists(identity.pid)
                && !OwnedProcess::process_exists(writer_pid)
            {
                let result = unsafe { libc::kill(-(identity.pid as i32), 0) };
                if result == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    let before = std::fs::read(root.path().join("writes")).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let stable = std::fs::read(root.path().join("writes")).unwrap() == before;
    if absent.is_err() {
        let _ = OwnedProcess::recover_exact(identity, Instant::now() + Duration::from_secs(2));
        let _ = process_tree::confirm_recovered_group_absent(
            identity.pid,
            Instant::now() + Duration::from_secs(2),
        )
        .await;
    }
    assert!(
        process_tree::terminate_tokio_scoped(
            &mut parent,
            kind,
            &scope,
            parent_pid,
            Instant::now() + Duration::from_secs(2)
        )
        .await
    );
    assert!(
        absent.is_ok(),
        "installer group survived its application parent"
    );
    assert!(stable, "descendant kept writing after parent death");
    assert!(root.path().join("identity.json").exists());
}

use serde::{Deserialize, Serialize};

#[path = "stream_recovery_process_emit.rs"]
mod emit;
#[path = "stream_recovery_process_restore.rs"]
mod restore;

const ROLE_ENV: &str = "BEAVER_STREAM_RECOVERY_PROCESS_ROLE";
const MANIFEST_ENV: &str = "BEAVER_STREAM_RECOVERY_PROCESS_MANIFEST";
const TEST_NAME: &str = "services::agent_local::stream_recovery_process_tests::stream_recovery_process_survives_a_real_process_kill";

#[derive(Serialize, Deserialize)]
struct Manifest {
    data_dir: std::path::PathBuf,
    session_id: String,
}

#[test]
fn stream_recovery_process_survives_a_real_process_kill() {
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("emit") => emit::run(),
        Ok("recover") => restore::run(),
        _ => parent(),
    }
}

fn parent() {
    let temp = tempfile::tempdir().expect("manifest directory");
    let manifest_path = temp.path().join("recovery.json");
    let mut emitter = child("emit", &manifest_path);
    let manifest = wait_for_second_call(&manifest_path, &mut emitter);
    emitter.kill().expect("kill emitting process");
    assert!(!emitter.wait().expect("wait emitting process").success());
    let status = child("recover", &manifest_path)
        .wait()
        .expect("wait recovery process");
    assert!(status.success(), "recovery child failed with {status}");
    std::fs::remove_dir_all(manifest.data_dir).expect("remove killed child data");
}

fn child(role: &str, manifest: &std::path::Path) -> std::process::Child {
    std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", TEST_NAME, "--nocapture"])
        .env(ROLE_ENV, role)
        .env(MANIFEST_ENV, manifest)
        .spawn()
        .expect("spawn recovery test child")
}

fn wait_for_second_call(path: &std::path::Path, child: &mut std::process::Child) -> Manifest {
    for _ in 0..200 {
        if let Some(status) = child.try_wait().expect("inspect emitting process") {
            panic!("emitting process exited before journal checkpoint: {status}");
        }
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(manifest) = serde_json::from_slice::<Manifest>(&bytes) {
                let session = manifest
                    .data_dir
                    .join("agent-sessions")
                    .join(format!("{}.json", manifest.session_id));
                let ready = std::fs::read_to_string(session)
                    .is_ok_and(|content| content.contains("call-second"));
                if ready {
                    return manifest;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("second tool call was not journaled");
}

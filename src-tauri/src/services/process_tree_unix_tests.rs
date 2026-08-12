use super::{configure, configure_update_helper};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const MODE_ENV: &str = "BEAVER_PARENT_DEATH_PROBE";
const READY_ENV: &str = "BEAVER_PARENT_DEATH_READY";
const WITNESS_ENV: &str = "BEAVER_PARENT_DEATH_WITNESS";
const PARENT_TEST: &str = "services::process_tree::linux_parent_death_tests::parent_probe";
const CHILD_TEST: &str = "services::process_tree::linux_parent_death_tests::child_probe";

#[test]
fn transferred_helper_survives_real_parent_death() {
    assert!(run_probe("transferred"));
}

#[test]
fn owned_child_dies_with_real_parent() {
    assert!(!run_probe("owned"));
}

#[test]
#[ignore = "subprocess entry point"]
#[expect(
    clippy::zombie_processes,
    reason = "the probe parent must exit without waiting to exercise parent-death behavior"
)]
fn parent_probe() {
    let mode = std::env::var(MODE_ENV).expect("probe mode");
    let ready = required_path(READY_ENV);
    let witness = required_path(WITNESS_ENV);
    let mut command = test_command(CHILD_TEST);
    command.env(READY_ENV, ready).env(WITNESS_ENV, witness);
    match mode.as_str() {
        "transferred" => configure_update_helper(&mut command),
        "owned" => configure(&mut command),
        _ => panic!("invalid probe mode"),
    }
    let _child = command.spawn().expect("probe child");
    assert!(wait_for_path(
        &required_path(READY_ENV),
        Duration::from_secs(5)
    ));
}

#[test]
#[ignore = "subprocess entry point"]
fn child_probe() {
    std::fs::write(required_path(READY_ENV), b"ready").expect("ready marker");
    std::thread::sleep(Duration::from_millis(400));
    std::fs::write(required_path(WITNESS_ENV), b"survived").expect("witness marker");
}

fn run_probe(mode: &str) -> bool {
    let temp = tempfile::tempdir().expect("probe tempdir");
    let ready = temp.path().join("ready");
    let witness = temp.path().join("witness");
    let status = test_command(PARENT_TEST)
        .env(MODE_ENV, mode)
        .env(READY_ENV, &ready)
        .env(WITNESS_ENV, &witness)
        .status()
        .expect("probe parent");
    assert!(status.success());
    wait_for_path(&witness, Duration::from_secs(2))
}

fn test_command(test_name: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args(["--ignored", "--exact", test_name, "--nocapture"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn required_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).expect("probe path"))
}

fn wait_for_path(path: &Path, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    path.exists()
}

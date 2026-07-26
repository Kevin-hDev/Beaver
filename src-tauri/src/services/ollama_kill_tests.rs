use crate::services::ollama_kill::{clear_pid_file, read_saved_pid, save_pid};
use std::sync::Mutex;

static PID_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn pid_file_roundtrip_and_format() {
    let _guard = PID_TEST_LOCK.lock().unwrap();
    clear_pid_file();

    save_pid(54321);
    let pid = read_saved_pid();
    assert_eq!(pid, Some(54321));

    let path = crate::services::paths::data_dir().join("ollama-sidecar.pid");
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(content.contains(':'), "should contain pid:timestamp");
    let parts: Vec<&str> = content.trim().split(':').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "54321");
    let ts: u64 = parts[1].parse().expect("timestamp should be valid u64");
    assert!(ts > 1_700_000_000, "timestamp should be recent");

    assert!(
        !path.with_extension("tmp").exists(),
        "tmp should be cleaned up"
    );

    clear_pid_file();
}

#[test]
fn pid_file_rejects_small_pid() {
    let _guard = PID_TEST_LOCK.lock().unwrap();
    let path = crate::services::paths::data_dir().join("ollama-sidecar.pid");
    let _ = std::fs::write(&path, "1:0");
    assert_eq!(read_saved_pid(), None);
    let _ = std::fs::write(&path, "0:0");
    assert_eq!(read_saved_pid(), None);
    clear_pid_file();
}

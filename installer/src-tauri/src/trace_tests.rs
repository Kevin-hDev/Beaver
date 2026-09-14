use super::trace::{
    ErrorCode, InstallerTrace, OperationId, Outcome, Phase, TraceEntry, MAX_TRACE_BYTES,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static NEXT: AtomicU64 = AtomicU64::new(1);

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "beaver-trace-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn entry() -> TraceEntry {
    TraceEntry {
        operation_id: OperationId::Download,
        phase: Phase::Running,
        outcome: Outcome::Succeeded,
        elapsed_ms: 42,
        error_code: None,
    }
}

#[test]
fn writes_only_the_closed_structured_fields() {
    let root = root();
    let mut trace = InstallerTrace::create_in(&root).unwrap();
    trace.record(entry()).unwrap();
    let text = fs::read_to_string(trace.path()).unwrap();
    let value: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        [
            "elapsed_ms",
            "error_code",
            "operation_id",
            "outcome",
            "phase"
        ]
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(trace.path()).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    drop(trace);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_writes_beyond_the_trace_bound() {
    let root = root();
    let mut trace = InstallerTrace::create_in(&root).unwrap();
    let mut failed = false;
    for elapsed_ms in 0..MAX_TRACE_BYTES {
        let mut value = entry();
        value.elapsed_ms = elapsed_ms as u64;
        if trace.record(value).is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    assert!(fs::metadata(trace.path()).unwrap().len() <= MAX_TRACE_BYTES as u64);
    drop(trace);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preserves_at_most_three_failure_traces() {
    let data = root();
    for index in 0..5_u8 {
        let run = root();
        let mut trace = InstallerTrace::create_in(&run).unwrap();
        trace
            .record(TraceEntry {
                error_code: Some(ErrorCode::InstallFailed),
                outcome: Outcome::Failed,
                ..entry()
            })
            .unwrap();
        let id = format!("{index:032x}");
        trace.preserve_into(&data, &id).unwrap();
        fs::remove_dir_all(run).unwrap();
        std::thread::sleep(Duration::from_millis(2));
    }
    let files: Vec<_> = fs::read_dir(data.join("logs/installer"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(files.len(), 3);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(data.join("logs/installer"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
    fs::remove_dir_all(data).unwrap();
}

#[test]
fn failed_preservation_does_not_block_the_next_trace() {
    let missing_data = root().join("missing");
    let run = root();
    let mut trace = InstallerTrace::create_in(&run).unwrap();
    let path = trace.path().to_path_buf();

    assert!(trace
        .preserve_into(&missing_data, "0123456789abcdef0123456789abcdef")
        .is_err());
    drop(trace);

    assert!(!path.exists());
    drop(InstallerTrace::create_in(&run).unwrap());
    fs::remove_dir_all(run).unwrap();
}

#[cfg(unix)]
#[test]
fn refuses_a_symbolic_trace_directory() {
    use std::os::unix::fs::symlink;

    let data = root();
    let outside = root();
    fs::create_dir(data.join("logs")).unwrap();
    symlink(&outside, data.join("logs/installer")).unwrap();
    let run = root();
    let mut trace = InstallerTrace::create_in(&run).unwrap();
    assert!(trace
        .preserve_into(&data, "0123456789abcdef0123456789abcdef")
        .is_err());
    assert!(outside.read_dir().unwrap().next().is_none());
    fs::remove_dir_all(run).unwrap();
    fs::remove_dir_all(data).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

use super::python_runtime::probe_matches;
#[cfg(unix)]
use super::python_runtime::PythonRuntime;
use super::python_runtime_path::lookup_suffixes;
#[cfg(unix)]
use super::python_runtime_path::{command_for, locate_with_suffixes};
use super::runtime_manifest::RuntimeManifest;
use std::ffi::OsStr;
#[cfg(unix)]
use std::path::Path;
#[cfg(windows)]
use std::time::Duration;

#[test]
fn python_probe_accepts_native_lf_and_crlf_output() {
    let manifest = RuntimeManifest::for_test(3, 14);

    assert!(probe_matches(b"cpython\n3\n14\n", &manifest));
    assert!(probe_matches(b"cpython\r\n3\r\n14\r\n", &manifest));
    assert!(!probe_matches(b"cpython\r3\r14\r", &manifest));
}

#[cfg(unix)]
#[tokio::test]
async fn resolver_executes_only_the_compatible_candidate_from_gui_path() {
    let directory = tempfile::tempdir().expect("temporary PATH");
    let incompatible = directory.path().join("python3.14");
    let compatible = directory.path().join("python3");
    let incompatible_marker = directory.path().join("incompatible-ran");
    let compatible_marker = directory.path().join("compatible-ran");
    write_fake_python(&incompatible, &incompatible_marker, 13, true);
    write_fake_python(&compatible, &compatible_marker, 14, true);

    let manifest = RuntimeManifest::for_test(3, 14);
    let selected = PythonRuntime::resolve_with_path(&manifest, directory.path()).await;

    assert_eq!(selected.expect("GUI PATH runtime").label(), "python3");
    assert!(incompatible_marker.is_file());
    assert!(compatible_marker.is_file());
}

#[cfg(unix)]
#[tokio::test]
async fn non_executable_candidate_does_not_mask_the_next_python() {
    let directory = tempfile::tempdir().expect("temporary PATH");
    let blocked = directory.path().join("python3.14");
    let compatible = directory.path().join("python3");
    let marker = directory.path().join("compatible-ran");
    write_fake_python(&blocked, &directory.path().join("blocked-ran"), 14, false);
    write_fake_python(&compatible, &marker, 14, true);

    let manifest = RuntimeManifest::for_test(3, 14);
    let selected = PythonRuntime::resolve_with_path(&manifest, directory.path()).await;

    assert_eq!(selected.expect("next runtime").label(), "python3");
    assert!(marker.is_file());
}

#[cfg(unix)]
#[tokio::test]
async fn pypy_candidate_does_not_mask_the_compatible_cpython() {
    let directory = tempfile::tempdir().expect("temporary PATH");
    let pypy = directory.path().join("python3.14");
    let cpython = directory.path().join("python3");
    write_fake_python_identity(&pypy, &directory.path().join("pypy-ran"), "pypy", 14, true);
    write_fake_python_identity(
        &cpython,
        &directory.path().join("cpython-ran"),
        "cpython",
        14,
        true,
    );

    let selected =
        PythonRuntime::resolve_with_path(&RuntimeManifest::for_test(3, 14), directory.path())
            .await
            .expect("compatible CPython");

    assert_eq!(selected.label(), "python3");
}

#[test]
fn platform_lookup_suffix_is_the_runtime_authority() {
    #[cfg(windows)]
    assert_eq!(lookup_suffixes(), [OsStr::new(".exe")]);
    #[cfg(not(windows))]
    assert_eq!(lookup_suffixes(), [OsStr::new("")]);
}

#[cfg(windows)]
#[tokio::test]
async fn windows_python_probe_is_confined_by_the_process_authority() {
    let python = crate::services::test_runtime::python().expect("test Python");
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let probe = tokio::spawn(super::python_runtime::run_probe_for_test(
        python,
        vec![
            "-c".into(),
            "import time; time.sleep(0.5); print('ok')".into(),
        ],
        Duration::from_secs(2),
        started_tx,
    ));
    let pid = started_rx.await.expect("probe pid");

    assert!(crate::services::owned_process::OwnedProcess::is_confined_for_test(pid));
    assert_eq!(probe.await.expect("probe task"), Some(b"ok\r\n".to_vec()));
    assert!(wait_until_process_is_gone(pid).await);
}

#[cfg(windows)]
#[tokio::test]
async fn timed_out_windows_python_probe_is_reaped() {
    let python = crate::services::test_runtime::python().expect("test Python");
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let probe = tokio::spawn(super::python_runtime::run_probe_for_test(
        python,
        vec!["-c".into(), "import time; time.sleep(30)".into()],
        Duration::from_millis(50),
        started_tx,
    ));
    let pid = started_rx.await.expect("probe pid");

    assert!(probe.await.expect("probe task").is_none());
    assert!(wait_until_process_is_gone(pid).await);
}

#[cfg(windows)]
async fn wait_until_process_is_gone(pid: u32) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        let mut processes = sysinfo::System::new();
        processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        if processes.process(sysinfo::Pid::from_u32(pid)).is_none() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[cfg(unix)]
#[test]
fn lookup_rejects_a_non_executable_file_directly() {
    let directory = tempfile::tempdir().expect("temporary PATH");
    let blocked = directory.path().join("python3.14");
    std::fs::write(&blocked, b"not executable").expect("blocked Python");

    assert!(
        locate_with_suffixes(Path::new("python3.14"), directory.path(), &[OsStr::new("")],)
            .is_none()
    );
}

#[cfg(unix)]
#[test]
fn lookup_adds_only_the_controlled_windows_executable_suffix() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("temporary PATH");
    let executable = directory.path().join("python3.14.exe");
    std::fs::write(&executable, b"fake executable").expect("fake executable");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))
        .expect("executable fake Python");

    let found = locate_with_suffixes(
        Path::new("python3.14"),
        directory.path(),
        &[OsStr::new(".exe")],
    );

    assert_eq!(found.as_deref(), Some(executable.as_path()));
}

#[cfg(unix)]
#[tokio::test]
async fn common_command_helper_applies_the_injected_gui_path() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("temporary PATH");
    let program = directory.path().join("python3.14");
    let marker = directory.path().join("command-used-gui-path");
    let path = std::env::join_paths([directory.path()]).expect("GUI PATH");
    std::fs::write(
        &program,
        format!(
            "#!/bin/sh\n[ \"$PATH\" = '{}' ] && : > '{}'\n",
            path.to_string_lossy(),
            marker.display()
        ),
    )
    .expect("fake Python");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700))
        .expect("executable fake Python");

    let status = command_for(&program, &path)
        .status()
        .await
        .expect("command");

    assert!(status.success());
    assert!(marker.is_file());
}

#[cfg(unix)]
fn write_fake_python(path: &Path, marker: &Path, minor: u8, executable: bool) {
    write_fake_python_identity(path, marker, "cpython", minor, executable);
}

#[cfg(unix)]
fn write_fake_python_identity(
    path: &Path,
    marker: &Path,
    implementation: &str,
    minor: u8,
    executable: bool,
) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        format!(
            "#!/bin/sh\n: > '{}'\nprintf '{}\\n3\\n{}\\n'\n",
            marker.display(),
            implementation,
            minor
        ),
    )
    .expect("fake Python");
    let mode = if executable { 0o700 } else { 0o600 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .expect("fake Python permissions");
}

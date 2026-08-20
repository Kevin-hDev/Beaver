use super::python_runtime::PythonRuntime;
use super::python_runtime_path::{command_for, locate_with_suffixes};
use super::runtime_manifest::RuntimeManifest;
use std::ffi::OsStr;
use std::path::Path;

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
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        format!(
            "#!/bin/sh\n: > '{}'\nprintf 'cpython\\n3\\n{}\\n'\n",
            marker.display(),
            minor
        ),
    )
    .expect("fake Python");
    let mode = if executable { 0o700 } else { 0o600 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .expect("fake Python permissions");
}

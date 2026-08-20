use super::runtime_environment::RuntimeEnvironment;
use super::runtime_manifest::RuntimeManifest;
#[cfg(unix)]
use crate::app_exit::AppExitCoordinator;
#[cfg(unix)]
use crate::services::work_registry::ServiceWorkSupervisor;

fn layout(root: &std::path::Path) -> super::runtime_environment_fs::Layout {
    super::runtime_environment_fs::Layout::at(root).expect("layout")
}

fn recover_at(root: &std::path::Path) -> Result<(), super::runtime_error::RuntimeError> {
    super::runtime_environment_fs::recover(&layout(root))
}

fn reusable_at(
    root: &std::path::Path,
    manifest: &RuntimeManifest,
    source_hash: &str,
) -> Result<bool, super::runtime_error::RuntimeError> {
    super::runtime_receipt::reusable(&layout(root), manifest, source_hash)
}

fn mark_started_at(root: &std::path::Path) -> Result<(), super::runtime_error::RuntimeError> {
    let layout = layout(root);
    if super::runtime_environment_fs::present_dir(&layout.previous)? {
        super::runtime_environment_fs::remove_dir(&layout, &layout.previous)?;
    }
    Ok(())
}

#[test]
fn interrupted_staging_never_replaces_the_current_runtime() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let current = root.path().join(".venv");
    let staged = root.path().join(".venv.next");
    std::fs::create_dir(&current).expect("current runtime");
    std::fs::write(current.join("current"), b"ready").expect("current marker");
    std::fs::create_dir(&staged).expect("abandoned staging");
    std::fs::write(staged.join("partial"), b"incomplete").expect("staged marker");

    recover_at(root.path()).expect("recover staging");

    assert!(current.join("current").is_file());
    assert!(!staged.exists());
}

#[test]
fn incompatible_legacy_runtime_is_rebuilt_beside_the_old_one() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let current = root.path().join(".venv");
    std::fs::create_dir(&current).expect("legacy runtime");
    std::fs::write(current.join("python"), b"legacy").expect("legacy marker");
    let manifest = RuntimeManifest::for_test(3, 14);

    assert!(
        !reusable_at(root.path(), &manifest, &"a".repeat(64)).expect("legacy receipt is rejected")
    );
    assert_eq!(layout(root.path()).staged, root.path().join(".venv.next"));
    super::runtime_environment_fs::prepare_staging(&layout(root.path()))
        .expect("stage replacement beside legacy");
    assert!(current.join("python").is_file());
    assert!(root.path().join(".venv.next").is_dir());
}

#[test]
fn failed_publication_restores_the_previous_runtime() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let current = root.path().join(".venv");
    let staged = root.path().join(".venv.next");
    std::fs::create_dir(&current).expect("current runtime");
    std::fs::write(current.join("current"), b"old").expect("old marker");
    std::fs::create_dir(&staged).expect("staged runtime");
    std::fs::write(staged.join("staged"), b"new").expect("new marker");

    let result = super::runtime_environment_fs::publish_with(&layout(root.path()), |_, _| {
        Err(super::runtime_error::RuntimeError::EnvironmentUnavailable)
    });

    assert!(result.is_err());
    assert!(current.join("current").is_file());
    assert!(!root.path().join(".venv.previous").exists());
    assert!(staged.join("staged").is_file());
}

#[test]
fn recovery_after_current_move_restores_the_complete_previous_runtime() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let previous = root.path().join(".venv.previous");
    std::fs::create_dir(&previous).expect("previous runtime");
    std::fs::write(previous.join("complete"), b"old").expect("previous marker");

    recover_at(root.path()).expect("recover moved current runtime");

    assert!(root.path().join(".venv/complete").is_file());
    assert!(!previous.exists());
}

#[test]
fn receipt_requires_exact_known_fields_before_reuse() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let staged = root.path().join(".venv.next");
    std::fs::create_dir(&staged).expect("staged runtime");
    let manifest = RuntimeManifest::for_test(3, 14);
    let hash = "a".repeat(64);
    let layout = layout(root.path());
    super::runtime_receipt::write_receipt(&layout, &manifest, &hash).expect("write receipt");
    std::fs::rename(&staged, root.path().join(".venv")).expect("publish receipt runtime");

    assert!(reusable_at(root.path(), &manifest, &hash).expect("valid receipt reuses runtime"));
    assert!(
        !reusable_at(root.path(), &RuntimeManifest::for_test(3, 13), &hash)
            .expect("different manifest is not reusable")
    );
    assert!(!reusable_at(root.path(), &manifest, &"b".repeat(64))
        .expect("different source is not reusable"));
    std::fs::write(
        root.path().join(".venv/.runtime.json"),
        format!(
            r#"{{"schema_version":1,"python_major":3,"python_minor":14,"requirements_sha256":"{hash}","source_sha256":"{hash}","future":true}}"#
        ),
    )
    .expect("unknown receipt field");

    assert!(!reusable_at(root.path(), &manifest, &hash).expect("unknown receipt is not reusable"));
}

#[test]
fn only_mark_started_discards_the_previous_runtime() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let current = root.path().join(".venv");
    let previous = root.path().join(".venv.previous");
    std::fs::create_dir(&current).expect("published runtime");
    std::fs::create_dir(&previous).expect("previous runtime");

    mark_started_at(root.path()).expect("cleanup after readiness");

    assert!(current.is_dir());
    assert!(!previous.exists());
}

#[test]
fn recovery_keeps_a_previous_runtime_until_readiness_is_confirmed() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let current = root.path().join(".venv");
    let previous = root.path().join(".venv.previous");
    std::fs::create_dir(&current).expect("published runtime");
    std::fs::create_dir(&previous).expect("previous runtime");

    recover_at(root.path()).expect("recover after publication");

    assert!(current.is_dir());
    assert!(previous.is_dir());
}

#[cfg(unix)]
#[test]
fn a_receipt_without_an_executable_runtime_is_not_reusable() {
    let root = tempfile::tempdir().expect("temporary sidecar");
    let python = root.path().join(".venv/bin/python");
    std::fs::create_dir_all(python.parent().expect("python parent")).expect("venv bin");
    std::fs::write(&python, b"not executable").expect("python placeholder");

    assert!(!super::runtime_environment_fs::regular_executable(
        &root.path().join(".venv/bin/python")
    )
    .expect("non executable runtime is rejected"));
}

#[cfg(unix)]
#[test]
fn recovery_refuses_a_staged_symlink_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("temporary sidecar");
    let outside = tempfile::tempdir().expect("outside runtime");
    std::fs::write(outside.path().join("marker"), b"safe").expect("outside marker");
    symlink(outside.path(), root.path().join(".venv.next")).expect("staged symlink");

    assert!(recover_at(root.path()).is_err());
    assert!(outside.path().join("marker").is_file());
}

#[cfg(unix)]
#[tokio::test]
async fn ensure_runs_the_offline_smoke_checked_staged_runtime_before_publication() {
    let fixture = RuntimeFixture::new(false);
    let python = fixture.compatible_python().await;
    fixture.clear_commands();
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("runtime admission");
    let layout = super::runtime_environment_fs::Layout::at(fixture.root()).expect("layout");

    let runtime = RuntimeEnvironment::ensure_with_layout(
        &fixture.source,
        &fixture.wheelhouse(),
        &python,
        &admission.cancellation(),
        &layout,
        |layout| {
            assert!(layout.staged.join(".runtime.json").is_file());
            super::runtime_environment_fs::publish(layout)
        },
    )
    .await;
    let runtime = runtime.expect("published runtime");

    assert_eq!(runtime, fixture.root().join(".venv/bin/python"));
    assert!(fixture.root().join(".venv/.runtime.json").is_file());
    assert!(!fixture.root().join(".venv.previous").exists());
    assert_eq!(fixture.commands(), fixture.expected_commands());
}

#[cfg(unix)]
#[tokio::test]
async fn a_failed_smoke_never_publishes_over_the_old_runtime() {
    let fixture = RuntimeFixture::new(true);
    let old = fixture.root().join(".venv/old-runtime");
    std::fs::create_dir_all(old.parent().expect("old parent")).expect("old runtime");
    std::fs::write(&old, b"keep").expect("old marker");
    let python = fixture.compatible_python().await;
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("runtime admission");
    let layout = super::runtime_environment_fs::Layout::at(fixture.root()).expect("layout");

    let result = RuntimeEnvironment::ensure_with_layout(
        &fixture.source,
        &fixture.wheelhouse(),
        &python,
        &admission.cancellation(),
        &layout,
        super::runtime_environment_fs::publish,
    )
    .await;

    assert!(matches!(
        result,
        Err(super::runtime_error::RuntimeError::EnvironmentUnavailable)
    ));
    assert!(old.is_file());
    assert!(!fixture.root().join(".venv.previous").exists());
    assert!(!fixture.root().join(".venv/.runtime.json").exists());
    assert!(fixture.root().join(".venv.next").is_dir());
}

#[cfg(unix)]
#[tokio::test]
async fn all_runtime_stages_share_one_absolute_deadline() {
    let fixture = RuntimeFixture::new_with_stage_delay(false, "0.04");
    let old = fixture.root().join(".venv/old-runtime");
    std::fs::create_dir_all(old.parent().expect("old parent")).expect("old runtime");
    std::fs::write(&old, b"keep").expect("old marker");
    let python = fixture.compatible_python().await;
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().expect("runtime admission");
    let layout = super::runtime_environment_fs::Layout::at(fixture.root()).expect("layout");

    let result = RuntimeEnvironment::ensure_with_layout_deadline(
        &fixture.source,
        &fixture.wheelhouse(),
        &python,
        &admission.cancellation(),
        tokio::time::Instant::now() + std::time::Duration::from_millis(50),
        &layout,
        super::runtime_environment_fs::publish,
    )
    .await;

    assert!(matches!(
        result,
        Err(super::runtime_error::RuntimeError::EnvironmentUnavailable)
    ));
    assert!(old.is_file());
    assert!(!fixture.root().join(".venv.previous").exists());
}

#[cfg(unix)]
struct RuntimeFixture {
    temp: tempfile::TempDir,
    source: std::path::PathBuf,
    wheels: std::path::PathBuf,
    commands: std::path::PathBuf,
    manifest: RuntimeManifest,
}

#[cfg(unix)]
impl RuntimeFixture {
    fn new(fail_smoke: bool) -> Self {
        Self::new_with_stage_delay(fail_smoke, "0")
    }

    fn new_with_stage_delay(fail_smoke: bool, stage_delay: &str) -> Self {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().expect("runtime fixture");
        let source = temp.path().join("source");
        let wheels = temp.path().join("wheels");
        let commands = temp.path().join("commands");
        let fail_marker = temp.path().join("fail-smoke");
        std::fs::create_dir_all(source.join("searx")).expect("source layout");
        std::fs::create_dir(&wheels).expect("wheelhouse");
        for file in [
            "setup.py",
            "requirements.txt",
            "LICENSE",
            "searx/version.py",
            "searx/webapp.py",
        ] {
            std::fs::write(source.join(file), b"fixture").expect("source file");
        }
        if fail_smoke {
            std::fs::write(&fail_marker, b"fail").expect("smoke marker");
        }
        let bin = temp.path().join("bin");
        std::fs::create_dir(&bin).expect("python bin");
        let script = format!(
            r#"#!/bin/sh
{{ for arg in "$@"; do printf '%s\037' "$arg"; done; printf '\n'; }} >> '{commands}'
if [ "$1" = "-c" ] && [ "$2" = "import sys; print(sys.implementation.name); print(sys.version_info.major); print(sys.version_info.minor)" ]; then
  printf 'cpython\n3\n14\n'
  exit 0
fi
/bin/sleep {stage_delay}
if [ "$1" = "-m" ] && [ "$2" = "venv" ]; then
  /bin/mkdir -p "$3/bin"
  /bin/cp "$0" "$3/bin/python"
  /bin/chmod 755 "$3/bin/python"
  exit 0
fi
if [ "$1" = "-c" ] && [ "$2" = "import lxml, markupsafe, msgspec, yaml, searx.webapp" ] && [ -f '{fail_marker}' ]; then
  exit 17
fi
exit 0
"#,
            commands = commands.display(),
            fail_marker = fail_marker.display(),
            stage_delay = stage_delay,
        );
        let program = bin.join("python3.14");
        std::fs::write(&program, script).expect("fake Python");
        let mut permissions = std::fs::metadata(&program).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(program, permissions).expect("executable fake Python");
        Self {
            temp,
            source,
            wheels,
            commands,
            manifest: RuntimeManifest::for_test(3, 14),
        }
    }

    fn root(&self) -> &std::path::Path {
        self.temp.path()
    }

    async fn compatible_python(&self) -> super::python_runtime::PythonRuntime {
        super::python_runtime::PythonRuntime::resolve_with_path(
            &self.manifest,
            &self.root().join("bin"),
        )
        .await
        .expect("compatible fake Python")
    }

    fn wheelhouse(&self) -> super::wheels::Wheelhouse {
        super::wheels::Wheelhouse {
            path: self.wheels.clone(),
            manifest: self.manifest.clone(),
        }
    }

    fn clear_commands(&self) {
        std::fs::write(&self.commands, b"").expect("clear probe command");
    }

    fn commands(&self) -> Vec<Vec<String>> {
        std::fs::read_to_string(&self.commands)
            .expect("command log")
            .lines()
            .map(|line| {
                line.split('\u{1f}')
                    .filter(|arg| !arg.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .collect()
    }

    fn expected_commands(&self) -> Vec<Vec<String>> {
        vec![
            vec!["-m", "venv", ".venv.next"],
            vec![
                "-m",
                "pip",
                "install",
                "--no-index",
                "--find-links",
                "wheels",
                "setuptools",
                "wheel",
            ],
            vec![
                "-m",
                "pip",
                "install",
                "--no-index",
                "--find-links",
                "wheels",
                "-r",
                "requirements.txt",
            ],
            vec!["-c", "import lxml, markupsafe, msgspec, yaml, searx.webapp"],
        ]
        .into_iter()
        .map(|command| {
            command
                .into_iter()
                .map(|argument| match argument {
                    ".venv.next" => self.root().join(".venv.next").display().to_string(),
                    "wheels" => self.wheels.display().to_string(),
                    "requirements.txt" => {
                        self.source.join("requirements.txt").display().to_string()
                    }
                    _ => argument.to_string(),
                })
                .collect()
        })
        .collect()
    }
}

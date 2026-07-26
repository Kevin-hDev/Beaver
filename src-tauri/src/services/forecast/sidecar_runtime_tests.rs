use super::{
    ensure_runtime, expected_requirements, family_runtime_ready, runtime_paths, RuntimePaths,
};
use std::fs;

fn write_fake_runtime(paths: &RuntimePaths, stamp: &str, marker: &str) {
    let python = paths.python_in(&paths.live);
    fs::create_dir_all(python.parent().unwrap()).unwrap();
    fs::write(python, marker).unwrap();
    fs::write(paths.live.join(".requirements.stamp"), stamp).unwrap();
}

#[test]
fn readiness_requires_the_exact_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let paths = runtime_paths(temp.path(), "chronos-bolt").unwrap();
    write_fake_runtime(&paths, "stale", "python");

    assert!(!family_runtime_ready(temp.path(), "chronos-bolt"));

    let expected = expected_requirements(temp.path(), "chronos-bolt").unwrap();
    fs::write(paths.live.join(".requirements.stamp"), expected).unwrap();
    assert!(family_runtime_ready(temp.path(), "chronos-bolt"));
}

#[test]
fn ensure_runtime_never_installs_missing_dependencies() {
    let temp = tempfile::tempdir().unwrap();

    assert!(ensure_runtime(temp.path(), "chronos-bolt").is_err());
    assert!(!temp.path().join(".venvs").exists());
}

#[test]
fn staged_runtime_replaces_live_runtime_cleanly() {
    let temp = tempfile::tempdir().unwrap();
    let paths = runtime_paths(temp.path(), "chronos-bolt").unwrap();
    write_fake_runtime(&paths, "old", "old-python");
    let staged_python = paths.python_in(&paths.staging);
    fs::create_dir_all(staged_python.parent().unwrap()).unwrap();
    fs::write(&staged_python, "new-python").unwrap();

    super::commit_staged_runtime(&paths).unwrap();

    assert_eq!(
        fs::read_to_string(paths.python_in(&paths.live)).unwrap(),
        "new-python"
    );
    assert!(!paths.staging.exists());
    assert!(!paths.backup.exists());
}

#[test]
fn missing_staging_never_removes_the_live_runtime() {
    let temp = tempfile::tempdir().unwrap();
    let paths = runtime_paths(temp.path(), "chronos-bolt").unwrap();
    write_fake_runtime(&paths, "current", "live-python");

    assert!(super::commit_staged_runtime(&paths).is_err());
    assert_eq!(
        fs::read_to_string(paths.python_in(&paths.live)).unwrap(),
        "live-python"
    );
}

#[test]
fn model_family_manifests_are_version_pinned() {
    let temp = tempfile::tempdir().unwrap();

    for family in [
        "chronos-bolt",
        "chronos-2",
        "timesfm-2-5",
        "toto-2",
        "moirai-2",
        "flowstate",
        "tabpfn-ts",
        "tirex",
        "kairos",
        "sundial",
    ] {
        let requirements = expected_requirements(temp.path(), family).unwrap();
        assert!(requirements.contains("=="), "{family}");
        assert!(requirements.contains("--hash=sha256:"), "{family}");
        assert!(!requirements.contains("git+"), "{family}");
        assert!(!requirements.contains("--extra-index-url"), "{family}");
        assert!(!requirements.contains("--trusted-host"), "{family}");
    }
}

#[test]
fn moirai_uses_the_official_wheel_with_bounded_resolution() {
    let temp = tempfile::tempdir().unwrap();

    let requirements = expected_requirements(temp.path(), "moirai-2").unwrap();

    assert!(requirements.contains("uni2ts==2.0.0"));
    assert!(requirements.contains("jax==0.6.1"));
    assert!(requirements.contains("multiprocess==0.70.16"));
    assert!(!requirements.contains("git+"));
}

#[test]
fn chronos_families_share_the_same_runtime_directory() {
    let temp = tempfile::tempdir().unwrap();
    let bolt = runtime_paths(temp.path(), "chronos-bolt").unwrap();
    let chronos_2 = runtime_paths(temp.path(), "chronos-2").unwrap();

    assert_eq!(bolt.live, chronos_2.live);
}

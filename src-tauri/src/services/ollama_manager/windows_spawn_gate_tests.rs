use super::path_identity_resolver::NativePathIdentityResolver;
use super::spawn_gate_windows;
use super::spawn_profile::OllamaSpawnAttempt;
use super::spawn_profile_test_support::env;
use super::types::OllamaEndpoint;
use crate::services::paths::ollama_paths;
use std::ffi::OsStr;
use std::num::NonZeroU16;
use std::time::{Duration, Instant};

fn attempt(
    root: &std::path::Path,
) -> (
    tempfile::TempDir,
    tempfile::TempDir,
    super::spawn_profile::OllamaSpawnProfile,
) {
    let root = dunce::canonicalize(root).expect("canonical root");
    let paths = ollama_paths(&root);
    std::fs::create_dir_all(paths.active.join("bin")).expect("active");
    let command = std::env::var_os("ComSpec").expect("ComSpec");
    std::fs::copy(command, paths.active.join("bin").join("ollama.exe")).expect("binary");
    let models = tempfile::tempdir().expect("models");
    let models_path = dunce::canonicalize(models.path()).expect("canonical models");
    let cwd = dunce::canonicalize(std::env::current_dir().expect("cwd")).expect("canonical cwd");
    let profile = super::spawn_profile::OllamaSpawnProfile::resolve(
        &paths,
        env(&[
            ("USERPROFILE", root.to_str().expect("root")),
            ("OLLAMA_MODELS", models_path.to_str().expect("models")),
        ]),
        &cwd,
        &NativePathIdentityResolver,
    )
    .expect("profile");
    let guard = tempfile::tempdir().expect("guard");
    (models, guard, profile)
}

#[test]
fn suspended_child_enters_global_job_before_resume_and_reaps() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    assert!(profile.executable().has_stable_handle());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_435).expect("port"));
    let attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let expected = profile
        .executable()
        .execution_identity()
        .expect("image identity");
    let mut process = spawn_gate_windows::create(&attempt).expect("suspended process");
    assert_ne!(process.identity().executable, 0);
    assert_eq!(process.identity().executable, expected);
    process.revalidate(expected).expect("stable identity");
    process
        .terminate_and_reap(Instant::now() + Duration::from_secs(2))
        .expect("reap");
}

#[test]
fn expired_wait_never_releases_ownership_before_reap() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_436).expect("port"));
    let attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let mut process = spawn_gate_windows::create(&attempt).expect("suspended process");
    let first = process.terminate_and_reap(Instant::now());
    assert_eq!(first, Err(super::process::OllamaProcessError::Reap));
    process
        .terminate_and_reap(Instant::now() + Duration::from_secs(2))
        .expect("reap after timeout");
}

#[test]
fn frozen_environment_is_exact_and_nul_rejected() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_437).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let block = spawn_gate_windows::environment_block(&spawn_attempt).expect("environment");
    let text = String::from_utf16(&block).expect("utf16");
    assert!(text.contains("OLLAMA_HOST=127.0.0.1:11437\0"));
    assert!(text.ends_with("\0\0"));
    let mut invalid = Vec::new();
    assert!(spawn_gate_windows::append_entry(
        &mut invalid,
        OsStr::new("BAD\0KEY"),
        OsStr::new("v")
    )
    .is_err());
}

#[test]
fn executable_identity_mismatch_is_rejected_before_resume() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_438).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let expected = profile
        .executable()
        .execution_identity()
        .expect("image identity");
    let mut process = spawn_gate_windows::create(&spawn_attempt).expect("suspended process");
    assert!(process.revalidate(expected ^ 1).is_err());
    process
        .terminate_and_reap(Instant::now() + Duration::from_secs(2))
        .expect("reap");
}

#[test]
fn image_identity_is_read_from_the_suspended_process_handle_after_path_swap() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_439).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let executable = profile.executable().path().to_path_buf();
    let backup = executable.with_extension("stable");
    let mut replacement =
        std::path::PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot"));
    replacement.push("System32");
    replacement.push("WindowsPowerShell");
    replacement.push("v1.0");
    replacement.push("powershell.exe");
    assert!(replacement.exists(), "replacement image");
    let mut restored = false;
    let mut process = spawn_gate_windows::create_with_hooks_for_test(
        &spawn_attempt,
        || {
            std::fs::rename(&executable, &backup).expect("move active image");
            std::fs::copy(replacement, &executable).expect("replace active image");
        },
        || {
            std::fs::remove_file(&executable).expect("remove replacement");
            std::fs::rename(&backup, &executable).expect("restore active image");
            restored = true;
        },
    )
    .expect("suspended process");
    assert!(restored);
    let expected = profile
        .executable()
        .execution_identity()
        .expect("image identity");
    assert_eq!(process.identity().executable, expected);
    process
        .terminate_and_reap(Instant::now() + Duration::from_secs(2))
        .expect("reap");
}

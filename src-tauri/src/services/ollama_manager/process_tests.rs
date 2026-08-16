use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::process::{DefaultOllamaProcessLauncher, OllamaProcessError, OllamaProcessLauncher};
use super::spawn_profile::OllamaSpawnAttempt;
use super::spawn_profile_test_support::env;
use super::types::OllamaEndpoint;
use crate::app_exit::AppExitCoordinator;
use crate::services::paths::ollama_paths;
use std::num::NonZeroU16;
use std::time::{Duration, Instant};

#[cfg(unix)]
fn attempt(
    root: &std::path::Path,
) -> (
    tempfile::TempDir,
    tempfile::TempDir,
    super::spawn_profile::OllamaSpawnProfile,
) {
    let canonical_root = std::fs::canonicalize(root).expect("canonical root");
    let paths = ollama_paths(&canonical_root);
    std::fs::create_dir_all(paths.active.join("bin")).expect("active");
    std::fs::copy("/usr/bin/yes", paths.active.join("bin").join("ollama")).expect("binary");
    let models = tempfile::tempdir().expect("models");
    let canonical_models = std::fs::canonicalize(models.path()).expect("canonical models");
    let resolver = super::path_identity_resolver::NativePathIdentityResolver;
    let profile = super::spawn_profile::OllamaSpawnProfile::resolve(
        &paths,
        env(&[
            ("HOME", canonical_root.to_str().expect("root")),
            ("OLLAMA_MODELS", canonical_models.to_str().expect("models")),
        ]),
        &canonical_root,
        &resolver,
    )
    .unwrap_or_else(|error| {
        panic!(
            "profile {error:?}: root={} models={}",
            canonical_root.display(),
            models.path().display()
        )
    });
    let guard = tempfile::tempdir_in(canonical_root.parent().expect("parent")).expect("guard");
    (models, guard, profile)
}

#[cfg(unix)]
#[test]
fn gated_process_can_be_terminated_and_reaped_before_publication() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_435).expect("port"));
    let attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let launcher = DefaultOllamaProcessLauncher::new(BundleFingerprint {
        version: OllamaVersion::parse("1.2.3").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"ab".repeat(32)).expect("digest"),
    });
    let process = launcher.create_gated(&attempt).expect("gated process");
    let identity = process.identity().expect("identity");
    assert!(identity.pid > 1);
    process
        .terminate_and_reap(Instant::now() + Duration::from_secs(2))
        .expect("reap");
}

#[cfg(unix)]
#[test]
fn gate_blocks_execution_and_drop_reaps_repeatedly() {
    for _ in 0..8 {
        let root = tempfile::tempdir().expect("root");
        let (_models, _guard, profile) = attempt(root.path());
        let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_438).expect("port"));
        let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
        let expected = profile.executable().identity().value();
        let launcher = DefaultOllamaProcessLauncher::new(BundleFingerprint {
            version: OllamaVersion::parse("1.2.3").expect("version"),
            executable_sha256: Sha256Digest::from_hex(&"ab".repeat(32)).expect("digest"),
        });
        let gated = launcher.create_gated(&spawn_attempt).expect("gated");
        let identity = gated.identity().expect("identity");
        let before = super::super::owned_process::OwnedProcess::identity(identity.pid)
            .expect("blocked child");
        assert_ne!(before.executable, expected);
        drop(gated);
        assert_eq!(unsafe { libc::kill(identity.pid as libc::pid_t, 0) }, -1);
    }
}

#[cfg(unix)]
#[test]
fn gated_process_drop_reaps_during_unwind() {
    let seen_pid = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let seen_for_panic = std::sync::Arc::clone(&seen_pid);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let root = tempfile::tempdir().expect("root");
        let (_models, _guard, profile) = attempt(root.path());
        let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_446).expect("port"));
        let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
        let launcher = DefaultOllamaProcessLauncher::new(BundleFingerprint {
            version: OllamaVersion::parse("1.2.3").expect("version"),
            executable_sha256: Sha256Digest::from_hex(&"ab".repeat(32)).expect("digest"),
        });
        let gated = launcher.create_gated(&spawn_attempt).expect("gated");
        seen_for_panic.store(
            gated.identity().expect("identity").pid,
            std::sync::atomic::Ordering::SeqCst,
        );
        panic!("cutpoint");
    }));
    assert!(result.is_err());
    let pid = seen_pid.load(std::sync::atomic::Ordering::SeqCst);
    assert!(pid > 1);
    assert_eq!(unsafe { libc::kill(pid as libc::pid_t, 0) }, -1);
}

#[cfg(unix)]
#[test]
fn owned_drop_keeps_receipt_and_slot_when_reap_fails() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_443).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let launcher = DefaultOllamaProcessLauncher::new(BundleFingerprint {
        version: OllamaVersion::parse("6.7.8").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"55".repeat(32)).expect("digest"),
    });
    let gated = launcher.create_gated(&spawn_attempt).expect("gated");
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let receipt_path = paths.process_receipt.clone();
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        receipt_path.clone(),
        receipt_path.with_extension("tmp"),
    );
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    let publisher = coordinator.emergency_publisher();
    let mut owned = gated.publish(&store, &publisher).expect("published");
    let pid = owned.identity().pid;
    let key = owned
        .registration
        .as_ref()
        .expect("registration")
        .key_for_test();
    owned
        .native
        .as_mut()
        .expect("native")
        .force_reap_failure_for_test();
    drop(owned);
    assert_eq!(publisher.active_count_for_test(), 1);
    assert!(store.read().expect("retained receipt").is_some());
    unsafe {
        libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
        libc::kill(pid as libc::pid_t, libc::SIGKILL);
        libc::waitpid(pid as libc::pid_t, std::ptr::null_mut(), 0);
    }
    crate::services::owned_process::release(pid);
    assert!(publisher.clear_for_test(key));
    store.remove().expect("cleanup receipt");
}

#[cfg(unix)]
#[test]
fn publication_uses_the_supplied_bundle_fingerprint_and_opens_gate() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_436).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let bundle = BundleFingerprint {
        version: OllamaVersion::parse("8.7.6").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"cd".repeat(32)).expect("digest"),
    };
    let launcher = DefaultOllamaProcessLauncher::new(bundle.clone());
    let gated = launcher
        .create_gated(&spawn_attempt)
        .expect("gated process");
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let receipt_path = paths.process_receipt.clone();
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        receipt_path.clone(),
        receipt_path.with_extension("tmp"),
    );
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    let owned = gated
        .publish(&store, &coordinator.emergency_publisher())
        .expect("publication");
    #[cfg(unix)]
    assert!(owned
        .native
        .as_ref()
        .expect("native")
        .exec_link_exists_for_test());
    let written = store.read().expect("read receipt").expect("receipt");
    assert_eq!(written.bundle, bundle);
    owned
        .terminate_and_reap(Instant::now() + Duration::from_secs(2))
        .expect("reap");
    #[cfg(unix)]
    assert_eq!(
        std::fs::read_dir(&paths.active)
            .expect("bundle entries")
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with(".beaver-gated-"))
            .count(),
        0,
        "the recovery identity link must live exactly as long as the owned process"
    );
    assert!(store.read().expect("removed receipt").is_none());
}

#[cfg(unix)]
#[test]
fn replacement_after_profile_resolution_is_rejected_before_fork() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let executable = profile.executable().path().to_path_buf();
    let backup = executable.with_extension("original");
    std::fs::rename(&executable, &backup).expect("move stable executable");
    std::fs::copy("/usr/bin/false", &executable).expect("replacement");
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_437).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let launcher = DefaultOllamaProcessLauncher::new(BundleFingerprint {
        version: OllamaVersion::parse("1.2.3").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"ab".repeat(32)).expect("digest"),
    });
    assert!(matches!(
        launcher.create_gated(&spawn_attempt),
        Err(OllamaProcessError::Identity)
    ));
}

#[cfg(unix)]
#[test]
fn identity_change_after_receipt_write_fails_before_emergency_admission() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_439).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let bundle = BundleFingerprint {
        version: OllamaVersion::parse("2.3.4").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"ef".repeat(32)).expect("digest"),
    };
    let launcher = DefaultOllamaProcessLauncher::new(bundle);
    let gated = launcher.create_gated(&spawn_attempt).expect("gated");
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let receipt_path = paths.process_receipt.clone();
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        receipt_path.clone(),
        receipt_path.with_extension("tmp"),
    );
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    assert!(matches!(
        gated.publish_with_identity_change_for_test(&store, &coordinator.emergency_publisher()),
        Err(OllamaProcessError::Identity)
    ));
    assert!(store.read().expect("reaped receipt").is_none());
}

#[cfg(unix)]
#[test]
fn revalidation_reap_failure_leaves_durable_recovery_handoff() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_445).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let bundle = BundleFingerprint {
        version: OllamaVersion::parse("7.8.9").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"77".repeat(32)).expect("digest"),
    };
    let launcher = DefaultOllamaProcessLauncher::new(bundle);
    let mut gated = launcher.create_gated(&spawn_attempt).expect("gated");
    let pid = gated.identity().expect("identity").pid;
    gated.force_reap_failure_for_test();
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        paths.process_receipt.clone(),
        paths.process_receipt_tmp.clone(),
    );
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    let result =
        gated.publish_with_identity_change_for_test(&store, &coordinator.emergency_publisher());
    assert!(matches!(result, Err(OllamaProcessError::Identity)));
    assert!(store.read().expect("durable handoff").is_some());
    unsafe { libc::waitpid(pid as libc::pid_t, std::ptr::null_mut(), 0) };
    crate::services::owned_process::release(pid);
    store.remove().expect("cleanup");
}

#[cfg(unix)]
#[test]
fn emergency_capacity_reap_failure_is_recoverable_without_a_slot() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_446).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let bundle = BundleFingerprint {
        version: OllamaVersion::parse("8.9.0").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"88".repeat(32)).expect("digest"),
    };
    let launcher = DefaultOllamaProcessLauncher::new(bundle);
    let mut gated = launcher.create_gated(&spawn_attempt).expect("gated");
    gated
        .open_gate_and_wait_for_test()
        .expect("running executable");
    gated.force_reap_failure_for_test();
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    let publisher = coordinator.emergency_publisher();
    let registrations = (0..crate::app_exit::EMERGENCY_CAPACITY)
        .map(|index| publisher.publish(index as u32 + 2, index as u64 + 1, index as u64 + 2, 1))
        .collect::<Result<Vec<_>, _>>()
        .expect("capacity");
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        paths.process_receipt.clone(),
        paths.process_receipt_tmp.clone(),
    );
    let publish_result = gated.publish(&store, &publisher);
    assert!(matches!(
        publish_result,
        Err(OllamaProcessError::EmergencyCapacity)
    ));
    assert!(store.read().expect("durable handoff").is_some());
    let expected = profile
        .executable()
        .execution_identity()
        .expect("executable");
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let recovery = launcher
            .recover_receipt(&store, expected, deadline)
            .expect("recovery");
        if recovery == super::process_receipt::ProcessReceiptRecovery::Reaped {
            break;
        }
        assert_eq!(
            recovery,
            super::process_receipt::ProcessReceiptRecovery::RecoveryRequired
        );
        assert!(
            Instant::now() < deadline,
            "recovery did not reap exited child"
        );
        std::thread::yield_now();
    }
    drop(registrations);
}

#[cfg(unix)]
#[test]
fn publish_write_failure_keeps_an_existing_receipt_and_reaps_child() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_440).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let bundle = BundleFingerprint {
        version: OllamaVersion::parse("3.4.5").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"11".repeat(32)).expect("digest"),
    };
    let launcher = DefaultOllamaProcessLauncher::new(bundle.clone());
    let gated = launcher.create_gated(&spawn_attempt).expect("gated");
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let receipt_path = paths.process_receipt.clone();
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        receipt_path.clone(),
        receipt_path.with_extension("tmp"),
    );
    let previous = super::process_receipt::ProcessReceipt::new(
        42,
        99,
        7,
        BundleFingerprint {
            version: OllamaVersion::parse("0.1.0").expect("version"),
            executable_sha256: Sha256Digest::from_hex(&"22".repeat(32)).expect("digest"),
        },
    )
    .expect("previous receipt");
    store.write_new(&previous).expect("previous receipt write");
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    assert!(matches!(
        gated.publish(&store, &coordinator.emergency_publisher()),
        Err(OllamaProcessError::Receipt)
    ));
    assert_eq!(store.read().expect("receipt"), Some(previous));
}

#[cfg(unix)]
#[test]
fn publish_gate_failure_reaps_child_and_releases_registration() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_441).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let launcher = DefaultOllamaProcessLauncher::new(BundleFingerprint {
        version: OllamaVersion::parse("4.5.6").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"33".repeat(32)).expect("digest"),
    });
    let mut gated = launcher.create_gated(&spawn_attempt).expect("gated");
    let pid = gated.identity().expect("identity").pid;
    gated.close_gate_for_test();
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let receipt_path = paths.process_receipt.clone();
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        receipt_path.clone(),
        receipt_path.with_extension("tmp"),
    );
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    let publisher = coordinator.emergency_publisher();
    assert!(matches!(
        gated.publish(&store, &publisher),
        Err(OllamaProcessError::Gate)
    ));
    assert_eq!(publisher.active_count_for_test(), 0);
    assert!(store.read().expect("receipt").is_none());
    assert_eq!(unsafe { libc::kill(pid as libc::pid_t, 0) }, -1);
}

#[cfg(unix)]
#[test]
fn publish_emergency_capacity_failure_reaps_child_and_keeps_slots() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_442).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let launcher = DefaultOllamaProcessLauncher::new(BundleFingerprint {
        version: OllamaVersion::parse("5.6.7").expect("version"),
        executable_sha256: Sha256Digest::from_hex(&"44".repeat(32)).expect("digest"),
    });
    let gated = launcher.create_gated(&spawn_attempt).expect("gated");
    let pid = gated.identity().expect("identity").pid;
    let coordinator = AppExitCoordinator::initialize().expect("coordinator");
    let publisher = coordinator.emergency_publisher();
    let registrations = (0..crate::app_exit::EMERGENCY_CAPACITY)
        .map(|index| publisher.publish(index as u32 + 2, index as u64 + 1, index as u64 + 2, 1))
        .collect::<Result<Vec<_>, _>>()
        .expect("capacity slots");
    let paths = ollama_paths(&std::fs::canonicalize(root.path()).expect("canonical root"));
    let receipt_path = paths.process_receipt.clone();
    let store = super::process_receipt::ProcessReceiptStore::new(
        std::sync::Arc::new(super::durable_fs::platform_fs()),
        receipt_path.clone(),
        receipt_path.with_extension("tmp"),
    );
    assert!(matches!(
        gated.publish(&store, &publisher),
        Err(OllamaProcessError::EmergencyCapacity)
    ));
    assert_eq!(
        publisher.active_count_for_test(),
        crate::app_exit::EMERGENCY_CAPACITY
    );
    assert!(store.read().expect("receipt").is_none());
    assert_eq!(unsafe { libc::kill(pid as libc::pid_t, 0) }, -1);
    drop(registrations);
    assert_eq!(publisher.active_count_for_test(), 0);
}

#[cfg(unix)]
#[test]
fn native_admission_failure_reaps_a_child_without_opening_gate() {
    let root = tempfile::tempdir().expect("root");
    let (_models, _guard, profile) = attempt(root.path());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_444).expect("port"));
    let spawn_attempt = OllamaSpawnAttempt::new(&profile, endpoint);
    let seen_pid = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let seen_for_admitter = std::sync::Arc::clone(&seen_pid);
    let result =
        super::spawn_gate_unix::create_with_admitter_for_test(&spawn_attempt, move |pid| {
            seen_for_admitter.store(pid, std::sync::atomic::Ordering::SeqCst);
            Err(crate::services::owned_process::OwnedProcessError::Admission)
        });
    assert!(matches!(result, Err(OllamaProcessError::Admission)));
    let pid = seen_pid.load(std::sync::atomic::Ordering::SeqCst);
    assert!(pid > 1);
    assert_eq!(unsafe { libc::kill(pid as libc::pid_t, 0) }, -1);
}

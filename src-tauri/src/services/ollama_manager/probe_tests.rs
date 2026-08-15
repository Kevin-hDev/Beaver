use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::net::TcpListener;
use std::num::NonZeroU16;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use super::error::OllamaErrorCode;
use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};
use super::port::OllamaPortAllocator;
use super::probe::{OllamaTargetProbe, OwnedOllamaTargetProbe, PreparedBundle, TargetValidation};
use super::spawn_profile::OllamaSpawnProfile;
use crate::services::paths::ollama_paths;

struct Fixture {
    _root: tempfile::TempDir,
    profile: OllamaSpawnProfile,
    target: PreparedBundle,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("temporary bundle root");
    let root_path = dunce::canonicalize(root.path()).expect("canonical bundle root");
    let paths = ollama_paths(&root_path);
    let executable = paths.active.join("bin").join("ollama");
    std::fs::create_dir_all(executable.parent().expect("bin parent")).expect("bin directory");
    std::fs::write(&executable, b"#!/bin/sh\nexit 0\n").expect("probe executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
            .expect("executable permissions");
    }
    let cwd = root_path.join("cwd");
    std::fs::create_dir_all(&cwd).expect("working directory");
    let resolver = NativePathIdentityResolver;
    let profile = OllamaSpawnProfile::resolve_probe(
        &paths,
        [("HOME".into(), root_path.as_os_str().into())],
        &cwd,
        &resolver,
    )
    .expect("probe profile");
    let root_directory = resolver
        .canonical_directory(&paths.active)
        .expect("bundle directory");
    let fingerprint = BundleFingerprint {
        version: OllamaVersion::parse("1.2.3").expect("version"),
        executable_sha256: digest_for_file(&executable),
    };
    Fixture {
        _root: root,
        target: PreparedBundle {
            root: root_directory,
            executable: profile.executable().clone(),
            fingerprint,
        },
        profile,
    }
}

fn digest_for_file(path: &Path) -> Sha256Digest {
    let bytes = std::fs::read(path).expect("file bytes");
    Sha256Digest::from_hex(&hex::encode(Sha256::digest(bytes))).expect("digest")
}

fn assert_invalid(result: TargetValidation) {
    assert!(matches!(
        result,
        TargetValidation::InvalidTarget {
            code: OllamaErrorCode::OllamaBundleInvalid
        }
    ));
}

#[test]
fn missing_executable_is_classified_as_invalid_target() {
    let fixture = fixture();
    std::fs::remove_file(fixture.profile.executable().path()).expect("remove executable");
    let result = tokio::runtime::Runtime::new().expect("runtime").block_on(
        OwnedOllamaTargetProbe::new(
            AlwaysUnavailable::default(),
            Instant::now() + Duration::from_secs(1),
        )
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new()),
    );
    assert_invalid(result);
}

#[tokio::test]
async fn invalid_permissions_are_classified_before_spawn() {
    let fixture = fixture();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            fixture.profile.executable().path(),
            std::fs::Permissions::from_mode(0o644),
        )
        .expect("remove execute permission");
    }
    let result = OwnedOllamaTargetProbe::new(
        AlwaysUnavailable::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
    .await;
    assert_invalid(result);
}

#[tokio::test]
async fn changed_executable_hash_is_classified_as_invalid() {
    let fixture = fixture();
    std::fs::write(fixture.profile.executable().path(), b"changed").expect("change executable");
    let result = OwnedOllamaTargetProbe::new(
        AlwaysUnavailable::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
    .await;
    assert_invalid(result);
}

#[tokio::test]
async fn malformed_executable_format_is_rejected_before_port_allocation() {
    let fixture = fixture();
    std::fs::write(fixture.profile.executable().path(), b"not an executable")
        .expect("change executable format");
    let allocator = AlwaysUnavailable::default();
    let calls = Arc::clone(&allocator.calls);
    let result = OwnedOllamaTargetProbe::new(allocator, Instant::now() + Duration::from_secs(1))
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
        .await;
    assert_invalid(result);
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn three_unavailable_ports_are_deferred_without_spawning() {
    let fixture = fixture();
    let allocator = AlwaysUnavailable::default();
    let calls = Arc::clone(&allocator.calls);
    let result = OwnedOllamaTargetProbe::new(allocator, Instant::now() + Duration::from_secs(1))
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
        .await;
    assert!(matches!(
        result,
        TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaValidationDeferred
        }
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn launched_then_dead_child_is_deferred_and_models_are_cleaned() {
    let fixture = fixture();
    let models_path = fixture.profile.models_directory().path().to_path_buf();
    let result = OwnedOllamaTargetProbe::new(SinglePort, Instant::now() + Duration::from_secs(1))
        .validate(&fixture.target, &fixture.profile, &CancellationToken::new())
        .await;
    assert!(matches!(result, TargetValidation::Deferred { .. }));
    assert!(!models_path.exists());
}

#[tokio::test]
async fn cancellation_is_deferred_with_a_stable_code() {
    let fixture = fixture();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let result = OwnedOllamaTargetProbe::new(
        AlwaysUnavailable::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .validate(&fixture.target, &fixture.profile, &cancellation)
    .await;
    assert!(matches!(
        result,
        TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaOperationCancelled
        }
    ));
}

#[derive(Default)]
struct AlwaysUnavailable {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl OllamaPortAllocator for AlwaysUnavailable {
    fn allocate_loopback(
        &self,
        _excluded: &[u16],
    ) -> Result<super::types::OllamaEndpoint, OllamaErrorCode> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Err(OllamaErrorCode::OllamaValidationDeferred)
    }

    async fn detect_external(
        &self,
    ) -> Result<Option<super::types::OllamaEndpoint>, OllamaErrorCode> {
        Ok(None)
    }
}

struct SinglePort;

#[async_trait]
impl OllamaPortAllocator for SinglePort {
    fn allocate_loopback(
        &self,
        excluded: &[u16],
    ) -> Result<super::types::OllamaEndpoint, OllamaErrorCode> {
        for _ in 0..4 {
            let listener = TcpListener::bind(("127.0.0.1", 0))
                .map_err(|_| OllamaErrorCode::OllamaValidationDeferred)?;
            let port = listener
                .local_addr()
                .map_err(|_| OllamaErrorCode::OllamaValidationDeferred)?
                .port();
            if !excluded.contains(&port) {
                return Ok(super::types::OllamaEndpoint::loopback(
                    NonZeroU16::new(port).expect("port"),
                ));
            }
        }
        Err(OllamaErrorCode::OllamaValidationDeferred)
    }

    async fn detect_external(
        &self,
    ) -> Result<Option<super::types::OllamaEndpoint>, OllamaErrorCode> {
        Ok(None)
    }
}

use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};

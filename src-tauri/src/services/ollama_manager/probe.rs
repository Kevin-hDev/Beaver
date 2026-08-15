#![allow(dead_code)]

use async_trait::async_trait;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

use super::canonical_executable::CanonicalExecutable;
use super::constants::MAX_PROBE_PORT_ATTEMPTS;
use super::error::OllamaErrorCode;
use super::fingerprint::BundleFingerprint;
use super::path_identity::CanonicalDirectory;
use super::port::{DefaultOllamaPortAllocator, OllamaPortAllocator};
use super::process::{NativeGatedProcess, OllamaProcessError};
use super::spawn_profile::{OllamaSpawnAttempt, OllamaSpawnProfile};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedBundle {
    pub root: CanonicalDirectory,
    pub executable: CanonicalExecutable,
    pub fingerprint: BundleFingerprint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetValidation {
    Valid { fingerprint: BundleFingerprint },
    InvalidTarget { code: OllamaErrorCode },
    Deferred { code: OllamaErrorCode },
}

#[async_trait]
pub trait OllamaTargetProbe: Send + Sync {
    async fn validate(
        &self,
        target: &PreparedBundle,
        profile: &OllamaSpawnProfile,
        cancellation: &CancellationToken,
    ) -> TargetValidation;
}

pub struct OwnedOllamaTargetProbe<A = DefaultOllamaPortAllocator> {
    allocator: A,
    deadline: Instant,
}

impl<A> OwnedOllamaTargetProbe<A> {
    pub fn new(allocator: A, deadline: Instant) -> Self {
        Self {
            allocator,
            deadline,
        }
    }
}

impl OwnedOllamaTargetProbe<DefaultOllamaPortAllocator> {
    pub fn with_deadline(deadline: Instant) -> Self {
        Self::new(DefaultOllamaPortAllocator::new(), deadline)
    }
}

#[async_trait]
impl<A> OllamaTargetProbe for OwnedOllamaTargetProbe<A>
where
    A: OllamaPortAllocator,
{
    async fn validate(
        &self,
        target: &PreparedBundle,
        profile: &OllamaSpawnProfile,
        cancellation: &CancellationToken,
    ) -> TargetValidation {
        if let Err(result) = super::probe_http::inspect_target(target, profile) {
            return result;
        }
        let mut excluded = [0_u16; MAX_PROBE_PORT_ATTEMPTS];
        let mut excluded_len = 0;
        let mut last = deferred(OllamaErrorCode::OllamaValidationDeferred);
        for _ in 0..MAX_PROBE_PORT_ATTEMPTS {
            if cancellation.is_cancelled() {
                return deferred(OllamaErrorCode::OllamaOperationCancelled);
            }
            if Instant::now() >= self.deadline {
                return last;
            }
            let endpoint = match self.allocator.allocate_loopback(&excluded[..excluded_len]) {
                Ok(endpoint) => endpoint,
                Err(code) => {
                    last = deferred(code);
                    continue;
                }
            };
            excluded[excluded_len] = endpoint.port();
            excluded_len += 1;
            let (result, can_retry) =
                probe_endpoint(target, profile, endpoint, self.deadline, cancellation).await;
            match result {
                valid @ TargetValidation::Valid { .. }
                | valid @ TargetValidation::InvalidTarget { .. } => return valid,
                deferred_result @ TargetValidation::Deferred { .. } => last = deferred_result,
            }
            if !can_retry {
                return last;
            }
        }
        last
    }
}

async fn probe_endpoint(
    target: &PreparedBundle,
    profile: &OllamaSpawnProfile,
    endpoint: super::types::OllamaEndpoint,
    deadline: Instant,
    cancellation: &CancellationToken,
) -> (TargetValidation, bool) {
    if let Err(result) = prepare_models(profile) {
        return (result, false);
    }
    let attempt = OllamaSpawnAttempt::new(profile, endpoint.clone());
    let mut process = match launch_gated(&attempt) {
        Ok(process) => process,
        Err(_) => {
            cleanup_models(profile);
            return (deferred(OllamaErrorCode::OllamaValidationDeferred), true);
        }
    };
    let executable = match profile.executable().execution_identity() {
        Some(value) => value,
        None => return reap_deferred(process, deadline, profile),
    };
    if process.revalidate(executable).is_err() || process.open_gate().is_err() {
        return reap_deferred(process, deadline, profile);
    }
    let response = super::probe_http::fetch_version(&endpoint, deadline, cancellation).await;
    let identity_ok = process.revalidate(executable).is_ok();
    let reap_ok = process.terminate_and_reap(deadline).is_ok();
    let models_cleaned = cleanup_models(profile);
    if !identity_ok || !reap_ok || !models_cleaned {
        return (
            deferred(OllamaErrorCode::OllamaValidationDeferred),
            reap_ok && models_cleaned,
        );
    }
    match response {
        Ok(version) if version == target.fingerprint.version => (
            TargetValidation::Valid {
                fingerprint: target.fingerprint.clone(),
            },
            true,
        ),
        Ok(_)
        | Err(super::probe_http::HttpProbeError::Oversized)
        | Err(super::probe_http::HttpProbeError::Malformed) => (invalid_target(), true),
        Err(super::probe_http::HttpProbeError::Cancelled) => {
            (deferred(OllamaErrorCode::OllamaOperationCancelled), true)
        }
        Err(_) => (deferred(OllamaErrorCode::OllamaValidationDeferred), true),
    }
}

fn prepare_models(profile: &OllamaSpawnProfile) -> Result<(), TargetValidation> {
    let path = profile.models_directory().path();
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(invalid_target()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => std::fs::create_dir(path)
            .map_err(|_| deferred(OllamaErrorCode::OllamaStorageUnavailable)),
        Err(_) => Err(deferred(OllamaErrorCode::OllamaStorageUnavailable)),
    }
}

fn cleanup_models(profile: &OllamaSpawnProfile) -> bool {
    let path = profile.models_directory().path();
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            std::fs::remove_dir(path).is_ok()
        }
        _ => false,
    }
}

fn reap_deferred(
    mut process: NativeGatedProcess,
    deadline: Instant,
    profile: &OllamaSpawnProfile,
) -> (TargetValidation, bool) {
    let reap_ok = process.terminate_and_reap(deadline).is_ok();
    let models_cleaned = cleanup_models(profile);
    (
        deferred(OllamaErrorCode::OllamaValidationDeferred),
        reap_ok && models_cleaned,
    )
}

#[cfg(unix)]
fn launch_gated(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    super::spawn_gate_unix::create(attempt)
}

#[cfg(windows)]
fn launch_gated(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    super::spawn_gate_windows::create(attempt)
}

fn invalid_target() -> TargetValidation {
    TargetValidation::InvalidTarget {
        code: OllamaErrorCode::OllamaBundleInvalid,
    }
}

fn deferred(code: OllamaErrorCode) -> TargetValidation {
    TargetValidation::Deferred { code }
}

#![allow(dead_code)]

use async_trait::async_trait;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::canonical_executable::CanonicalExecutable;
use super::constants::MAX_PROBE_PORT_ATTEMPTS;
use super::error::OllamaErrorCode;
use super::fingerprint::BundleFingerprint;
use super::path_identity::CanonicalDirectory;
use super::port::{DefaultOllamaPortAllocator, OllamaPortAllocator};
use super::spawn_profile::OllamaSpawnProfile;

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
    active: Mutex<()>,
}

impl<A> OwnedOllamaTargetProbe<A> {
    pub fn new(allocator: A, deadline: Instant) -> Self {
        Self {
            allocator,
            deadline,
            active: Mutex::new(()),
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
        if cancellation.is_cancelled() {
            return super::probe_support::deferred(OllamaErrorCode::OllamaOperationCancelled);
        }
        if Instant::now() >= self.deadline {
            return super::probe_support::deferred(OllamaErrorCode::OllamaValidationDeferred);
        }
        let _active = tokio::select! {
            _ = cancellation.cancelled() => {
                return super::probe_support::deferred(OllamaErrorCode::OllamaOperationCancelled);
            }
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(self.deadline)) => {
                return super::probe_support::deferred(OllamaErrorCode::OllamaValidationDeferred);
            }
            guard = self.active.lock() => guard,
        };
        if let Err(result) = super::probe_http::inspect_target(target, profile) {
            return result;
        }
        let mut excluded = [0_u16; MAX_PROBE_PORT_ATTEMPTS];
        let mut excluded_len = 0;
        let mut last = super::probe_support::deferred(OllamaErrorCode::OllamaValidationDeferred);
        for _ in 0..MAX_PROBE_PORT_ATTEMPTS {
            if cancellation.is_cancelled() {
                return super::probe_support::deferred(OllamaErrorCode::OllamaOperationCancelled);
            }
            if Instant::now() >= self.deadline {
                return last;
            }
            let endpoint = match self.allocator.allocate_loopback(&excluded[..excluded_len]) {
                Ok(endpoint) => endpoint,
                Err(code) => {
                    last = super::probe_support::deferred(code);
                    continue;
                }
            };
            excluded[excluded_len] = endpoint.port();
            excluded_len += 1;
            let (result, can_retry) = super::probe_runner::probe_endpoint(
                target,
                profile,
                endpoint,
                self.deadline,
                cancellation,
            )
            .await;
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

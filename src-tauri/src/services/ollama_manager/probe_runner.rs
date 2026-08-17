#![allow(dead_code)]

use std::time::Instant;
use tokio_util::sync::CancellationToken;

use super::error::OllamaErrorCode;
use super::probe::{PreparedBundle, TargetValidation};
use super::spawn_profile::OllamaSpawnProfile;
use super::types::OllamaEndpoint;

pub(crate) async fn probe_endpoint(
    target: &PreparedBundle,
    profile: &OllamaSpawnProfile,
    endpoint: OllamaEndpoint,
    deadline: Instant,
    cancellation: &CancellationToken,
) -> (TargetValidation, bool) {
    if let Err(result) = super::probe_ownership::prepare_models(profile) {
        return (result, false);
    }
    let attempt = super::spawn_profile::OllamaSpawnAttempt::new(profile, endpoint.clone());
    let mut process = match super::probe_support::launch_gated(&attempt) {
        Ok(process) => process,
        Err(_) => {
            super::probe_ownership::cleanup_models(profile);
            return (
                super::probe_support::deferred(OllamaErrorCode::OllamaValidationDeferred),
                true,
            );
        }
    };
    let executable = match profile.executable().execution_identity() {
        Some(value) => value,
        None => {
            return super::probe_support::reap_deferred(
                process,
                deadline,
                profile,
                OllamaErrorCode::OllamaValidationDeferred,
            )
        }
    };
    if process.revalidate(executable).is_err() || process.open_gate().is_err() {
        return super::probe_support::reap_deferred(
            process,
            deadline,
            profile,
            OllamaErrorCode::OllamaValidationDeferred,
        );
    }
    match super::probe_ownership::wait_for_owned_endpoint(
        &endpoint,
        process.identity(),
        deadline,
        cancellation,
    )
    .await
    {
        super::probe_ownership::EndpointWaitResult::Ready => {}
        super::probe_ownership::EndpointWaitResult::Cancelled => {
            return super::probe_support::reap_deferred(
                process,
                deadline,
                profile,
                OllamaErrorCode::OllamaOperationCancelled,
            )
        }
        super::probe_ownership::EndpointWaitResult::Deadline => {
            return super::probe_support::reap_deferred(
                process,
                deadline,
                profile,
                OllamaErrorCode::OllamaValidationDeferred,
            )
        }
    }
    if process.revalidate(executable).is_err() {
        return super::probe_support::reap_deferred(
            process,
            deadline,
            profile,
            OllamaErrorCode::OllamaValidationDeferred,
        );
    }
    let response = super::probe_http::fetch_version(&endpoint, deadline, cancellation).await;
    let identity_ok = process.revalidate(executable).is_ok();
    let reap_ok = process.terminate_and_reap(deadline).is_ok();
    let models_cleaned = super::probe_ownership::cleanup_models(profile);
    if !identity_ok || !reap_ok || !models_cleaned {
        return (
            super::probe_support::deferred(OllamaErrorCode::OllamaValidationDeferred),
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
        | Err(super::probe_http::HttpProbeError::Malformed) => {
            (super::probe_support::invalid_target(), true)
        }
        Err(super::probe_http::HttpProbeError::Cancelled) => (
            super::probe_support::deferred(OllamaErrorCode::OllamaOperationCancelled),
            true,
        ),
        Err(_) => (
            super::probe_support::deferred(OllamaErrorCode::OllamaValidationDeferred),
            true,
        ),
    }
}

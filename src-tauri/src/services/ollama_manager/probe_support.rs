#![allow(dead_code)]

use std::time::Instant;

use super::error::OllamaErrorCode;
use super::probe::TargetValidation;
use super::process::{NativeGatedProcess, OllamaProcessError};
use super::spawn_profile::{OllamaSpawnAttempt, OllamaSpawnProfile};

pub(crate) fn reap_deferred(
    mut process: NativeGatedProcess,
    deadline: Instant,
    profile: &OllamaSpawnProfile,
    code: OllamaErrorCode,
) -> (TargetValidation, bool) {
    let reap_ok = process.terminate_and_reap(deadline).is_ok();
    let models_cleaned = super::probe_ownership::cleanup_models(profile);
    (deferred(code), reap_ok && models_cleaned)
}

#[cfg(unix)]
pub(crate) fn launch_gated(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    super::spawn_gate_unix::create(attempt)
}

#[cfg(windows)]
pub(crate) fn launch_gated(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    super::spawn_gate_windows::create(attempt)
}

pub(crate) fn invalid_target() -> TargetValidation {
    TargetValidation::InvalidTarget {
        code: OllamaErrorCode::OllamaBundleInvalid,
    }
}

pub(crate) fn deferred(code: OllamaErrorCode) -> TargetValidation {
    TargetValidation::Deferred { code }
}

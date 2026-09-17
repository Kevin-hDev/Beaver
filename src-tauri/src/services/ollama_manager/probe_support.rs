use std::time::Instant;

use super::error::OllamaErrorCode;
use super::probe::TargetValidation;
use super::process::NativeGatedProcess;
use super::spawn_profile::OllamaSpawnProfile;

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

pub(crate) fn invalid_target() -> TargetValidation {
    TargetValidation::InvalidTarget {
        code: OllamaErrorCode::OllamaBundleInvalid,
    }
}

pub(crate) fn deferred(code: OllamaErrorCode) -> TargetValidation {
    TargetValidation::Deferred { code }
}

use super::super::emergency::VerifiedProcessIdentity;
use super::super::emergency_drain::EmergencyObservation;
use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity};

pub(super) fn signal_or_recheck(
    identity: VerifiedProcessIdentity,
    already_requested: bool,
) -> EmergencyObservation {
    if identity.pid < 2 || identity.native_scope > i32::MAX as u64 {
        return EmergencyObservation::IdentityMismatch;
    }
    if identity.executable == 0 {
        return EmergencyObservation::IdentityMismatch;
    }
    OwnedProcess::signal_exact(
        OwnedProcessIdentity {
            pid: identity.pid,
            native_scope: identity.native_scope,
            native_start_time: identity.started_at,
            executable: identity.executable,
        },
        already_requested,
    )
    .map(|()| EmergencyObservation::Terminating)
    .unwrap_or(EmergencyObservation::IdentityMismatch)
}

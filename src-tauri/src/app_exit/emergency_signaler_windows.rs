use super::super::emergency::VerifiedProcessIdentity;
use super::super::emergency_drain::EmergencyObservation;
use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity};

pub(super) fn signal_or_recheck(
    identity: VerifiedProcessIdentity,
    already_requested: bool,
) -> EmergencyObservation {
    if identity.pid < 2 || identity.executable == 0 {
        return EmergencyObservation::IdentityMismatch;
    }
    let expected = OwnedProcessIdentity {
        pid: identity.pid,
        native_scope: identity.native_scope,
        native_start_time: identity.started_at,
        executable: identity.executable,
    };
    if already_requested {
        if !OwnedProcess::process_exists(identity.pid) {
            EmergencyObservation::Exited
        } else {
            OwnedProcess::signal_exact(expected, true)
                .map(|()| EmergencyObservation::Terminating)
                .unwrap_or(EmergencyObservation::IdentityMismatch)
        }
    } else {
        OwnedProcess::signal_exact(expected, false)
            .map(|()| EmergencyObservation::Terminating)
            .unwrap_or(EmergencyObservation::IdentityMismatch)
    }
}

use super::super::emergency::VerifiedProcessIdentity;
use super::super::emergency_drain::EmergencyObservation;
use crate::services::owned_process;

pub(super) fn signal_or_recheck(
    identity: VerifiedProcessIdentity,
    already_requested: bool,
) -> EmergencyObservation {
    if identity.pid < 2 || identity.executable == 0 || !owned_process::is_confined(identity.pid) {
        return EmergencyObservation::IdentityMismatch;
    }
    let Ok(current) = owned_process::identity(identity.pid) else {
        return EmergencyObservation::IdentityMismatch;
    };
    if current.native_scope != identity.native_scope
        || current.native_start_time != identity.started_at
        || current.executable != identity.executable
    {
        return EmergencyObservation::IdentityMismatch;
    }
    if already_requested {
        if !owned_process::process_exists(identity.pid) {
            EmergencyObservation::Exited
        } else {
            owned_process::terminate_native(identity.pid);
            EmergencyObservation::Terminating
        }
    } else {
        owned_process::terminate_native(identity.pid);
        EmergencyObservation::Terminating
    }
}

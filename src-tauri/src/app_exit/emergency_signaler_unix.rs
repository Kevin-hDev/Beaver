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
    if identity.executable == 0 || !matches_identity(identity) {
        return EmergencyObservation::IdentityMismatch;
    }
    if already_requested {
        if !process_exists(identity.pid) {
            return EmergencyObservation::Exited;
        }
        unsafe {
            libc::kill(-(identity.native_scope as libc::pid_t), libc::SIGKILL);
            libc::kill(identity.pid as libc::pid_t, libc::SIGKILL);
        }
        EmergencyObservation::Terminating
    } else {
        unsafe {
            libc::kill(-(identity.native_scope as libc::pid_t), libc::SIGTERM);
            libc::kill(identity.pid as libc::pid_t, libc::SIGTERM);
        }
        EmergencyObservation::Terminating
    }
}

fn matches_identity(identity: VerifiedProcessIdentity) -> bool {
    OwnedProcess::identity_matches(OwnedProcessIdentity {
        pid: identity.pid,
        native_scope: identity.native_scope,
        native_start_time: identity.started_at,
        executable: identity.executable,
    })
    .is_ok()
}

fn process_exists(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

use super::emergency::{
    EmergencyInventory, VerifiedProcessIdentity, SLOT_CLAIMED, SLOT_FREE, SLOT_PUBLISHED,
    SLOT_REMOVE_PENDING,
};
use std::sync::atomic::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Exited is returned by native service signalers introduced in milestone 2"
    )
)]
pub(super) enum EmergencyObservation {
    Terminating,
    Exited,
    IdentityMismatch,
}

pub(super) trait EmergencySignaler: Send + Sync {
    fn signal_or_recheck(
        &self,
        identity: VerifiedProcessIdentity,
        already_requested: bool,
    ) -> EmergencyObservation;
}

impl EmergencyInventory {
    pub(super) fn drain_once(&self, signaler: &dyn EmergencySignaler) {
        for slot in &self.inner.slots {
            if slot
                .state
                .compare_exchange(
                    SLOT_PUBLISHED,
                    SLOT_CLAIMED,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_err()
            {
                continue;
            }
            let identity = VerifiedProcessIdentity {
                pid: slot.pid.load(Ordering::Relaxed),
                native_scope: slot.native_scope.load(Ordering::Relaxed),
                started_at: slot.started_at.load(Ordering::Relaxed),
            };
            let already_requested = slot.termination_requested.load(Ordering::Acquire);
            let observation = signaler.signal_or_recheck(identity, already_requested);
            if observation == EmergencyObservation::Terminating {
                slot.termination_requested.store(true, Ordering::Release);
            }
            finish_claim(slot, observation);
        }
    }
}

fn finish_claim(slot: &super::emergency::EmergencySlot, observation: EmergencyObservation) {
    let keep = observation == EmergencyObservation::Terminating;
    let target = if keep { SLOT_PUBLISHED } else { SLOT_FREE };
    match slot
        .state
        .compare_exchange(SLOT_CLAIMED, target, Ordering::AcqRel, Ordering::Acquire)
    {
        Ok(_) => {}
        Err(SLOT_REMOVE_PENDING) => {
            let _ = slot.state.compare_exchange(
                SLOT_REMOVE_PENDING,
                SLOT_FREE,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
        }
        Err(_) => {}
    }
}

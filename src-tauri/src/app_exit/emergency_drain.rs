use super::emergency::{
    EmergencyInventory, VerifiedProcessIdentity, SLOT_CLAIMED, SLOT_FREE, SLOT_PUBLISHED,
    SLOT_REMOVE_PENDING,
};
use std::sync::atomic::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum EmergencyObservation {
    Terminating,
    Exited,
    IdentityMismatch,
}

pub(crate) trait EmergencySignaler: Send + Sync {
    fn signal_or_recheck(
        &self,
        identity: VerifiedProcessIdentity,
        already_requested: bool,
    ) -> EmergencyObservation;
}

impl EmergencyInventory {
    pub(crate) fn drain_once(&self, signaler: &dyn EmergencySignaler) {
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
                executable: (u128::from(slot.executable_high.load(Ordering::Relaxed)) << 64)
                    | u128::from(slot.executable_low.load(Ordering::Relaxed)),
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

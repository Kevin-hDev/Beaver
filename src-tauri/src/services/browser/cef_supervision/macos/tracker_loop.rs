use super::super::constants::{CEF_SLOT_CAPACITY, CEF_TRACKER_POLL};
use super::super::mac_supervision_failure::MacSupervisionFailure;
use super::super::{CefPublication, CefSharedLayoutError};
use super::identity::MacProcessIdentity;
use super::liveness_policy::MacLivenessDecision;
use super::tracker::MacTrackerShared;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

pub(super) fn run_tracker(shared: Arc<MacTrackerShared>) {
    let mut active: [Option<ActiveHelper>; CEF_SLOT_CAPACITY] = std::array::from_fn(|_| None);
    while !shared.tracker_stopping.load(Ordering::Acquire) {
        if shared.failure().is_some() {
            shared.pending.drain();
        } else {
            scan_pending(&shared, &mut active);
        }
        refresh_active(&shared, &mut active);
        std::thread::park_timeout(CEF_TRACKER_POLL);
    }
    shared.pending.drain();
}

fn scan_pending(
    shared: &Arc<MacTrackerShared>,
    active: &mut [Option<ActiveHelper>; CEF_SLOT_CAPACITY],
) {
    for (slot, active_slot) in active.iter_mut().enumerate() {
        let Some(snapshot) = shared.pending.mailbox_snapshot(slot) else {
            continue;
        };
        match snapshot {
            Err(CefSharedLayoutError::Unpublished) => {
                let Some(pending) = shared.pending.take_if_expired(slot, Instant::now()) else {
                    continue;
                };
                match pending.objects.mailbox_snapshot() {
                    Err(CefSharedLayoutError::Unpublished) => {
                        let pending = *pending;
                        if pending.reservation.expire() {
                            ::log::warn!("[browser] CEF helper publication expired");
                        }
                        continue;
                    }
                    Err(_) => {
                        drop(pending);
                        shared.fail(MacSupervisionFailure::PendingLayout);
                        return;
                    }
                    Ok(_) => match admit(shared, *pending) {
                        Ok(helper) if active_slot.is_none() => *active_slot = Some(helper),
                        Ok(_) => {
                            shared.fail(MacSupervisionFailure::ActiveSlotOccupied);
                            return;
                        }
                        Err(failure) => {
                            shared.fail(failure);
                            return;
                        }
                    },
                }
            }
            Err(_) => {
                drop(shared.pending.take(slot));
                shared.fail(MacSupervisionFailure::PendingLayout);
                return;
            }
            Ok(_) => {
                let Some(pending) = shared.pending.take(slot) else {
                    shared.fail(MacSupervisionFailure::PendingMissing);
                    continue;
                };
                match admit(shared, *pending) {
                    Ok(helper) if active_slot.is_none() => *active_slot = Some(helper),
                    Ok(_) => {
                        shared.fail(MacSupervisionFailure::ActiveSlotOccupied);
                        return;
                    }
                    Err(failure) => {
                        shared.fail(failure);
                        return;
                    }
                }
            }
        }
    }
}

fn admit(
    shared: &Arc<MacTrackerShared>,
    pending: super::pending::MacPendingLaunch,
) -> Result<ActiveHelper, MacSupervisionFailure> {
    let snapshot = pending
        .objects
        .mailbox_snapshot()
        .map_err(|_| MacSupervisionFailure::MailboxSnapshot)?;
    if snapshot.generation != pending.reservation.marker().generation() {
        return Err(MacSupervisionFailure::GenerationMismatch);
    }
    let publication = CefPublication::from_marker(pending.reservation.marker(), snapshot.pid)
        .map_err(|_| MacSupervisionFailure::Publication)?;
    let claim = shared
        .table
        .claim(&publication)
        .map_err(|_| MacSupervisionFailure::AuthorityClaim)?;
    let identity = MacProcessIdentity::validate(
        snapshot.pid,
        shared.parent_pid,
        snapshot.started_at,
        snapshot.native_group,
        &shared.expected_executables,
    )
    .map_err(|_| MacSupervisionFailure::Identity)?;
    let admission = claim
        .admit()
        .map_err(|_| MacSupervisionFailure::AuthorityAdmission)?;
    let slot = pending.reservation.marker().slot();
    let generation = pending.reservation.marker().generation();
    shared
        .emergency
        .install(
            slot,
            generation,
            identity,
            Arc::clone(&pending.objects),
            admission,
        )
        .map_err(|_| MacSupervisionFailure::EmergencyInstall)?;
    pending.objects.signal_admission();
    Ok(ActiveHelper { slot, generation })
}

fn refresh_active(
    shared: &Arc<MacTrackerShared>,
    active: &mut [Option<ActiveHelper>; CEF_SLOT_CAPACITY],
) {
    for helper in active.iter_mut() {
        let Some(current) = helper.as_ref() else {
            continue;
        };
        match shared.emergency.refresh(current.slot, current.generation) {
            Ok(Some(MacLivenessDecision::Alive | MacLivenessDecision::Pending)) => {}
            Ok(Some(MacLivenessDecision::Stopped)) | Ok(None) => drop(helper.take()),
            Ok(Some(MacLivenessDecision::Expired)) | Err(_) => {
                shared.fail(MacSupervisionFailure::Liveness)
            }
        }
    }
}

struct ActiveHelper {
    slot: usize,
    generation: u64,
}

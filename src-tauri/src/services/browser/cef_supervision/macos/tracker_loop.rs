use super::super::constants::{CEF_SLOT_CAPACITY, CEF_TRACKER_POLL};
use super::super::{CefPublication, CefSharedLayoutError, CefUnavailableCategory};
use super::identity::MacProcessIdentity;
use super::tracker::MacTrackerShared;
use super::MacPublicationObjects;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub(super) fn run_tracker(shared: Arc<MacTrackerShared>) {
    let mut active: [Option<ActiveHelper>; CEF_SLOT_CAPACITY] = std::array::from_fn(|_| None);
    while !shared.stopping.load(Ordering::Acquire) {
        if shared.force_requested.load(Ordering::Acquire) {
            shared.pending.drain();
            terminate_all(&shared, &mut active);
        } else if shared.failure().is_some() {
            shared.pending.drain();
        } else {
            scan_pending(&shared, &mut active);
        }
        refresh_active(&shared, &mut active);
        shared.active_count.store(
            active.iter().filter(|item| item.is_some()).count(),
            Ordering::Release,
        );
        std::thread::park_timeout(CEF_TRACKER_POLL);
    }
    shared.pending.drain();
    terminate_all(&shared, &mut active);
    shared.active_count.store(0, Ordering::Release);
}

fn terminate_all(
    shared: &MacTrackerShared,
    active: &mut [Option<ActiveHelper>; CEF_SLOT_CAPACITY],
) {
    for helper in active.iter_mut().filter_map(Option::take) {
        if helper.terminate().is_err() {
            shared.fail(CefUnavailableCategory::Reaper);
        }
    }
}

fn scan_pending(
    shared: &Arc<MacTrackerShared>,
    active: &mut [Option<ActiveHelper>; CEF_SLOT_CAPACITY],
) {
    for (slot, active_slot) in active.iter_mut().enumerate() {
        let Some(pending) = shared.pending.peek(slot) else {
            continue;
        };
        match pending.objects.mailbox_snapshot() {
            Err(CefSharedLayoutError::Unpublished) => continue,
            Err(_) => {
                drop(shared.pending.take(slot));
                shared.fail(CefUnavailableCategory::Admission);
                return;
            }
            Ok(_) => {
                let Some(pending) = shared.pending.take(slot) else {
                    shared.fail(CefUnavailableCategory::Admission);
                    continue;
                };
                match admit(shared, *pending) {
                    Ok(helper) if active_slot.is_none() => *active_slot = Some(helper),
                    Ok(_) | Err(_) => {
                        shared.fail(CefUnavailableCategory::Admission);
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
) -> Result<ActiveHelper, CefUnavailableCategory> {
    let snapshot = pending
        .objects
        .mailbox_snapshot()
        .map_err(|_| CefUnavailableCategory::Admission)?;
    if snapshot.generation != pending.reservation.marker().generation() {
        return Err(CefUnavailableCategory::Admission);
    }
    let publication = CefPublication::from_marker(pending.reservation.marker(), snapshot.pid)
        .map_err(|_| CefUnavailableCategory::Admission)?;
    let claim = shared
        .table
        .claim(&publication)
        .map_err(|_| CefUnavailableCategory::Admission)?;
    let identity = MacProcessIdentity::validate(
        snapshot.pid,
        shared.parent_pid,
        snapshot.started_at,
        snapshot.native_group,
        &shared.expected_executable,
    )?;
    let admission = claim
        .admit()
        .map_err(|_| CefUnavailableCategory::Admission)?;
    pending.objects.signal_admission();
    Ok(ActiveHelper {
        _objects: pending.objects,
        identity,
        _admission: admission,
    })
}

fn refresh_active(
    shared: &Arc<MacTrackerShared>,
    active: &mut [Option<ActiveHelper>; CEF_SLOT_CAPACITY],
) {
    for helper in active.iter_mut() {
        let Some(current) = helper.as_ref() else {
            continue;
        };
        match current.identity.is_alive() {
            Ok(true) => {}
            Ok(false) => drop(helper.take()),
            Err(_) => shared.fail(CefUnavailableCategory::Reaper),
        }
    }
}

struct ActiveHelper {
    _objects: MacPublicationObjects,
    identity: MacProcessIdentity,
    _admission: super::super::reservation::CefAdmission,
}

impl ActiveHelper {
    fn terminate(&self) -> Result<(), CefUnavailableCategory> {
        if self.identity.is_alive()? {
            self.identity.kill_group()?;
        }
        Ok(())
    }
}

impl Drop for ActiveHelper {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

use super::super::constants::{CEF_SLOT_CAPACITY, CEF_TRACKER_POLL};
use super::super::{CefPublication, CefSharedLayoutError, CefUnavailableCategory};
use super::confinement::WindowsConfinement;
use super::native_authority::{WindowsTerminationState, WindowsTrackedAdmission};
use super::objects::WindowsPublicationObjects;
use super::tracker::WindowsTrackerShared;
use super::tracker_pending::WindowsPendingLaunch;
use super::WindowsProcessIdentity;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub(super) fn run_tracker(shared: Arc<WindowsTrackerShared>) {
    let mut active: [Option<ActiveHelper>; CEF_SLOT_CAPACITY] = std::array::from_fn(|_| None);
    while !shared.stopping.load(Ordering::Acquire) {
        if shared.failure().is_some() {
            shared.pending.drain();
        } else {
            scan_pending(&shared, &mut active);
        }
        refresh_active(&shared, &mut active);
        std::thread::park_timeout(CEF_TRACKER_POLL);
    }
    shared.pending.drain();
    drop(active);
    for _ in 0..50 {
        match shared.native.refresh_all() {
            Ok(0) => break,
            Ok(_) => std::thread::park_timeout(CEF_TRACKER_POLL),
            Err(_) => {
                shared.fail(CefUnavailableCategory::Reaper);
                break;
            }
        }
    }
}

fn scan_pending(
    shared: &Arc<WindowsTrackerShared>,
    active: &mut [Option<ActiveHelper>; CEF_SLOT_CAPACITY],
) {
    for slot in 0..CEF_SLOT_CAPACITY {
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
                match admit_pending(shared, pending) {
                    Ok(helper) if active[slot].is_none() => active[slot] = Some(helper),
                    Ok(_) | Err(_) => {
                        shared.fail(CefUnavailableCategory::Admission);
                        return;
                    }
                }
            }
        }
    }
}

fn admit_pending(
    shared: &Arc<WindowsTrackerShared>,
    pending: Box<WindowsPendingLaunch>,
) -> Result<ActiveHelper, CefUnavailableCategory> {
    let snapshot = pending
        .objects
        .mailbox_snapshot()
        .map_err(|_| CefUnavailableCategory::Admission)?;
    if snapshot.generation != pending.reservation.marker().generation()
        || snapshot.native_group != 0
    {
        return Err(CefUnavailableCategory::Admission);
    }
    let publication = CefPublication::from_marker(pending.reservation.marker(), snapshot.pid)
        .map_err(|_| CefUnavailableCategory::Admission)?;
    let claim = shared
        .table
        .claim(&publication)
        .map_err(|_| CefUnavailableCategory::Admission)?;
    let identity = WindowsProcessIdentity::acquire(
        snapshot.pid,
        shared.parent_pid,
        snapshot.started_at,
        &shared.expected_executable,
    )?;
    let confinement = WindowsConfinement::establish(identity)?;
    let native = shared.native.prepare(&claim, confinement)?;
    let admission = native.admit(claim)?;
    pending.objects.signal_admission()?;
    Ok(ActiveHelper {
        _objects: pending.objects,
        admission,
    })
}

fn refresh_active(
    shared: &Arc<WindowsTrackerShared>,
    active: &mut [Option<ActiveHelper>; CEF_SLOT_CAPACITY],
) {
    for helper in active.iter_mut() {
        let Some(current) = helper.as_ref() else {
            continue;
        };
        match current.admission.observe() {
            Ok(WindowsTerminationState::Exited) => drop(helper.take()),
            Ok(_) => {}
            Err(_) => shared.fail(CefUnavailableCategory::Reaper),
        }
    }
}

struct ActiveHelper {
    _objects: WindowsPublicationObjects,
    admission: WindowsTrackedAdmission,
}

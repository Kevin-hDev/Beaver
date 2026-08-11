use super::super::constants::CEF_SLOT_CAPACITY;
use super::super::reservation::CefReservation;
use super::super::shared_layout::CefMailboxSnapshot;
use super::super::{CefSharedLayoutError, CefUnavailableCategory};
use super::MacPublicationObjects;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

pub(super) struct MacPendingLaunch {
    pub(super) reservation: CefReservation,
    pub(super) objects: Arc<MacPublicationObjects>,
    pub(super) expires_at: Instant,
}

pub(super) struct MacPendingSlots {
    slots: [Mutex<Option<MacPendingLaunch>>; CEF_SLOT_CAPACITY],
}

impl MacPendingSlots {
    pub(super) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| Mutex::new(None)),
        }
    }

    pub(super) fn install(
        &self,
        slot: usize,
        pending: MacPendingLaunch,
    ) -> Result<(), CefUnavailableCategory> {
        let mut target = self.lock(slot).ok_or(CefUnavailableCategory::Admission)?;
        if target.is_some() {
            return Err(CefUnavailableCategory::Admission);
        }
        *target = Some(pending);
        Ok(())
    }

    pub(super) fn mailbox_snapshot(
        &self,
        slot: usize,
    ) -> Option<Result<CefMailboxSnapshot, CefSharedLayoutError>> {
        let target = self.lock(slot)?;
        target
            .as_ref()
            .map(|pending| pending.objects.mailbox_snapshot())
    }

    pub(super) fn take(&self, slot: usize) -> Option<Box<MacPendingLaunch>> {
        self.lock(slot)?.take().map(Box::new)
    }

    pub(super) fn take_if_expired(
        &self,
        slot: usize,
        now: Instant,
    ) -> Option<Box<MacPendingLaunch>> {
        let mut target = self.lock(slot)?;
        if target
            .as_ref()
            .is_none_or(|pending| pending.expires_at > now)
        {
            return None;
        }
        target.take().map(Box::new)
    }

    pub(super) fn begin_closing(&self, deadline_ticks: u64) -> Result<(), CefSharedLayoutError> {
        let mut failed = false;
        for slot in 0..CEF_SLOT_CAPACITY {
            let Some(target) = self.lock(slot) else {
                failed = true;
                continue;
            };
            if target
                .as_ref()
                .is_some_and(|pending| pending.objects.begin_closing(deadline_ticks).is_err())
            {
                failed = true;
            }
        }
        (!failed).then_some(()).ok_or(CefSharedLayoutError::Invalid)
    }

    pub(super) fn drain(&self) {
        for slot in 0..CEF_SLOT_CAPACITY {
            drop(self.take(slot));
        }
    }

    fn lock(&self, slot: usize) -> Option<MutexGuard<'_, Option<MacPendingLaunch>>> {
        self.slots
            .get(slot)
            .map(|slot| slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }
}

impl Drop for MacPendingSlots {
    fn drop(&mut self) {
        self.drain();
    }
}

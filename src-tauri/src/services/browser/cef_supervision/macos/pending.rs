use super::super::constants::CEF_SLOT_CAPACITY;
use super::super::reservation::CefReservation;
use super::super::CefUnavailableCategory;
use super::MacPublicationObjects;
use std::sync::atomic::{AtomicPtr, Ordering};

pub(super) struct MacPendingLaunch {
    pub(super) reservation: CefReservation,
    pub(super) objects: MacPublicationObjects,
}

pub(super) struct MacPendingSlots {
    slots: [AtomicPtr<MacPendingLaunch>; CEF_SLOT_CAPACITY],
}

impl MacPendingSlots {
    pub(super) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| AtomicPtr::new(std::ptr::null_mut())),
        }
    }

    pub(super) fn install(
        &self,
        slot: usize,
        pending: MacPendingLaunch,
    ) -> Result<(), CefUnavailableCategory> {
        let target = self
            .slots
            .get(slot)
            .ok_or(CefUnavailableCategory::Admission)?;
        let raw = Box::into_raw(Box::new(pending));
        if target
            .compare_exchange(
                std::ptr::null_mut(),
                raw,
                Ordering::Release,
                Ordering::Acquire,
            )
            .is_err()
        {
            unsafe { drop(Box::from_raw(raw)) };
            Err(CefUnavailableCategory::Admission)
        } else {
            Ok(())
        }
    }

    pub(super) fn peek(&self, slot: usize) -> Option<&MacPendingLaunch> {
        let pointer = self.slots.get(slot)?.load(Ordering::Acquire);
        (!pointer.is_null()).then(|| unsafe { &*pointer })
    }

    pub(super) fn take(&self, slot: usize) -> Option<Box<MacPendingLaunch>> {
        let pointer = self
            .slots
            .get(slot)?
            .swap(std::ptr::null_mut(), Ordering::AcqRel);
        (!pointer.is_null()).then(|| unsafe { Box::from_raw(pointer) })
    }

    pub(super) fn drain(&self) {
        for slot in 0..CEF_SLOT_CAPACITY {
            drop(self.take(slot));
        }
    }
}

impl Drop for MacPendingSlots {
    fn drop(&mut self) {
        self.drain();
    }
}

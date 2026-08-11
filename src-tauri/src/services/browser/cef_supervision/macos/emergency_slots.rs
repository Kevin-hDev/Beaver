use super::super::constants::CEF_SLOT_CAPACITY;
use super::super::reservation::CefAdmission;
use super::super::CefUnavailableCategory;
use super::identity::MacProcessIdentity;
use super::MacPublicationObjects;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

struct MacEmergencyEntry {
    generation: u64,
    identity: MacProcessIdentity,
    objects: Arc<MacPublicationObjects>,
    _admission: CefAdmission,
}

#[derive(Clone)]
pub(super) struct MacEmergencyTarget {
    pub(super) slot: usize,
    pub(super) generation: u64,
    pub(super) identity: MacProcessIdentity,
    objects: Arc<MacPublicationObjects>,
}

pub(super) struct MacEmergencySlots {
    slots: [RwLock<Option<MacEmergencyEntry>>; CEF_SLOT_CAPACITY],
    occupied: AtomicUsize,
    closing_deadline: AtomicU64,
}

impl MacEmergencySlots {
    pub(super) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| RwLock::new(None)),
            occupied: AtomicUsize::new(0),
            closing_deadline: AtomicU64::new(0),
        }
    }

    pub(super) fn install(
        &self,
        slot: usize,
        generation: u64,
        identity: MacProcessIdentity,
        objects: Arc<MacPublicationObjects>,
        admission: CefAdmission,
    ) -> Result<(), CefUnavailableCategory> {
        if generation == 0 || self.closing_deadline.load(Ordering::Acquire) != 0 {
            return Err(CefUnavailableCategory::Admission);
        }
        let mut target = self.write(slot).ok_or(CefUnavailableCategory::Admission)?;
        if target.is_some() || self.closing_deadline.load(Ordering::Acquire) != 0 {
            return Err(CefUnavailableCategory::Admission);
        }
        *target = Some(MacEmergencyEntry {
            generation,
            identity,
            objects,
            _admission: admission,
        });
        self.occupied.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    pub(super) fn begin_closing(&self, deadline_ticks: u64) -> Result<(), ()> {
        if deadline_ticks == 0 {
            return Err(());
        }
        let stored = match self.closing_deadline.compare_exchange(
            0,
            deadline_ticks,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => deadline_ticks,
            Err(existing) if existing != 0 => existing,
            Err(_) => return Err(()),
        };
        let mut failed = false;
        for slot in 0..CEF_SLOT_CAPACITY {
            if self
                .target(slot)
                .is_some_and(|target| target.objects.begin_closing(stored).is_err())
            {
                failed = true;
            }
        }
        (!failed).then_some(()).ok_or(())
    }

    pub(super) fn force_pass(&self) -> Result<(), ()> {
        let mut failed = false;
        for slot in 0..CEF_SLOT_CAPACITY {
            let Some(target) = self.target(slot) else {
                continue;
            };
            match target.identity.is_alive() {
                Ok(false) => self.clear(target.slot, target.generation),
                Ok(true) => {
                    if target.identity.kill_group().is_err() {
                        failed = true;
                    }
                }
                Err(_) => failed = true,
            }
        }
        (!failed).then_some(()).ok_or(())
    }

    pub(super) fn clear(&self, slot: usize, generation: u64) {
        let Some(mut target) = self.write(slot) else {
            return;
        };
        if target
            .as_ref()
            .is_some_and(|entry| entry.generation == generation)
        {
            drop(target.take());
            self.occupied.fetch_sub(1, Ordering::AcqRel);
        }
    }

    pub(super) fn has_entries(&self) -> bool {
        self.occupied.load(Ordering::Acquire) != 0
    }

    fn target(&self, slot: usize) -> Option<MacEmergencyTarget> {
        let target = self.read(slot)?;
        target.as_ref().map(|entry| MacEmergencyTarget {
            slot,
            generation: entry.generation,
            identity: entry.identity.clone(),
            objects: Arc::clone(&entry.objects),
        })
    }

    fn read(&self, slot: usize) -> Option<RwLockReadGuard<'_, Option<MacEmergencyEntry>>> {
        self.slots
            .get(slot)
            .map(|slot| slot.read().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    fn write(&self, slot: usize) -> Option<RwLockWriteGuard<'_, Option<MacEmergencyEntry>>> {
        self.slots.get(slot).map(|slot| {
            slot.write()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        })
    }
}

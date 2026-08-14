use super::super::constants::CEF_SLOT_CAPACITY;
use super::super::reservation::CefAdmission;
use super::super::CefUnavailableCategory;
use super::identity::MacProcessIdentity;
use super::liveness_policy::{MacLivenessDecision, MacLivenessState};
use super::process_state::{MacProcessActions, MacProcessObservation, MacSystemProcessActions};
use super::MacPublicationObjects;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

struct MacEmergencyEntry {
    generation: u64,
    identity: MacProcessIdentity,
    objects: Arc<MacPublicationObjects>,
    _admission: CefAdmission,
    liveness: MacLivenessState,
}

pub(super) struct MacEmergencySlots {
    slots: [RwLock<Option<MacEmergencyEntry>>; CEF_SLOT_CAPACITY],
    occupied: AtomicUsize,
    closing: OnceLock<MacClosingDeadlines>,
}

#[derive(Clone, Copy)]
struct MacClosingDeadlines {
    helper_exit: u64,
    ultimate: u64,
}

impl MacEmergencySlots {
    pub(super) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| RwLock::new(None)),
            occupied: AtomicUsize::new(0),
            closing: OnceLock::new(),
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
        if generation == 0 || self.closing.get().is_some() {
            return Err(CefUnavailableCategory::Admission);
        }
        let mut target = self.write(slot).ok_or(CefUnavailableCategory::Admission)?;
        if target.is_some() || self.closing.get().is_some() {
            return Err(CefUnavailableCategory::Admission);
        }
        *target = Some(MacEmergencyEntry {
            generation,
            identity,
            objects,
            _admission: admission,
            liveness: MacLivenessState::new(),
        });
        self.occupied.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    pub(super) fn begin_closing(
        &self,
        helper_exit_ticks: u64,
        ultimate_ticks: u64,
    ) -> Result<(), ()> {
        if helper_exit_ticks == 0 || ultimate_ticks == 0 || helper_exit_ticks > ultimate_ticks {
            return Err(());
        }
        let stored = *self.closing.get_or_init(|| MacClosingDeadlines {
            helper_exit: helper_exit_ticks,
            ultimate: ultimate_ticks,
        });
        let mut failed = false;
        for slot in 0..CEF_SLOT_CAPACITY {
            if self
                .objects(slot)
                .is_some_and(|objects| objects.begin_closing(stored.helper_exit).is_err())
            {
                failed = true;
            }
        }
        (!failed).then_some(()).ok_or(())
    }

    pub(super) fn refresh(
        &self,
        slot: usize,
        generation: u64,
    ) -> Result<Option<MacLivenessDecision>, CefUnavailableCategory> {
        let mut target = match self.write(slot) {
            Some(target) => target,
            None => return Ok(None),
        };
        let Some(entry) = target
            .as_mut()
            .filter(|entry| entry.generation == generation)
        else {
            return Ok(None);
        };
        let observation = MacSystemProcessActions.observe(&entry.identity);
        let now_ticks = super::clock::now_ticks()?;
        self.apply_observation(&mut target, observation, now_ticks)
    }

    fn apply_observation(
        &self,
        target: &mut Option<MacEmergencyEntry>,
        observation: MacProcessObservation,
        now_ticks: u64,
    ) -> Result<Option<MacLivenessDecision>, CefUnavailableCategory> {
        let Some(entry) = target.as_mut() else {
            return Ok(None);
        };
        let decision = entry
            .liveness
            .apply(
                observation,
                now_ticks,
                self.closing.get().map(|deadlines| deadlines.ultimate),
            )
            .map_err(|_| CefUnavailableCategory::Reaper)?;
        if decision == MacLivenessDecision::Stopped {
            drop(target.take());
            self.occupied.fetch_sub(1, Ordering::AcqRel);
        }
        Ok(Some(decision))
    }

    #[cfg(test)]
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

    fn objects(&self, slot: usize) -> Option<Arc<MacPublicationObjects>> {
        let target = self.read(slot)?;
        target.as_ref().map(|entry| Arc::clone(&entry.objects))
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

mod force;
#[cfg(test)]
mod test_api;

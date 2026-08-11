use super::super::constants::CEF_SLOT_CAPACITY;
use super::super::CefUnavailableCategory;
use super::objects::WindowsPublicationObjects;
use std::sync::{Arc, Mutex};

struct WindowsEmergencyEntry {
    generation: u64,
    objects: Arc<WindowsPublicationObjects>,
}

pub(super) struct WindowsEmergencySlots {
    // Cette table est l'unique propriétaire d'urgence des objets natifs : la
    // fermeture ne dépend donc jamais de la progression du fil de suivi.
    slots: [Mutex<Option<WindowsEmergencyEntry>>; CEF_SLOT_CAPACITY],
}

impl WindowsEmergencySlots {
    pub(super) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| Mutex::new(None)),
        }
    }

    pub(super) fn install(
        self: &Arc<Self>,
        slot: usize,
        generation: u64,
        objects: Arc<WindowsPublicationObjects>,
    ) -> Result<WindowsEmergencyRegistration, CefUnavailableCategory> {
        if generation == 0 {
            return Err(CefUnavailableCategory::Admission);
        }
        let target = self
            .slots
            .get(slot)
            .ok_or(CefUnavailableCategory::Admission)?;
        let mut entry = lock(target);
        if entry.is_some() {
            return Err(CefUnavailableCategory::Admission);
        }
        *entry = Some(WindowsEmergencyEntry {
            generation,
            objects,
        });
        Ok(WindowsEmergencyRegistration {
            slots: Arc::clone(self),
            slot,
            generation,
        })
    }

    pub(super) fn begin_closing(&self, deadline_ticks: u64) -> Result<(), CefUnavailableCategory> {
        let mut first_error = None;
        for target in &self.slots {
            let entry = lock(target);
            if let Some(entry) = entry.as_ref() {
                if let Err(error) = entry.objects.begin_closing(deadline_ticks) {
                    first_error.get_or_insert(error);
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    fn clear(&self, slot: usize, generation: u64) {
        let Some(target) = self.slots.get(slot) else {
            return;
        };
        let mut entry = lock(target);
        if entry
            .as_ref()
            .is_some_and(|current| current.generation == generation)
        {
            *entry = None;
        }
    }
}

pub(super) struct WindowsEmergencyRegistration {
    slots: Arc<WindowsEmergencySlots>,
    slot: usize,
    generation: u64,
}

impl Drop for WindowsEmergencyRegistration {
    fn drop(&mut self) {
        self.slots.clear(self.slot, self.generation);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

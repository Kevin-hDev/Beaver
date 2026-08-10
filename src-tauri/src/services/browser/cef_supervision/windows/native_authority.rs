use super::super::constants::CEF_SLOT_CAPACITY;
use super::super::reservation::{CefAdmission, CefClaim};
use super::super::CefUnavailableCategory;
use super::confinement::WindowsConfinement;
use super::native_slot::WindowsNativeSlot;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::browser) enum WindowsTerminationState {
    Admitted,
    Terminating,
    Exited,
}

pub(in crate::services::browser) fn classify_termination(
    shutdown_requested: bool,
    process_signaled: bool,
) -> WindowsTerminationState {
    if process_signaled {
        WindowsTerminationState::Exited
    } else if shutdown_requested {
        WindowsTerminationState::Terminating
    } else {
        WindowsTerminationState::Admitted
    }
}

pub(in crate::services::browser) struct WindowsNativeAuthority {
    slots: [WindowsNativeSlot; CEF_SLOT_CAPACITY],
}

impl WindowsNativeAuthority {
    pub(in crate::services::browser) fn new() -> Arc<Self> {
        Arc::new(Self {
            slots: std::array::from_fn(|_| WindowsNativeSlot::new()),
        })
    }

    pub(in crate::services::browser) fn prepare(
        self: &Arc<Self>,
        claim: &CefClaim,
        confinement: WindowsConfinement,
    ) -> Result<WindowsPendingAdmission, CefUnavailableCategory> {
        let slot = claim.slot();
        let generation = claim.generation();
        self.slots
            .get(slot)
            .ok_or(CefUnavailableCategory::Admission)?
            .prepare(generation, confinement)?;
        Ok(WindowsPendingAdmission {
            authority: Arc::clone(self),
            slot,
            generation,
            active: true,
        })
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn terminate(
        &self,
        slot: usize,
        generation: u64,
    ) -> Result<WindowsTerminationState, CefUnavailableCategory> {
        self.slot(slot)?.inspect(generation, true)
    }

    pub(in crate::services::browser) fn observe(
        &self,
        slot: usize,
        generation: u64,
    ) -> Result<WindowsTerminationState, CefUnavailableCategory> {
        self.slot(slot)?.inspect(generation, false)
    }

    pub(in crate::services::browser) fn refresh_all(
        &self,
    ) -> Result<usize, CefUnavailableCategory> {
        for slot in &self.slots {
            slot.refresh()?;
        }
        Ok(self.slots.iter().filter(|slot| slot.is_occupied()).count())
    }

    pub(super) fn force_all(&self) -> Result<(), CefUnavailableCategory> {
        let mut first_error = None;
        for slot in &self.slots {
            if let Err(error) = slot.force_current() {
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    pub(in crate::services::browser) fn occupied_slots(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_occupied()).count()
    }

    fn slot(&self, slot: usize) -> Result<&WindowsNativeSlot, CefUnavailableCategory> {
        self.slots
            .get(slot)
            .ok_or(CefUnavailableCategory::Admission)
    }
}

impl Drop for WindowsNativeAuthority {
    fn drop(&mut self) {
        for slot in &self.slots {
            slot.force_close();
        }
    }
}

pub(in crate::services::browser) struct WindowsPendingAdmission {
    authority: Arc<WindowsNativeAuthority>,
    slot: usize,
    generation: u64,
    active: bool,
}

impl WindowsPendingAdmission {
    pub(in crate::services::browser) fn admit(
        mut self,
        claim: CefClaim,
    ) -> Result<WindowsTrackedAdmission, CefUnavailableCategory> {
        if claim.slot() != self.slot || claim.generation() != self.generation {
            return Err(CefUnavailableCategory::Admission);
        }
        let admission = claim
            .admit()
            .map_err(|_| CefUnavailableCategory::Admission)?;
        self.authority
            .slot(self.slot)?
            .mark_admitted(self.generation)?;
        self.active = false;
        Ok(WindowsTrackedAdmission {
            authority: Arc::clone(&self.authority),
            slot: self.slot,
            generation: self.generation,
            _admission: admission,
        })
    }
}

impl Drop for WindowsPendingAdmission {
    fn drop(&mut self) {
        if self.active {
            if let Ok(slot) = self.authority.slot(self.slot) {
                slot.release(self.generation);
            }
        }
    }
}

pub(in crate::services::browser) struct WindowsTrackedAdmission {
    authority: Arc<WindowsNativeAuthority>,
    slot: usize,
    generation: u64,
    _admission: CefAdmission,
}

impl WindowsTrackedAdmission {
    #[cfg(test)]
    pub(in crate::services::browser) fn terminate(
        &self,
    ) -> Result<WindowsTerminationState, CefUnavailableCategory> {
        self.authority.terminate(self.slot, self.generation)
    }

    pub(in crate::services::browser) fn observe(
        &self,
    ) -> Result<WindowsTerminationState, CefUnavailableCategory> {
        self.authority.observe(self.slot, self.generation)
    }
}

impl Drop for WindowsTrackedAdmission {
    fn drop(&mut self) {
        if let Ok(slot) = self.authority.slot(self.slot) {
            slot.release(self.generation);
        }
    }
}

impl std::fmt::Debug for WindowsTrackedAdmission {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WindowsTrackedAdmission([redacted])")
    }
}

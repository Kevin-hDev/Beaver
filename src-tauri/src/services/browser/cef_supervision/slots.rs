use super::authority_slot::{
    CefAuthoritySlot, SLOT_ADMITTED, SLOT_CLAIMING, SLOT_FREE, SLOT_PUBLISHED, SLOT_RESERVED,
    SLOT_WRITING,
};
use super::constants::CEF_SLOT_CAPACITY;
use super::gate::CefLaunchGate;
use super::reservation::{CefClaim, CefReservation};
use super::{CefLaunchMarker, CefProcessRole, CefPublication};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::browser) enum CefTableError {
    Capacity,
    Closed,
    Invalid,
    Stale,
}

#[derive(Clone, Copy)]
pub(super) struct CefSlotKey {
    index: usize,
    generation: u64,
}

impl CefSlotKey {
    #[cfg(target_os = "windows")]
    pub(super) fn index(self) -> usize {
        self.index
    }

    #[cfg(target_os = "windows")]
    pub(super) fn generation(self) -> u64 {
        self.generation
    }
}

pub(super) struct CefAuthorityInner {
    gate: CefLaunchGate,
    slots: [CefAuthoritySlot; CEF_SLOT_CAPACITY],
}

impl CefAuthorityInner {
    pub(super) fn release(&self, key: CefSlotKey, expected: u8) -> bool {
        self.slots
            .get(key.index)
            .is_some_and(|slot| slot.clear_if(key.generation, expected))
    }

    pub(super) fn admit(&self, key: CefSlotKey) -> Result<(), CefTableError> {
        let _permit = self.gate.try_enter().map_err(|_| CefTableError::Closed)?;
        let slot = self.slots.get(key.index).ok_or(CefTableError::Invalid)?;
        if self.gate.is_closed() || slot.generation.load(Ordering::Acquire) != key.generation {
            return Err(CefTableError::Closed);
        }
        slot.state
            .compare_exchange(
                SLOT_PUBLISHED,
                SLOT_ADMITTED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| CefTableError::Stale)
    }
}

#[derive(Clone)]
pub(in crate::services::browser) struct CefAuthorityTable {
    inner: Arc<CefAuthorityInner>,
}

impl CefAuthorityTable {
    pub(super) fn new() -> Self {
        Self {
            inner: Arc::new(CefAuthorityInner {
                gate: CefLaunchGate::new(),
                slots: std::array::from_fn(|_| CefAuthoritySlot::new()),
            }),
        }
    }

    pub(super) fn try_reserve(
        &self,
        role: CefProcessRole,
    ) -> Result<CefReservation, CefTableError> {
        let _permit = self
            .inner
            .gate
            .try_enter()
            .map_err(|_| CefTableError::Closed)?;
        for (index, slot) in self.inner.slots.iter().enumerate() {
            if slot
                .state
                .compare_exchange(SLOT_FREE, SLOT_WRITING, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                continue;
            }
            let generation = next_generation(slot.generation.load(Ordering::Relaxed));
            let marker = match CefLaunchMarker::generate(index, generation, role) {
                Ok(marker) => marker,
                Err(_) => {
                    slot.clear_if(slot.generation.load(Ordering::Relaxed), SLOT_WRITING);
                    return Err(CefTableError::Invalid);
                }
            };
            slot.generation.store(generation, Ordering::Relaxed);
            slot.write_marker(&marker);
            if self.inner.gate.is_closed() {
                slot.clear_if(generation, SLOT_WRITING);
                return Err(CefTableError::Closed);
            }
            slot.state.store(SLOT_RESERVED, Ordering::Release);
            return Ok(CefReservation {
                table: Arc::clone(&self.inner),
                key: Some(CefSlotKey { index, generation }),
                marker,
            });
        }
        Err(CefTableError::Capacity)
    }

    pub(super) fn claim(&self, publication: &CefPublication) -> Result<CefClaim, CefTableError> {
        let _permit = self
            .inner
            .gate
            .try_enter()
            .map_err(|_| CefTableError::Closed)?;
        let slot = self
            .inner
            .slots
            .get(publication.slot)
            .ok_or(CefTableError::Invalid)?;
        if slot.state.load(Ordering::Acquire) != SLOT_RESERVED || !slot.matches(publication) {
            return Err(CefTableError::Stale);
        }
        slot.state
            .compare_exchange(
                SLOT_RESERVED,
                SLOT_CLAIMING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| CefTableError::Stale)?;
        if self.inner.gate.is_closed() {
            slot.clear_if(publication.generation, SLOT_CLAIMING);
            return Err(CefTableError::Closed);
        }
        slot.pid.store(publication.pid, Ordering::Relaxed);
        slot.state.store(SLOT_PUBLISHED, Ordering::Release);
        Ok(CefClaim::new(
            Arc::clone(&self.inner),
            CefSlotKey {
                index: publication.slot,
                generation: publication.generation,
            },
        ))
    }

    pub(super) fn close_and_invalidate(&self, deadline: Instant) -> bool {
        let drained = self.inner.gate.close_and_wait(deadline);
        for slot in &self.inner.slots {
            let generation = slot.generation.load(Ordering::Acquire);
            for state in [SLOT_WRITING, SLOT_RESERVED, SLOT_CLAIMING, SLOT_PUBLISHED] {
                if slot.clear_if(generation, state) {
                    break;
                }
            }
        }
        drained
    }
}

impl Default for CefAuthorityTable {
    fn default() -> Self {
        Self::new()
    }
}

fn next_generation(current: u64) -> u64 {
    match current.wrapping_add(1) {
        0 => 1,
        generation => generation,
    }
}

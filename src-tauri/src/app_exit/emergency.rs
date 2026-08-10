#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "service process publishers adopt the emergency slots in milestone 2"
    )
)]

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

pub(super) const EMERGENCY_CAPACITY: usize = 128;
pub(super) const SLOT_FREE: u8 = 0;
pub(super) const SLOT_WRITING: u8 = 1;
pub(super) const SLOT_PUBLISHED: u8 = 2;
pub(super) const SLOT_CLAIMED: u8 = 3;
pub(super) const SLOT_REMOVE_PENDING: u8 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct VerifiedProcessIdentity {
    pub(super) pid: u32,
    pub(super) native_scope: u64,
    pub(super) started_at: u64,
}

impl VerifiedProcessIdentity {
    pub(super) fn new(pid: u32, native_scope: u64, started_at: u64) -> Option<Self> {
        (pid > 0 && native_scope > 0 && started_at > 0).then_some(Self {
            pid,
            native_scope,
            started_at,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EmergencyKey {
    pub(super) index: usize,
    pub(super) generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EmergencyPublishError {
    Capacity,
}

pub(super) struct EmergencySlot {
    pub(super) state: AtomicU8,
    pub(super) generation: AtomicU64,
    pub(super) pid: AtomicU32,
    pub(super) native_scope: AtomicU64,
    pub(super) started_at: AtomicU64,
    pub(super) termination_requested: AtomicBool,
}

impl EmergencySlot {
    fn new() -> Self {
        Self {
            state: AtomicU8::new(SLOT_FREE),
            generation: AtomicU64::new(0),
            pid: AtomicU32::new(0),
            native_scope: AtomicU64::new(0),
            started_at: AtomicU64::new(0),
            termination_requested: AtomicBool::new(false),
        }
    }
}

pub(super) struct EmergencyInner {
    pub(super) slots: [EmergencySlot; EMERGENCY_CAPACITY],
}

#[derive(Clone)]
pub(super) struct EmergencyInventory {
    pub(super) inner: Arc<EmergencyInner>,
}

impl EmergencyInventory {
    pub(super) fn new() -> Self {
        Self {
            inner: Arc::new(EmergencyInner {
                slots: std::array::from_fn(|_| EmergencySlot::new()),
            }),
        }
    }

    pub(super) fn try_publish(
        &self,
        identity: VerifiedProcessIdentity,
    ) -> Result<EmergencyRegistration, EmergencyPublishError> {
        for (index, slot) in self.inner.slots.iter().enumerate() {
            if slot
                .state
                .compare_exchange(SLOT_FREE, SLOT_WRITING, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                continue;
            }
            let generation = next_generation(slot.generation.load(Ordering::Relaxed));
            slot.pid.store(identity.pid, Ordering::Relaxed);
            slot.native_scope
                .store(identity.native_scope, Ordering::Relaxed);
            slot.started_at
                .store(identity.started_at, Ordering::Relaxed);
            slot.termination_requested.store(false, Ordering::Relaxed);
            slot.generation.store(generation, Ordering::Relaxed);
            slot.state.store(SLOT_PUBLISHED, Ordering::Release);
            return Ok(EmergencyRegistration {
                inventory: self.clone(),
                key: Some(EmergencyKey { index, generation }),
            });
        }
        Err(EmergencyPublishError::Capacity)
    }

    pub(super) fn clear(&self, key: EmergencyKey) -> bool {
        let Some(slot) = self.inner.slots.get(key.index) else {
            return false;
        };
        loop {
            if slot.generation.load(Ordering::Acquire) != key.generation {
                return false;
            }
            match slot.state.load(Ordering::Acquire) {
                SLOT_PUBLISHED => {
                    if slot
                        .state
                        .compare_exchange(
                            SLOT_PUBLISHED,
                            SLOT_FREE,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return true;
                    }
                }
                SLOT_CLAIMED => {
                    if slot
                        .state
                        .compare_exchange(
                            SLOT_CLAIMED,
                            SLOT_REMOVE_PENDING,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        return true;
                    }
                }
                SLOT_REMOVE_PENDING => return true,
                SLOT_WRITING => std::hint::spin_loop(),
                _ => return false,
            }
        }
    }

    pub(super) fn has_active(&self) -> bool {
        self.inner
            .slots
            .iter()
            .any(|slot| slot.state.load(Ordering::Acquire) != SLOT_FREE)
    }

    #[cfg(test)]
    pub(super) fn active_count_for_test(&self) -> usize {
        self.inner
            .slots
            .iter()
            .filter(|slot| slot.state.load(Ordering::Acquire) != SLOT_FREE)
            .count()
    }

    #[cfg(test)]
    pub(super) fn clear_key_for_test(&self, key: EmergencyKey) -> bool {
        self.clear(key)
    }
}

impl Default for EmergencyInventory {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) struct EmergencyRegistration {
    inventory: EmergencyInventory,
    key: Option<EmergencyKey>,
}

impl EmergencyRegistration {
    #[cfg(test)]
    pub(super) fn key_for_test(&self) -> EmergencyKey {
        self.key.expect("emergency registration key")
    }
}

impl Drop for EmergencyRegistration {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            let _ = self.inventory.clear(key);
        }
    }
}

fn next_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the bounded admission API is adopted by service producers in milestone 2"
    )
)]

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub(super) use super::registry_admission::AdmissionKey;
pub use super::registry_admission::TrackedAdmission;

pub(super) const REGISTRY_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    Closing,
    Capacity,
}

impl AdmissionError {
    pub fn public_code(self) -> &'static str {
        match self {
            Self::Closing => "app-shutting-down",
            Self::Capacity => "app-work-capacity-reached",
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Slot {
    generation: u64,
    occupied: bool,
}

struct RegistryState {
    closed: bool,
    active: usize,
    slots: [Slot; REGISTRY_CAPACITY],
}

struct RegistryInner {
    state: Mutex<RegistryState>,
    cancel: CancellationToken,
    released: Notify,
}

#[derive(Clone)]
pub(super) struct AdmissionRegistry {
    inner: Arc<RegistryInner>,
}

impl AdmissionRegistry {
    pub(super) fn new() -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                state: Mutex::new(RegistryState {
                    closed: false,
                    active: 0,
                    slots: [Slot::default(); REGISTRY_CAPACITY],
                }),
                cancel: CancellationToken::new(),
                released: Notify::new(),
            }),
        }
    }

    pub(super) fn try_admit(&self) -> Result<TrackedAdmission, AdmissionError> {
        let child_cancel = self.inner.cancel.child_token();
        let mut state = self.lock_state();
        if state.closed {
            return Err(AdmissionError::Closing);
        }
        if state.active == REGISTRY_CAPACITY {
            return Err(AdmissionError::Capacity);
        }
        let (index, slot) = state
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| !slot.occupied)
            .ok_or(AdmissionError::Capacity)?;
        slot.generation = super::registry_admission::next_generation(slot.generation);
        slot.occupied = true;
        let key = AdmissionKey {
            index,
            generation: slot.generation,
        };
        state.active += 1;
        drop(state);
        Ok(TrackedAdmission {
            registry: self.clone(),
            key: Some(key),
            cancel: child_cancel,
        })
    }

    pub(super) fn close(&self) -> bool {
        let mut state = self.lock_state();
        if state.closed {
            return false;
        }
        state.closed = true;
        drop(state);
        self.inner.cancel.cancel();
        true
    }

    pub(super) fn active_count(&self) -> usize {
        self.lock_state().active
    }

    pub(super) async fn wait_empty_until(&self, deadline: Instant) -> bool {
        loop {
            let notified = self.inner.released.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.active_count() == 0 {
                return true;
            }
            if tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), notified)
                .await
                .is_err()
            {
                return self.active_count() == 0;
            }
        }
    }

    pub(super) fn release(&self, key: AdmissionKey) -> bool {
        let mut state = self.lock_state();
        let Some(slot) = state.slots.get_mut(key.index) else {
            return false;
        };
        if !slot.occupied || slot.generation != key.generation {
            return false;
        }
        slot.occupied = false;
        if state.active == 0 {
            state.closed = true;
            drop(state);
            self.inner.cancel.cancel();
            return false;
        }
        state.active -= 1;
        drop(state);
        self.inner.released.notify_waiters();
        true
    }

    fn lock_state(&self) -> MutexGuard<'_, RegistryState> {
        self.inner.state.lock().unwrap_or_else(|poisoned| {
            let mut state = poisoned.into_inner();
            state.closed = true;
            self.inner.cancel.cancel();
            state
        })
    }

    #[cfg(test)]
    pub(super) fn release_key_for_test(&self, key: AdmissionKey) -> bool {
        self.release(key)
    }
}

impl Default for AdmissionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

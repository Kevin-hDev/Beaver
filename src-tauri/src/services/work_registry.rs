mod stop;
mod task;

use crate::app_exit::{AppWorkAdmission, AppWorkSupervisor};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::{Mutex as AsyncMutex, Notify};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceWorkPhase {
    Open,
    Closing,
    Closed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServiceWorkDiagnostics {
    pub active: usize,
    pub high_water: usize,
    pub saturation_refusals: u64,
    pub closing_refusals: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceWorkAdmissionError {
    AppClosing,
    AppCapacity,
    Closing,
    Capacity,
}

impl ServiceWorkAdmissionError {
    pub fn public_code(self) -> &'static str {
        match self {
            Self::AppClosing => "app-shutting-down",
            Self::AppCapacity => "app-work-capacity-reached",
            Self::Closing => "service-shutting-down",
            Self::Capacity => "service-work-capacity-reached",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ServiceWorkKey {
    pub(super) index: usize,
    pub(super) generation: u64,
}

#[derive(Clone)]
pub struct ServiceWorkCancellation {
    pub(super) app: CancellationToken,
    pub(super) service: CancellationToken,
}

pub struct ServiceWorkAdmission<const CAPACITY: usize> {
    pub(super) registry: WorkRegistry<CAPACITY>,
    pub(super) key: Option<ServiceWorkKey>,
    pub(super) cancellation: ServiceWorkCancellation,
}

pub(super) struct ServiceWorkSlot {
    generation: u64,
    occupied: bool,
    app_admission: Option<AppWorkAdmission>,
    handle: Option<JoinHandle<()>>,
}

impl ServiceWorkSlot {
    fn empty() -> Self {
        Self {
            generation: 0,
            occupied: false,
            app_admission: None,
            handle: None,
        }
    }
}

pub(super) struct ServiceWorkState<const CAPACITY: usize> {
    phase: ServiceWorkPhase,
    diagnostics: ServiceWorkDiagnostics,
    slots: [ServiceWorkSlot; CAPACITY],
}

pub(super) struct WorkRegistryInner<const CAPACITY: usize> {
    state: Mutex<ServiceWorkState<CAPACITY>>,
    cancel: CancellationToken,
    changed: Notify,
    stop_owner: AsyncMutex<()>,
}

pub struct WorkRegistry<const CAPACITY: usize> {
    pub(super) inner: Arc<WorkRegistryInner<CAPACITY>>,
}

#[derive(Clone)]
pub struct ServiceWorkSupervisor<const CAPACITY: usize> {
    pub(super) app: AppWorkSupervisor,
    pub(super) registry: WorkRegistry<CAPACITY>,
}

impl<const CAPACITY: usize> WorkRegistry<CAPACITY> {
    pub fn new() -> Self {
        assert!(CAPACITY > 0, "work registry capacity must be positive");
        Self {
            inner: Arc::new(WorkRegistryInner {
                state: Mutex::new(ServiceWorkState {
                    phase: ServiceWorkPhase::Open,
                    diagnostics: ServiceWorkDiagnostics::default(),
                    slots: std::array::from_fn(|_| ServiceWorkSlot::empty()),
                }),
                cancel: CancellationToken::new(),
                changed: Notify::new(),
                stop_owner: AsyncMutex::new(()),
            }),
        }
    }

    pub fn phase(&self) -> ServiceWorkPhase {
        self.lock_state().phase
    }

    pub fn diagnostics(&self) -> ServiceWorkDiagnostics {
        self.lock_state().diagnostics
    }

    pub(super) fn release(&self, key: ServiceWorkKey) -> bool {
        let mut state = self.lock_state();
        let Some(slot) = state.slots.get_mut(key.index) else {
            return false;
        };
        if !slot.occupied || slot.generation != key.generation {
            return false;
        }
        slot.occupied = false;
        drop(slot.handle.take());
        let app_admission = slot.app_admission.take();
        if state.diagnostics.active == 0 {
            state.phase = ServiceWorkPhase::Closing;
            drop(state);
            drop(app_admission);
            self.inner.cancel.cancel();
            return false;
        }
        state.diagnostics.active -= 1;
        if state.diagnostics.active == 0 && state.phase == ServiceWorkPhase::Closing {
            state.phase = ServiceWorkPhase::Closed;
        }
        drop(state);
        // L'ordre d'acquisition est global puis local ; on libère donc le
        // verrou local avant de rendre l'admission globale.
        drop(app_admission);
        self.inner.changed.notify_waiters();
        true
    }

    pub(super) fn lock_state(&self) -> MutexGuard<'_, ServiceWorkState<CAPACITY>> {
        self.inner.state.lock().unwrap_or_else(|poisoned| {
            let mut state = poisoned.into_inner();
            state.phase = ServiceWorkPhase::Closing;
            self.inner.cancel.cancel();
            state
        })
    }

    #[cfg(test)]
    pub(super) fn release_key_for_test(&self, key: ServiceWorkKey) -> bool {
        self.release(key)
    }
}

impl<const CAPACITY: usize> Clone for WorkRegistry<CAPACITY> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<const CAPACITY: usize> Default for WorkRegistry<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAPACITY: usize> ServiceWorkSupervisor<CAPACITY> {
    pub fn phase(&self) -> ServiceWorkPhase {
        self.registry.phase()
    }

    pub fn diagnostics(&self) -> ServiceWorkDiagnostics {
        self.registry.diagnostics()
    }
}

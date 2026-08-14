use super::emergency::VerifiedProcessIdentity;
use super::emergency::{EmergencyInventory, EmergencyPublishError};
use super::emergency_drain::{EmergencyObservation, EmergencySignaler};
use super::emergency_registration::EmergencyHandoffReason;
use super::emergency_registration::EmergencyRegistration;

#[derive(Clone)]
pub(crate) struct AppEmergencyPublisher {
    inventory: EmergencyInventory,
}

pub(crate) struct AppEmergencyRegistration {
    registration: Option<EmergencyRegistration>,
}

impl AppEmergencyPublisher {
    pub(crate) fn new(inventory: EmergencyInventory) -> Self {
        Self { inventory }
    }

    pub(crate) fn publish(
        &self,
        pid: u32,
        native_scope: u64,
        started_at: u64,
        executable: u128,
    ) -> Result<AppEmergencyRegistration, EmergencyPublishError> {
        if pid < 2 || executable == 0 {
            return Err(EmergencyPublishError::InvalidIdentity);
        }
        let identity =
            VerifiedProcessIdentity::new_with_executable(pid, native_scope, started_at, executable)
                .ok_or(EmergencyPublishError::InvalidIdentity)?;
        self.inventory
            .try_publish(identity)
            .map(|registration| AppEmergencyRegistration {
                registration: Some(registration),
            })
    }

    #[cfg(test)]
    pub(crate) fn active_count_for_test(&self) -> usize {
        self.inventory.active_count_for_test()
    }

    #[cfg(test)]
    pub(crate) fn clear_for_test(&self, key: super::emergency::EmergencyKey) -> bool {
        self.inventory.clear_key_for_test(key)
    }
}

impl AppEmergencyRegistration {
    #[cfg(test)]
    pub(crate) fn key_for_test(&self) -> super::emergency::EmergencyKey {
        self.registration
            .as_ref()
            .expect("emergency registration")
            .key_for_test()
    }

    pub(crate) fn release_after_reap(mut self) {
        let _ = self.registration.take();
    }

    pub(crate) fn hand_off_to_watchdog(mut self, reason: EmergencyHandoffReason) {
        if let Some(registration) = self.registration.take() {
            registration.hand_off_to_watchdog(reason);
        }
    }
}

pub(crate) struct NativeEmergencySignaler;

impl EmergencySignaler for NativeEmergencySignaler {
    fn signal_or_recheck(
        &self,
        identity: VerifiedProcessIdentity,
        already_requested: bool,
    ) -> EmergencyObservation {
        platform::signal_or_recheck(identity, already_requested)
    }
}

#[cfg(unix)]
#[path = "emergency_signaler_unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "emergency_signaler_windows.rs"]
mod platform;

#[cfg(test)]
#[cfg(not(any(unix, windows)))]
mod platform {
    use super::*;
    pub(super) fn signal_or_recheck(
        _identity: VerifiedProcessIdentity,
        _already_requested: bool,
    ) -> EmergencyObservation {
        EmergencyObservation::IdentityMismatch
    }
}

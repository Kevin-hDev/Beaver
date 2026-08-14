use super::constants::PROCESS_REAP_FALLBACK_TIMEOUT;
use super::process::{NativeGatedProcess, OllamaProcessError};
use super::process_receipt::ProcessReceiptStore;
use crate::app_exit::AppEmergencyRegistration;
use crate::app_exit::EmergencyHandoffReason;
use crate::services::owned_process::OwnedProcessIdentity;
use std::time::Instant;

pub(crate) struct OwnedOllamaProcess {
    pub(crate) native: Option<NativeGatedProcess>,
    pub(crate) identity: OwnedProcessIdentity,
    pub(crate) receipt: Option<ProcessReceiptStore>,
    pub(crate) registration: Option<AppEmergencyRegistration>,
}

impl OwnedOllamaProcess {
    pub(crate) fn identity(&self) -> OwnedProcessIdentity {
        self.identity
    }

    pub(crate) fn terminate_and_reap(
        mut self,
        deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        let result = self
            .native
            .as_mut()
            .ok_or(OllamaProcessError::InvalidState)?
            .terminate_and_reap(deadline);
        if result.is_ok() {
            if let Some(receipt) = self.receipt.take() {
                if receipt.remove().is_err() {
                    self.receipt = Some(receipt);
                    if let Some(guard) = self.registration.take() {
                        guard.release_after_reap();
                    }
                    return Err(OllamaProcessError::Receipt);
                }
            }
            if let Some(guard) = self.registration.take() {
                guard.release_after_reap();
            }
        }
        result
    }
}

impl Drop for OwnedOllamaProcess {
    fn drop(&mut self) {
        let Some(native) = self.native.as_mut() else {
            return;
        };
        if native
            .terminate_and_reap(Instant::now() + PROCESS_REAP_FALLBACK_TIMEOUT)
            .is_err()
        {
            if let Some(registration) = self.registration.take() {
                registration.hand_off_to_watchdog(EmergencyHandoffReason::ReapFailed);
            }
            return;
        }
        if let Some(receipt) = self.receipt.take() {
            if receipt.remove().is_err() {
                self.receipt = Some(receipt);
            }
        }
        if let Some(guard) = self.registration.take() {
            guard.release_after_reap();
        }
    }
}

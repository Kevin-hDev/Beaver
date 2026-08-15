use super::super::constants::PROCESS_REAP_FALLBACK_TIMEOUT;
use super::super::process_receipt::ProcessReceiptStore;
use super::{GatedOllamaProcess, OllamaProcessError};
#[cfg(all(test, unix))]
use crate::app_exit::AppEmergencyPublisher;
use crate::app_exit::AppEmergencyRegistration;
use crate::app_exit::EmergencyHandoffReason;
use std::time::Instant;

impl GatedOllamaProcess {
    pub(crate) fn terminate(&mut self) -> Result<(), OllamaProcessError> {
        self.native
            .as_mut()
            .ok_or(OllamaProcessError::InvalidState)?
            .terminate()
    }

    pub(crate) fn reap(&mut self, deadline: Instant) -> Result<(), OllamaProcessError> {
        self.native
            .as_mut()
            .ok_or(OllamaProcessError::InvalidState)?
            .reap(deadline)
    }

    pub(crate) fn terminate_and_reap(
        mut self,
        deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        self.terminate()?;
        self.reap(deadline)
    }

    fn abort(&mut self) -> Result<(), OllamaProcessError> {
        self.terminate()?;
        self.reap(Instant::now() + PROCESS_REAP_FALLBACK_TIMEOUT)
    }

    pub(super) fn cleanup_failed_publish(
        &mut self,
        receipt: &ProcessReceiptStore,
        registration: Option<AppEmergencyRegistration>,
        remove_receipt: bool,
    ) {
        if self.abort().is_ok() {
            if let Some(registration) = registration {
                registration.release_after_reap();
            }
            if remove_receipt {
                let _ = receipt.remove();
            }
        } else if let Some(registration) = registration {
            registration.hand_off_to_watchdog(EmergencyHandoffReason::ReapFailed);
        }
    }
}

#[cfg(test)]
impl GatedOllamaProcess {
    #[cfg(unix)]
    pub(crate) fn close_gate_for_test(&mut self) {
        if let Some(native) = self.native.as_mut() {
            native.close_gate_for_test();
        }
    }

    #[cfg(unix)]
    pub(crate) fn publish_with_identity_change_for_test(
        self,
        receipt: &ProcessReceiptStore,
        emergency: &AppEmergencyPublisher,
    ) -> Result<super::OwnedOllamaProcess, OllamaProcessError> {
        self.publish_inner(receipt, emergency, |process| {
            // A deterministic identity transition proves the post-receipt
            // revalidation without depending on signal delivery scheduling.
            if let Some(native) = process.native.as_mut() {
                native.force_identity_change_for_test();
            }
        })
    }
}

impl Drop for GatedOllamaProcess {
    fn drop(&mut self) {
        if let Some(native) = self.native.as_mut() {
            let _ = native.terminate();
            let _ = native.reap(Instant::now() + PROCESS_REAP_FALLBACK_TIMEOUT);
        }
    }
}

use super::constants::PROCESS_REAP_FALLBACK_TIMEOUT;
use super::fingerprint::BundleFingerprint;
use super::process_receipt::{ProcessReceipt, ProcessReceiptRecovery, ProcessReceiptStore};
use super::spawn_profile::OllamaSpawnAttempt;
#[path = "process_lifecycle.rs"]
mod lifecycle;
use crate::app_exit::AppEmergencyPublisher;
use crate::services::owned_process::OwnedProcessIdentity;
use std::time::Instant;

#[cfg(unix)]
pub(crate) use super::spawn_gate_unix::NativeGatedProcess;
#[cfg(windows)]
pub(crate) use super::spawn_gate_windows::NativeGatedProcess;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OllamaProcessError {
    Spawn,
    Gate,
    Admission,
    Identity,
    Receipt,
    EmergencyCapacity,
    Reap,
    InvalidState,
}

pub(crate) trait OllamaProcessLauncher: Send + Sync {
    fn create_gated(
        &self,
        attempt: &OllamaSpawnAttempt<'_>,
    ) -> Result<GatedOllamaProcess, OllamaProcessError>;
}

pub(crate) struct DefaultOllamaProcessLauncher {
    bundle: BundleFingerprint,
}

impl DefaultOllamaProcessLauncher {
    pub(crate) fn new(bundle: BundleFingerprint) -> Self {
        Self { bundle }
    }

    pub(crate) fn recover_receipt(
        &self,
        store: &ProcessReceiptStore,
        expected_executable: u128,
        deadline: Instant,
    ) -> Result<ProcessReceiptRecovery, OllamaProcessError> {
        store
            .recover_active(&self.bundle, expected_executable, deadline)
            .map_err(|_| OllamaProcessError::Receipt)
    }
}

pub(crate) struct GatedOllamaProcess {
    native: Option<NativeGatedProcess>,
    identity: OwnedProcessIdentity,
    executable: u128,
    bundle: BundleFingerprint,
}

pub(crate) use super::process_owned::OwnedOllamaProcess;

impl OllamaProcessLauncher for DefaultOllamaProcessLauncher {
    fn create_gated(
        &self,
        attempt: &OllamaSpawnAttempt<'_>,
    ) -> Result<GatedOllamaProcess, OllamaProcessError> {
        let native = platform_create(attempt)?;
        let identity = native.identity();
        let executable = attempt
            .profile()
            .executable()
            .execution_identity()
            .ok_or(OllamaProcessError::Identity)?;
        Ok(GatedOllamaProcess {
            native: Some(native),
            identity,
            executable,
            bundle: self.bundle.clone(),
        })
    }
}

impl GatedOllamaProcess {
    pub(crate) fn identity(&self) -> Result<OwnedProcessIdentity, OllamaProcessError> {
        self.native
            .as_ref()
            .ok_or(OllamaProcessError::InvalidState)?
            .revalidate(self.executable)?;
        Ok(self.identity)
    }

    pub(crate) fn publish(
        self,
        receipt: &ProcessReceiptStore,
        emergency: &AppEmergencyPublisher,
    ) -> Result<OwnedOllamaProcess, OllamaProcessError> {
        self.publish_inner(receipt, emergency, || {})
    }

    #[cfg(test)]
    pub(crate) fn publish_with_cutpoint(
        self,
        receipt: &ProcessReceiptStore,
        emergency: &AppEmergencyPublisher,
        after_receipt: impl FnOnce(),
    ) -> Result<OwnedOllamaProcess, OllamaProcessError> {
        self.publish_inner(receipt, emergency, after_receipt)
    }

    #[cfg(test)]
    pub(crate) fn force_reap_failure_for_test(&mut self) {
        #[cfg(unix)]
        if let Some(native) = self.native.as_mut() {
            native.force_reap_failure_for_test();
        }
    }

    #[cfg(test)]
    pub(crate) fn open_gate_for_test(&mut self) {
        if let Some(native) = self.native.as_mut() {
            let _ = native.open_gate();
        }
    }

    fn publish_inner(
        mut self,
        receipt: &ProcessReceiptStore,
        emergency: &AppEmergencyPublisher,
        after_receipt: impl FnOnce(),
    ) -> Result<OwnedOllamaProcess, OllamaProcessError> {
        self.native
            .as_ref()
            .ok_or(OllamaProcessError::InvalidState)?
            .revalidate(self.executable)?;
        let process_receipt = ProcessReceipt::new(
            self.identity.pid,
            self.identity.native_start_time,
            self.identity.native_scope,
            self.bundle.clone(),
        )
        .map_err(|_| OllamaProcessError::Receipt)?;
        if receipt.write_new(&process_receipt).is_err() {
            self.cleanup_failed_publish(receipt, None, false);
            return Err(OllamaProcessError::Receipt);
        }
        after_receipt();
        if self
            .native
            .as_ref()
            .ok_or(OllamaProcessError::InvalidState)
            .and_then(|native| native.revalidate(self.executable))
            .is_err()
        {
            self.cleanup_failed_publish(receipt, None, true);
            return Err(OllamaProcessError::Identity);
        }
        let registration = match emergency.publish(
            self.identity.pid,
            self.identity.native_scope,
            self.identity.native_start_time,
            self.identity.executable,
        ) {
            Ok(registration) => registration,
            Err(_) => {
                self.cleanup_failed_publish(receipt, None, true);
                return Err(OllamaProcessError::EmergencyCapacity);
            }
        };
        let gate_result = self
            .native
            .as_mut()
            .ok_or(OllamaProcessError::InvalidState)
            .and_then(NativeGatedProcess::open_gate);
        if gate_result.is_err() {
            self.cleanup_failed_publish(receipt, Some(registration), true);
            return Err(OllamaProcessError::Gate);
        }
        let ready = self
            .native
            .as_mut()
            .ok_or(OllamaProcessError::InvalidState)
            .and_then(|native| {
                native.wait_for_executable(
                    self.executable,
                    Instant::now() + PROCESS_REAP_FALLBACK_TIMEOUT,
                )
            });
        if ready.is_err() {
            self.cleanup_failed_publish(receipt, Some(registration), true);
            return Err(OllamaProcessError::Identity);
        }
        let native = self.native.take().ok_or(OllamaProcessError::InvalidState)?;
        Ok(OwnedOllamaProcess {
            native: Some(native),
            identity: self.identity,
            receipt: Some(receipt.clone()),
            registration: Some(registration),
        })
    }
}

#[cfg(unix)]
fn platform_create(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    super::spawn_gate_unix::create(attempt)
}

#[cfg(windows)]
fn platform_create(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    super::spawn_gate_windows::create(attempt)
}

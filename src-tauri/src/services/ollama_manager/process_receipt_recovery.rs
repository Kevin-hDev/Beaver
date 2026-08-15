use super::fingerprint::BundleFingerprint;
use super::process_receipt::{ProcessReceipt, ProcessReceiptError, ProcessReceiptStore};
use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryProbe {
    Exact,
    Missing,
    Different,
    Ambiguous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProcessReceiptRecovery {
    Missing,
    StaleRemoved,
    RecoveryRequired,
    Exact(ProcessReceipt),
    Reaped,
}

impl ProcessReceiptStore {
    pub(crate) fn recover<F>(
        &self,
        active_bundle: &BundleFingerprint,
        inspect: F,
    ) -> Result<ProcessReceiptRecovery, ProcessReceiptError>
    where
        F: FnOnce(&ProcessReceipt) -> RecoveryProbe,
    {
        let Some(receipt) = self.read()? else {
            return Ok(ProcessReceiptRecovery::Missing);
        };
        let probe = inspect(&receipt);
        if !bundle_matches(&receipt.bundle, active_bundle) {
            return match probe {
                RecoveryProbe::Missing | RecoveryProbe::Different => {
                    self.remove()?;
                    Ok(ProcessReceiptRecovery::StaleRemoved)
                }
                RecoveryProbe::Exact | RecoveryProbe::Ambiguous => {
                    Ok(ProcessReceiptRecovery::RecoveryRequired)
                }
            };
        }
        match probe {
            RecoveryProbe::Exact => Ok(ProcessReceiptRecovery::Exact(receipt)),
            RecoveryProbe::Missing | RecoveryProbe::Different => {
                self.remove()?;
                Ok(ProcessReceiptRecovery::StaleRemoved)
            }
            RecoveryProbe::Ambiguous => Ok(ProcessReceiptRecovery::RecoveryRequired),
        }
    }

    pub(crate) fn recover_identity(
        &self,
        active_bundle: &BundleFingerprint,
        expected_executable: u128,
        inspect: impl FnOnce(u32) -> Result<OwnedProcessIdentity, RecoveryProbe>,
    ) -> Result<ProcessReceiptRecovery, ProcessReceiptError> {
        self.recover(active_bundle, |receipt| {
            let identity = match inspect(receipt.pid) {
                Ok(identity) => identity,
                Err(probe) => return probe,
            };
            if identity.executable == 0 {
                return RecoveryProbe::Ambiguous;
            }
            if identity.pid == receipt.pid
                && identity.native_start_time == receipt.native_start_time
                && identity.native_scope == receipt.native_scope
                && identity.executable == expected_executable
            {
                RecoveryProbe::Exact
            } else {
                RecoveryProbe::Different
            }
        })
    }

    pub(crate) fn reap_exact<Reap>(
        &self,
        receipt: &ProcessReceipt,
        reap: Reap,
    ) -> Result<(), ProcessReceiptError>
    where
        Reap: FnOnce(&ProcessReceipt) -> Result<(), ProcessReceiptError>,
    {
        reap(receipt)?;
        self.remove()
    }

    pub(crate) fn recover_active(
        &self,
        active_bundle: &BundleFingerprint,
        expected_executable: u128,
        deadline: Instant,
    ) -> Result<ProcessReceiptRecovery, ProcessReceiptError> {
        let Some(receipt) = self.read()? else {
            return Ok(ProcessReceiptRecovery::Missing);
        };
        let identity = match inspect_native_identity(receipt.pid, expected_executable) {
            Ok(identity) => identity,
            Err(_) => {
                if OwnedProcess::process_exists(receipt.pid) {
                    return Ok(ProcessReceiptRecovery::RecoveryRequired);
                }
                self.remove()?;
                return Ok(ProcessReceiptRecovery::StaleRemoved);
            }
        };
        let exact_identity = identity.pid == receipt.pid
            && identity.native_start_time == receipt.native_start_time
            && identity.native_scope == receipt.native_scope
            && identity.executable != 0
            && identity.executable == expected_executable;
        if !exact_identity {
            if identity.executable == 0 {
                return Ok(ProcessReceiptRecovery::RecoveryRequired);
            }
            self.remove()?;
            return Ok(ProcessReceiptRecovery::StaleRemoved);
        }
        if !bundle_matches(&receipt.bundle, active_bundle) {
            return Ok(ProcessReceiptRecovery::RecoveryRequired);
        }
        if OwnedProcess::recover_exact(identity, deadline).is_err() {
            return Ok(ProcessReceiptRecovery::RecoveryRequired);
        }
        self.remove()?;
        Ok(ProcessReceiptRecovery::Reaped)
    }
}

fn inspect_native_identity(
    pid: u32,
    expected_executable: u128,
) -> Result<OwnedProcessIdentity, crate::services::owned_process::OwnedProcessError> {
    #[cfg(windows)]
    return OwnedProcess::identity_with_executable(pid, expected_executable);
    #[cfg(not(windows))]
    {
        let _ = expected_executable;
        OwnedProcess::identity(pid)
    }
}

fn bundle_matches(left: &BundleFingerprint, right: &BundleFingerprint) -> bool {
    left.version == right.version
        && left
            .executable_sha256
            .constant_time_eq(&right.executable_sha256)
}

use super::fingerprint::BundleFingerprint;
use super::process_receipt::{ProcessReceiptError, ProcessReceiptStore};
use crate::services::owned_process::{OwnedProcess, OwnedProcessInspection};
use std::time::Instant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProcessReceiptRecovery {
    Missing,
    StaleRemoved,
    RecoveryRequired,
    Reaped,
}

impl ProcessReceiptStore {
    pub(crate) fn recover_active(
        &self,
        active_bundle: &BundleFingerprint,
        expected_executable: u128,
        deadline: Instant,
    ) -> Result<ProcessReceiptRecovery, ProcessReceiptError> {
        let Some(receipt) = self.read()? else {
            return Ok(ProcessReceiptRecovery::Missing);
        };
        let identity =
            match OwnedProcess::inspect_for_recovery(receipt.pid, receipt.native_start_time) {
                Ok(OwnedProcessInspection::Owned(identity)) => identity,
                Ok(OwnedProcessInspection::Unowned) => {
                    self.remove()?;
                    return Ok(ProcessReceiptRecovery::StaleRemoved);
                }
                Err(_) => {
                    match OwnedProcess::reap_exited_child(receipt.pid) {
                        Ok(true) => {
                            self.remove()?;
                            return Ok(ProcessReceiptRecovery::Reaped);
                        }
                        Ok(false) => {}
                        Err(_) => return Ok(ProcessReceiptRecovery::RecoveryRequired),
                    }
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

fn bundle_matches(left: &BundleFingerprint, right: &BundleFingerprint) -> bool {
    left.version == right.version
        && left
            .executable_sha256
            .constant_time_eq(&right.executable_sha256)
}

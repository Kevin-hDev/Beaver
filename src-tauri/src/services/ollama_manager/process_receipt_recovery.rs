use super::fingerprint::BundleFingerprint;
use super::process_receipt::{ProcessReceipt, ProcessReceiptError, ProcessReceiptStore};
use crate::services::owned_process::OwnedProcessIdentity;

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
        if !bundle_matches(&receipt.bundle, active_bundle) {
            self.remove()?;
            return Ok(ProcessReceiptRecovery::StaleRemoved);
        }
        match inspect(&receipt) {
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
}

fn bundle_matches(left: &BundleFingerprint, right: &BundleFingerprint) -> bool {
    left.version == right.version
        && left
            .executable_sha256
            .constant_time_eq(&right.executable_sha256)
}

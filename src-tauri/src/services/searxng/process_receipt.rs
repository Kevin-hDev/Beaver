#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity, OwnedProcessInspection};
use crate::services::private_store::{self, BoundedFile};

const SCHEMA_VERSION: u8 = 1;
const MAX_RECEIPT_BYTES: u64 = 4_096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct SearxngProcessReceipt {
    schema_version: u8,
    pid: u32,
    native_start_time: u64,
    native_scope: u64,
    executable_high: u64,
    executable_low: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearxngProcessReceiptWire {
    schema_version: u8,
    pid: u32,
    native_start_time: u64,
    native_scope: u64,
    executable_high: u64,
    executable_low: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RecoveryOutcome {
    Missing,
    LegacyUnverifiable,
    Stale,
    Exact,
    Blocked,
}

pub(super) struct SearxngProcessReceiptStore {
    path: PathBuf,
}

pub(super) fn store() -> SearxngProcessReceiptStore {
    SearxngProcessReceiptStore {
        path: crate::services::paths::data_dir().join("searxng-sidecar.pid"),
    }
}

impl SearxngProcessReceipt {
    pub(super) fn from_identity(identity: OwnedProcessIdentity) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            pid: identity.pid,
            native_start_time: identity.native_start_time,
            native_scope: identity.native_scope,
            executable_high: (identity.executable >> 64) as u64,
            executable_low: identity.executable as u64,
        }
    }

    fn identity(&self) -> OwnedProcessIdentity {
        OwnedProcessIdentity {
            pid: self.pid,
            native_start_time: self.native_start_time,
            native_scope: self.native_scope,
            executable: (u128::from(self.executable_high) << 64) | u128::from(self.executable_low),
        }
    }

    fn valid(&self) -> bool {
        self.schema_version == SCHEMA_VERSION
            && self.pid >= 2
            && self.native_start_time != 0
            && self.native_scope != 0
            && self.executable_high != 0
            && self.executable_low != 0
    }
}

impl SearxngProcessReceiptStore {
    #[cfg(test)]
    pub(super) fn at(path: PathBuf) -> Self {
        Self { path }
    }

    #[cfg(test)]
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn write(&self, identity: &OwnedProcessIdentity) -> Result<(), ()> {
        let receipt = SearxngProcessReceipt::from_identity(*identity);
        if !receipt.valid() {
            return Err(());
        }
        let bytes = serde_json::to_vec(&receipt).map_err(|_| ())?;
        (bytes.len() < MAX_RECEIPT_BYTES as usize)
            .then_some(())
            .ok_or(())?;
        private_store::atomic_write(&self.path, &bytes).map_err(|_| ())
    }

    #[cfg(test)]
    pub(super) fn read(&self) -> Result<SearxngProcessReceipt, ()> {
        match self.load()? {
            ReceiptState::Receipt(receipt) => Ok(receipt),
            ReceiptState::Missing | ReceiptState::Legacy => Err(()),
        }
    }

    pub(super) fn recover_and_reap(&self, deadline: Instant) -> Result<RecoveryOutcome, ()> {
        match self.load()? {
            ReceiptState::Missing => Ok(RecoveryOutcome::Missing),
            ReceiptState::Legacy => {
                self.remove()?;
                ::log::warn!("[searxng] legacy pid receipt removed without recovery");
                Ok(RecoveryOutcome::LegacyUnverifiable)
            }
            ReceiptState::Receipt(receipt) => {
                let outcome = classify_recovery(
                    receipt,
                    OwnedProcess::process_exists,
                    |pid| OwnedProcess::reap_exited_child(pid).unwrap_or(false),
                    |pid, started| OwnedProcess::inspect_for_recovery(pid, started).map_err(|_| ()),
                    |identity, until| OwnedProcess::recover_exact(identity, until).map_err(|_| ()),
                    deadline,
                );
                if matches!(outcome, RecoveryOutcome::Stale | RecoveryOutcome::Exact) {
                    self.remove()?;
                }
                Ok(outcome)
            }
        }
    }

    pub(super) fn remove(&self) -> Result<(), ()> {
        match private_store::read_bounded_regular(&self.path, MAX_RECEIPT_BYTES).map_err(|_| ())? {
            BoundedFile::Missing => Ok(()),
            BoundedFile::Content(_) => std::fs::remove_file(&self.path).map_err(|_| ()),
        }
    }

    fn load(&self) -> Result<ReceiptState, ()> {
        match private_store::read_bounded_regular(&self.path, MAX_RECEIPT_BYTES).map_err(|_| ())? {
            BoundedFile::Missing => Ok(ReceiptState::Missing),
            BoundedFile::Content(bytes) if legacy_numeric(&bytes) => Ok(ReceiptState::Legacy),
            BoundedFile::Content(bytes) => parse(&bytes).map(ReceiptState::Receipt),
        }
    }
}

enum ReceiptState {
    Missing,
    Legacy,
    Receipt(SearxngProcessReceipt),
}

pub(super) fn classify_recovery(
    receipt: SearxngProcessReceipt,
    exists: impl Fn(u32) -> bool,
    reap_exited: impl Fn(u32) -> bool,
    inspect: impl FnOnce(u32, u64) -> Result<OwnedProcessInspection, ()>,
    recover: impl FnOnce(OwnedProcessIdentity, Instant) -> Result<(), ()>,
    deadline: Instant,
) -> RecoveryOutcome {
    match inspect(receipt.pid, receipt.native_start_time) {
        Ok(OwnedProcessInspection::Unowned) => RecoveryOutcome::Stale,
        Ok(OwnedProcessInspection::Owned(identity)) if identity == receipt.identity() => {
            recover(identity, deadline)
                .map(|_| RecoveryOutcome::Exact)
                .unwrap_or(RecoveryOutcome::Blocked)
        }
        Ok(OwnedProcessInspection::Owned(_)) => RecoveryOutcome::Stale,
        Err(()) if reap_exited(receipt.pid) => RecoveryOutcome::Exact,
        Err(()) if !exists(receipt.pid) => RecoveryOutcome::Stale,
        Err(()) => RecoveryOutcome::Blocked,
    }
}

fn parse(bytes: &[u8]) -> Result<SearxngProcessReceipt, ()> {
    let wire = serde_json::from_slice::<SearxngProcessReceiptWire>(bytes).map_err(|_| ())?;
    let receipt = SearxngProcessReceipt {
        schema_version: wire.schema_version,
        pid: wire.pid,
        native_start_time: wire.native_start_time,
        native_scope: wire.native_scope,
        executable_high: wire.executable_high,
        executable_low: wire.executable_low,
    };
    receipt.valid().then_some(receipt).ok_or(())
}

fn legacy_numeric(bytes: &[u8]) -> bool {
    bytes.iter().any(u8::is_ascii_digit)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || byte.is_ascii_whitespace())
}

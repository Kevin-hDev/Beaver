use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::durable_fs::{OllamaDurableFs, OllamaFsErrorKind};
use super::fingerprint::BundleFingerprint;
use crate::services::paths::OllamaPaths;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) type NativeStartTime = u64;
pub(crate) type NativeProcessScope = u64;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProcessReceipt {
    pub schema_version: u8,
    pub pid: u32,
    pub native_start_time: NativeStartTime,
    pub bundle: BundleFingerprint,
    pub native_scope: NativeProcessScope,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProcessReceiptWire {
    schema_version: u8,
    pid: u32,
    native_start_time: NativeStartTime,
    bundle: BundleFingerprint,
    native_scope: NativeProcessScope,
}

impl From<ProcessReceiptWire> for ProcessReceipt {
    fn from(wire: ProcessReceiptWire) -> Self {
        Self {
            schema_version: wire.schema_version,
            pid: wire.pid,
            native_start_time: wire.native_start_time,
            bundle: wire.bundle,
            native_scope: wire.native_scope,
        }
    }
}

impl ProcessReceipt {
    pub(crate) fn new(
        pid: u32,
        native_start_time: NativeStartTime,
        native_scope: NativeProcessScope,
        bundle: BundleFingerprint,
    ) -> Result<Self, ProcessReceiptError> {
        let receipt = Self {
            schema_version: 1,
            pid,
            native_start_time,
            bundle,
            native_scope,
        };
        receipt.validate().map(|()| receipt)
    }

    pub(crate) fn parse_bounded(bytes: &[u8]) -> Result<Self, ProcessReceiptError> {
        if bytes.len() > MAX_DURABLE_DOCUMENT_BYTES {
            return Err(ProcessReceiptError::Oversized);
        }
        let receipt: ProcessReceiptWire =
            serde_json::from_slice(bytes).map_err(|_| ProcessReceiptError::Invalid)?;
        let receipt: ProcessReceipt = receipt.into();
        receipt.validate()?;
        Ok(receipt)
    }

    pub(crate) fn serialize_bounded(&self) -> Result<Vec<u8>, ProcessReceiptError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| ProcessReceiptError::Invalid)?;
        (bytes.len() <= MAX_DURABLE_DOCUMENT_BYTES)
            .then_some(bytes)
            .ok_or(ProcessReceiptError::Oversized)
    }

    fn validate(&self) -> Result<(), ProcessReceiptError> {
        (self.schema_version == 1
            && self.pid > 1
            && self.native_start_time > 0
            && self.native_scope > 0)
            .then_some(())
            .ok_or(ProcessReceiptError::Invalid)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProcessReceiptError {
    Missing,
    Oversized,
    Invalid,
    Storage,
}

pub(crate) use super::process_receipt_recovery::ProcessReceiptRecovery;
#[cfg(test)]
pub(crate) use super::process_receipt_recovery::RecoveryProbe;

#[derive(Clone)]
pub(crate) struct ProcessReceiptStore {
    fs: Arc<dyn OllamaDurableFs>,
    path: PathBuf,
    tmp: PathBuf,
}

impl ProcessReceiptStore {
    pub(crate) fn new(fs: Arc<dyn OllamaDurableFs>, path: PathBuf, tmp: PathBuf) -> Self {
        Self { fs, path, tmp }
    }

    pub(crate) fn platform(paths: OllamaPaths) -> Self {
        let path = paths.process_receipt.clone();
        Self::new(
            Arc::new(super::durable_fs::platform_fs()),
            path.clone(),
            path.with_extension("tmp"),
        )
    }

    pub(crate) fn read(&self) -> Result<Option<ProcessReceipt>, ProcessReceiptError> {
        match self.fs.read_bounded(&self.path, MAX_DURABLE_DOCUMENT_BYTES) {
            Ok(bytes) => ProcessReceipt::parse_bounded(&bytes).map(Some),
            Err(error) if error.kind() == OllamaFsErrorKind::NotFound => Ok(None),
            Err(_) => Err(ProcessReceiptError::Storage),
        }
    }

    pub(crate) fn write_new(&self, receipt: &ProcessReceipt) -> Result<(), ProcessReceiptError> {
        let bytes = receipt.serialize_bounded()?;
        let parent = self.path.parent().ok_or(ProcessReceiptError::Storage)?;
        match self.fs.read_bounded(&self.tmp, MAX_DURABLE_DOCUMENT_BYTES) {
            Err(error) if error.kind() == OllamaFsErrorKind::NotFound => {}
            Ok(_) | Err(_) => return Err(ProcessReceiptError::Storage),
        }
        self.fs
            .create_directory_durable(parent)
            .map_err(|_| ProcessReceiptError::Storage)?;
        self.fs
            .write_new_atomic(&self.tmp, &self.path, &bytes)
            .map_err(|_| ProcessReceiptError::Storage)?;
        let committed = self.read()?.ok_or(ProcessReceiptError::Storage)?;
        (committed == *receipt)
            .then_some(())
            .ok_or(ProcessReceiptError::Invalid)
    }

    pub(crate) fn replace(&self, receipt: &ProcessReceipt) -> Result<(), ProcessReceiptError> {
        let bytes = receipt.serialize_bounded()?;
        self.fs
            .replace_atomic(&self.tmp, &self.path, &bytes)
            .map_err(|_| ProcessReceiptError::Storage)?;
        Ok(())
    }

    pub(crate) fn remove(&self) -> Result<(), ProcessReceiptError> {
        match self.fs.remove_file_durable(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == OllamaFsErrorKind::NotFound => Ok(()),
            Err(_) => Err(ProcessReceiptError::Storage),
        }
    }
}

use super::authority_slot::{SLOT_ADMITTED, SLOT_PUBLISHED, SLOT_RESERVED};
use super::slots::{CefAuthorityInner, CefSlotKey};
use super::{CefLaunchMarker, CefTableError};
use std::fmt;
use std::sync::Arc;
use zeroize::Zeroizing;

pub(super) struct CefReservation {
    pub(super) table: Arc<CefAuthorityInner>,
    pub(super) key: CefSlotKey,
    pub(super) marker: CefLaunchMarker,
}

impl CefReservation {
    pub(super) fn marker(&self) -> &CefLaunchMarker {
        &self.marker
    }

    pub(super) fn encode_marker(&self) -> Zeroizing<String> {
        self.marker.encode()
    }
}

impl fmt::Debug for CefReservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CefReservation([redacted])")
    }
}

impl Drop for CefReservation {
    fn drop(&mut self) {
        self.table.release(self.key, SLOT_RESERVED);
    }
}

pub(super) struct CefClaim {
    table: Arc<CefAuthorityInner>,
    key: Option<CefSlotKey>,
}

impl CefClaim {
    pub(super) fn new(table: Arc<CefAuthorityInner>, key: CefSlotKey) -> Self {
        Self {
            table,
            key: Some(key),
        }
    }

    pub(super) fn admit(mut self) -> Result<CefAdmission, CefTableError> {
        let key = self.key.ok_or(CefTableError::Stale)?;
        self.table.admit(key)?;
        self.key = None;
        Ok(CefAdmission {
            table: Arc::clone(&self.table),
            key: Some(key),
        })
    }

    pub(super) fn slot(&self) -> usize {
        self.key.map_or(usize::MAX, |key| key.index())
    }

    pub(super) fn generation(&self) -> u64 {
        self.key.map_or(0, |key| key.generation())
    }
}

impl fmt::Debug for CefClaim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CefClaim([redacted])")
    }
}

impl Drop for CefClaim {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.table.release(key, SLOT_PUBLISHED);
        }
    }
}

pub(super) struct CefAdmission {
    table: Arc<CefAuthorityInner>,
    key: Option<CefSlotKey>,
}

impl Drop for CefAdmission {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.table.release(key, SLOT_ADMITTED);
        }
    }
}

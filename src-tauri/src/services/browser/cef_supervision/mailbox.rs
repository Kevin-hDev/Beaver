use super::constants::CEF_NONCE_BYTES;
use super::{CefLaunchMarker, CefProcessRole, CefTableError};
use std::fmt;
use zeroize::Zeroizing;

pub(in crate::services::browser) struct CefPublication {
    pub(super) slot: usize,
    pub(super) generation: u64,
    pub(super) role: CefProcessRole,
    pub(super) pid: u32,
    pub(super) nonce: Zeroizing<[u8; CEF_NONCE_BYTES]>,
}

impl CefPublication {
    pub(super) fn from_marker(marker: &CefLaunchMarker, pid: u32) -> Result<Self, CefTableError> {
        if pid == 0 {
            return Err(CefTableError::Invalid);
        }
        Ok(Self {
            slot: marker.slot(),
            generation: marker.generation(),
            role: marker.role(),
            pid,
            nonce: marker.copy_nonce(),
        })
    }
}

impl fmt::Debug for CefPublication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CefPublication([redacted])")
    }
}

use super::constants::{CEF_MARKER_MAX_BYTES, CEF_NONCE_BYTES, CEF_SLOT_CAPACITY};
use super::CefProcessRole;
use rand::{rngs::OsRng, RngCore};
use std::fmt;
use zeroize::Zeroizing;

const MARKER_VERSION: &str = "v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::browser) enum CefMarkerError {
    Missing,
    Duplicate,
    Invalid,
}

#[derive(Clone)]
pub(in crate::services::browser) struct CefLaunchMarker {
    slot: usize,
    generation: u64,
    role: CefProcessRole,
    nonce: Zeroizing<[u8; CEF_NONCE_BYTES]>,
}

impl CefLaunchMarker {
    pub(super) fn generate(
        slot: usize,
        generation: u64,
        role: CefProcessRole,
    ) -> Result<Self, CefMarkerError> {
        if slot >= CEF_SLOT_CAPACITY || generation == 0 {
            return Err(CefMarkerError::Invalid);
        }
        let mut nonce = Zeroizing::new([0_u8; CEF_NONCE_BYTES]);
        OsRng.fill_bytes(nonce.as_mut());
        Ok(Self {
            slot,
            generation,
            role,
            nonce,
        })
    }

    pub(super) fn encode(&self) -> Zeroizing<String> {
        let mut nonce_hex = Zeroizing::new([0_u8; CEF_NONCE_BYTES * 2]);
        hex::encode_to_slice(self.nonce.as_ref(), nonce_hex.as_mut())
            .expect("fixed hexadecimal buffer");
        let nonce = std::str::from_utf8(nonce_hex.as_ref()).expect("hexadecimal is UTF-8");
        Zeroizing::new(format!(
            "{MARKER_VERSION}:{}:{}:{}:{nonce}",
            self.slot,
            self.generation,
            u8::from(self.role)
        ))
    }

    pub(super) fn constant_time_matches(&self, other: &Self) -> bool {
        let mut difference = 0_u8;
        for (left, right) in self.nonce.iter().zip(other.nonce.iter()) {
            difference |= left ^ right;
        }
        self.slot == other.slot
            && self.generation == other.generation
            && self.role == other.role
            && difference == 0
    }

    pub(super) fn slot(&self) -> usize {
        self.slot
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn role(&self) -> CefProcessRole {
        self.role
    }

    pub(super) fn copy_nonce(&self) -> Zeroizing<[u8; CEF_NONCE_BYTES]> {
        self.nonce.clone()
    }

    pub(super) fn nonce_words(&self) -> [u64; CEF_NONCE_BYTES / 8] {
        std::array::from_fn(|index| {
            let offset = index * 8;
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(&self.nonce[offset..offset + 8]);
            u64::from_le_bytes(bytes)
        })
    }

    pub(super) fn nonce_bytes(&self) -> &[u8; CEF_NONCE_BYTES] {
        &self.nonce
    }

    pub(super) fn decode_unique(values: &[&str]) -> Result<Self, CefMarkerError> {
        let value = match values {
            [] => return Err(CefMarkerError::Missing),
            [value] => *value,
            _ => return Err(CefMarkerError::Duplicate),
        };
        Self::decode(value)
    }

    fn decode(value: &str) -> Result<Self, CefMarkerError> {
        if value.is_empty() || value.len() > CEF_MARKER_MAX_BYTES {
            return Err(CefMarkerError::Invalid);
        }
        let mut parts = value.split(':');
        let version = parts.next().ok_or(CefMarkerError::Invalid)?;
        let slot = parts
            .next()
            .ok_or(CefMarkerError::Invalid)?
            .parse::<usize>()
            .map_err(|_| CefMarkerError::Invalid)?;
        let generation = parts
            .next()
            .ok_or(CefMarkerError::Invalid)?
            .parse::<u64>()
            .map_err(|_| CefMarkerError::Invalid)?;
        let role = parts
            .next()
            .ok_or(CefMarkerError::Invalid)?
            .parse::<u8>()
            .map_err(|_| CefMarkerError::Invalid)?;
        let nonce_hex = parts.next().ok_or(CefMarkerError::Invalid)?;
        if version != MARKER_VERSION
            || parts.next().is_some()
            || slot >= CEF_SLOT_CAPACITY
            || generation == 0
            || nonce_hex.len() != CEF_NONCE_BYTES * 2
        {
            return Err(CefMarkerError::Invalid);
        }
        let role = CefProcessRole::try_from(role).map_err(|_| CefMarkerError::Invalid)?;
        let mut nonce = Zeroizing::new([0_u8; CEF_NONCE_BYTES]);
        hex::decode_to_slice(nonce_hex, nonce.as_mut()).map_err(|_| CefMarkerError::Invalid)?;
        Ok(Self {
            slot,
            generation,
            role,
            nonce,
        })
    }
}

impl fmt::Debug for CefLaunchMarker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CefLaunchMarker([redacted])")
    }
}

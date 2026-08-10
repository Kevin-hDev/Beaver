use super::constants::CEF_NONCE_BYTES;
use super::{CefLaunchMarker, CefPublication};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};

pub(super) const SLOT_FREE: u8 = 0;
pub(super) const SLOT_WRITING: u8 = 1;
pub(super) const SLOT_RESERVED: u8 = 2;
pub(super) const SLOT_CLAIMING: u8 = 3;
pub(super) const SLOT_PUBLISHED: u8 = 4;
pub(super) const SLOT_ADMITTED: u8 = 5;
const SLOT_CLEANING: u8 = 6;

pub(super) struct CefAuthoritySlot {
    pub(super) state: AtomicU8,
    pub(super) generation: AtomicU64,
    pub(super) pid: AtomicU32,
    role: AtomicU8,
    nonce: [AtomicU64; CEF_NONCE_BYTES / 8],
}

impl CefAuthoritySlot {
    pub(super) fn new() -> Self {
        Self {
            state: AtomicU8::new(SLOT_FREE),
            generation: AtomicU64::new(0),
            role: AtomicU8::new(0),
            pid: AtomicU32::new(0),
            nonce: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    pub(super) fn write_marker(&self, marker: &CefLaunchMarker) {
        self.role.store(u8::from(marker.role()), Ordering::Relaxed);
        for (target, value) in self.nonce.iter().zip(marker.nonce_words()) {
            target.store(value, Ordering::Relaxed);
        }
    }

    pub(super) fn matches(&self, publication: &CefPublication) -> bool {
        if self.generation.load(Ordering::Acquire) != publication.generation
            || self.role.load(Ordering::Acquire) != u8::from(publication.role)
        {
            return false;
        }
        let mut difference = 0_u64;
        for (stored, provided) in self.nonce.iter().zip(publication.nonce.chunks_exact(8)) {
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(provided);
            difference |= stored.load(Ordering::Acquire) ^ u64::from_le_bytes(bytes);
        }
        difference == 0
    }

    pub(super) fn clear_if(&self, generation: u64, expected: u8) -> bool {
        if self.generation.load(Ordering::Acquire) != generation
            || self
                .state
                .compare_exchange(expected, SLOT_CLEANING, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return false;
        }
        self.role.store(0, Ordering::Relaxed);
        self.pid.store(0, Ordering::Relaxed);
        for nonce in &self.nonce {
            nonce.store(0, Ordering::Relaxed);
        }
        self.state.store(SLOT_FREE, Ordering::Release);
        true
    }
}

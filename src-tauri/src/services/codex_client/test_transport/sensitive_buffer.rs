use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use zeroize::{Zeroize, Zeroizing};

pub(super) struct SensitiveBuffer {
    bytes: Zeroizing<Vec<u8>>,
    zeroized: Arc<AtomicBool>,
}

impl SensitiveBuffer {
    pub(super) fn with_capacity(capacity: usize, zeroized: Arc<AtomicBool>) -> Self {
        zeroized.store(false, Ordering::SeqCst);
        Self {
            bytes: Zeroizing::new(Vec::with_capacity(capacity)),
            zeroized,
        }
    }

    pub(super) fn zeroed(len: usize, zeroized: Arc<AtomicBool>) -> Self {
        zeroized.store(false, Ordering::SeqCst);
        Self {
            bytes: Zeroizing::new(vec![0; len]),
            zeroized,
        }
    }
}

impl Deref for SensitiveBuffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for SensitiveBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl Drop for SensitiveBuffer {
    fn drop(&mut self) {
        self.bytes.as_mut_slice().zeroize();
        let cleared = self.bytes.iter().all(|byte| *byte == 0);
        self.zeroized.store(cleared, Ordering::SeqCst);
    }
}

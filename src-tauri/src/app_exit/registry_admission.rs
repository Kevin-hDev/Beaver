#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "service producers consume tracked admissions in milestone 2"
    )
)]

use super::registry::AdmissionRegistry;
use std::future::Future;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AdmissionKey {
    pub(super) index: usize,
    pub(super) generation: u64,
}

pub(super) struct TrackedAdmission {
    pub(super) registry: AdmissionRegistry,
    pub(super) key: Option<AdmissionKey>,
    pub(super) cancel: CancellationToken,
}

impl TrackedAdmission {
    pub(super) fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    pub(super) async fn run<F>(self, future: F) -> F::Output
    where
        F: Future,
    {
        let guard = self;
        let output = future.await;
        drop(guard);
        output
    }

    #[cfg(test)]
    pub(super) fn key_for_test(&self) -> AdmissionKey {
        self.key.expect("tracked admission key")
    }
}

impl std::fmt::Debug for TrackedAdmission {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrackedAdmission")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl Drop for TrackedAdmission {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            let _ = self.registry.release(key);
        }
    }
}

pub(super) fn next_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

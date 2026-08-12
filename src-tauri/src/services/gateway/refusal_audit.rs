use super::types::ChannelKey;
use crate::services::work_registry::ServiceWorkCancellation;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub(super) const REFUSAL_AUDIT_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RefusalAuditOutcome {
    Queued,
    Full,
    Closed,
}

pub(super) struct RefusalAuditEntry {
    key: ChannelKey,
    decision: &'static str,
}

#[derive(Clone, Default)]
pub(super) struct RefusalCounter(Arc<AtomicU64>);

impl RefusalCounter {
    pub(super) fn total(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    fn increment(&self) {
        increment_saturating(&self.0);
    }
}

#[derive(Clone)]
pub(super) struct RefusalAudit {
    sender: mpsc::Sender<RefusalAuditEntry>,
    refused: RefusalCounter,
    dropped: Arc<AtomicU64>,
}

impl RefusalAudit {
    pub(super) fn channel() -> (Self, mpsc::Receiver<RefusalAuditEntry>) {
        let (sender, receiver) = mpsc::channel(REFUSAL_AUDIT_CAPACITY);
        (
            Self {
                sender,
                refused: RefusalCounter::default(),
                dropped: Arc::new(AtomicU64::new(0)),
            },
            receiver,
        )
    }

    pub(super) fn record_refusal(
        &self,
        key: ChannelKey,
        decision: &'static str,
    ) -> RefusalAuditOutcome {
        // The in-memory observation is authoritative even when persistent audit is disabled.
        self.refused.increment();
        self.try_record(key, decision)
    }

    pub(super) fn counter(&self) -> RefusalCounter {
        self.refused.clone()
    }

    pub(super) fn try_record(
        &self,
        key: ChannelKey,
        decision: &'static str,
    ) -> RefusalAuditOutcome {
        match self.sender.try_send(RefusalAuditEntry { key, decision }) {
            Ok(()) => RefusalAuditOutcome::Queued,
            Err(mpsc::error::TrySendError::Full(_)) => {
                increment_saturating(&self.dropped);
                RefusalAuditOutcome::Full
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                increment_saturating(&self.dropped);
                RefusalAuditOutcome::Closed
            }
        }
    }

    #[cfg(test)]
    pub(super) fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

pub(super) async fn run(
    mut receiver: mpsc::Receiver<RefusalAuditEntry>,
    cancel: ServiceWorkCancellation,
) {
    loop {
        let entry = tokio::select! {
            _ = cancel.cancelled() => return,
            entry = receiver.recv() => entry,
        };
        let Some(entry) = entry else {
            return;
        };
        let result = tokio::task::spawn_blocking(move || {
            super::service_audit::work_refused(&entry.key, entry.decision)
        })
        .await;
        if !matches!(result, Ok(Ok(()))) {
            ::log::warn!("[gateway] audit indisponible pour un refus de file");
        }
    }
}

fn increment_saturating(counter: &AtomicU64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value.saturating_add(1))
    });
}

#[cfg(test)]
pub(super) async fn run_with_writer_for_test(
    mut receiver: mpsc::Receiver<RefusalAuditEntry>,
    writer: impl FnOnce(ChannelKey, &'static str) -> bool + Send + 'static,
) {
    let Some(entry) = receiver.recv().await else {
        return;
    };
    let _ = tokio::task::spawn_blocking(move || writer(entry.key, entry.decision)).await;
}

use super::event_payload::{EventDraft, EventEnvelope};
use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

mod activity;

#[derive(Clone, Default)]
pub(super) struct EventRouter {
    state: Arc<Mutex<RouterState>>,
}

#[derive(Default)]
struct RouterState {
    deliveries: BTreeMap<HostIdentity, DeliveryEntry>,
    sequences: BTreeMap<String, u64>,
}

struct DeliveryEntry {
    generation: u64,
    subscriptions: BTreeSet<String>,
    delivery: EventDelivery,
}

#[derive(Clone)]
pub(super) struct EventDelivery {
    sender: tokio::sync::mpsc::Sender<EventEnvelope>,
    cancel: tokio_util::sync::CancellationToken,
    counters: Arc<EventCounters>,
}

#[derive(Default)]
struct EventCounters {
    queued: AtomicU64,
    delivered: AtomicU64,
    dropped: AtomicU64,
    timed_out: AtomicU64,
    active_handlers: AtomicU64,
}

impl EventRouter {
    pub(super) fn configured(
        &self,
        identity: &HostIdentity,
        generation: u64,
        subscriptions: &BTreeSet<String>,
    ) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .deliveries
            .get(identity)
            .is_some_and(|entry| {
                entry.generation == generation && entry.subscriptions == *subscriptions
            })
    }

    pub(super) fn install(
        &self,
        identity: HostIdentity,
        generation: u64,
        subscriptions: BTreeSet<String>,
        delivery: EventDelivery,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(previous) = state.deliveries.insert(
            identity,
            DeliveryEntry {
                generation,
                subscriptions,
                delivery,
            },
        ) {
            previous.delivery.cancel.cancel();
        }
    }

    pub(super) fn clear(&self, identity: &HostIdentity) {
        let previous = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .deliveries
            .remove(identity);
        if let Some(previous) = previous {
            previous.delivery.cancel.cancel();
        }
    }

    pub(super) fn publish(&self, draft: EventDraft) -> bool {
        if !super::types::EXTENSION_EVENTS.contains(&draft.event) {
            return false;
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.deliveries.is_empty() {
            return false;
        }
        let sequence = match next_sequence(&mut state.sequences, &draft) {
            Some(sequence) => sequence,
            None => return false,
        };
        let Some(envelope) = EventEnvelope::build(draft, sequence) else {
            return false;
        };
        for entry in state.deliveries.values() {
            if entry.subscriptions.contains(envelope.event) {
                entry.delivery.enqueue(envelope.clone());
            }
        }
        true
    }
}

fn next_sequence(sequences: &mut BTreeMap<String, u64>, draft: &EventDraft) -> Option<u64> {
    if draft.starts_flow {
        if sequences.contains_key(&draft.flow_id)
            || sequences.len() >= super::types::MAX_ACTIVE_CONTEXTS
        {
            return None;
        }
        sequences.insert(draft.flow_id.clone(), 1);
        return Some(1);
    }
    let sequence = sequences.get_mut(&draft.flow_id)?;
    *sequence = sequence.saturating_add(1);
    let value = *sequence;
    if draft.terminal {
        sequences.remove(&draft.flow_id);
    }
    Some(value)
}

impl EventDelivery {
    pub(super) fn start(
        process: Arc<HostProcess>,
        generation_cancel: tokio_util::sync::CancellationToken,
        work: &super::work_supervision::ExtensionWorkServices,
    ) -> Result<Self, super::work_supervision::ExtensionWorkAdmissionError> {
        let (sender, mut receiver) =
            tokio::sync::mpsc::channel(super::types::MAX_EVENT_QUEUE_PER_HOST);
        let cancel = tokio_util::sync::CancellationToken::new();
        let local_cancel = cancel.clone();
        let counters = Arc::new(EventCounters::default());
        let worker_counters = Arc::clone(&counters);
        work.spawn_event_worker(move |work_cancel| async move {
            loop {
                let envelope = tokio::select! {
                    _ = work_cancel.cancelled() => break,
                    _ = generation_cancel.cancelled() => break,
                    _ = local_cancel.cancelled() => break,
                    value = receiver.recv() => match value {
                        Some(value) => value,
                        None => break,
                    },
                };
                increment(&worker_counters.active_handlers);
                let Ok(payload) = serde_json::to_value(envelope) else {
                    worker_counters.active_handlers.store(0, Ordering::Release);
                    increment(&worker_counters.dropped);
                    continue;
                };
                let result = process
                    .request("event.emit", payload)
                    .await;
                worker_counters.active_handlers.store(0, Ordering::Release);
                match result {
                    Ok(_) => increment(&worker_counters.delivered),
                    Err(error) if error == super::error_codes::HOST_TIMEOUT => {
                        increment(&worker_counters.timed_out)
                    }
                    Err(_) => increment(&worker_counters.dropped),
                }
            }
        })?;
        Ok(Self {
            sender,
            cancel,
            counters,
        })
    }

    pub(super) fn enqueue(&self, envelope: EventEnvelope) {
        if self.cancel.is_cancelled() {
            increment(&self.counters.dropped);
            return;
        }
        match self.sender.try_send(envelope) {
            Ok(()) => increment(&self.counters.queued),
            Err(_) => increment(&self.counters.dropped),
        }
    }


    #[cfg(test)]
    pub(super) fn test_delivery() -> (Self, tokio::sync::mpsc::Receiver<EventEnvelope>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        (
            Self {
                sender,
                cancel: tokio_util::sync::CancellationToken::new(),
                counters: Arc::new(EventCounters::default()),
            },
            receiver,
        )
    }

    #[cfg(test)]
    pub(super) fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    #[cfg(test)]
    pub(super) fn dropped(&self) -> u64 {
        self.counters.dropped.load(Ordering::Acquire)
    }
}

fn increment(counter: &AtomicU64) {
    let _ = counter.fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
        Some(value.saturating_add(1))
    });
}

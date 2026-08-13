use crate::models::ScheduledWakeup;
use chrono::{DateTime, Local};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use super::work_supervision::SCHEDULED_WAKEUPS_CAPACITY;

#[derive(Debug, Eq, Hash, PartialEq)]
struct OccurrenceKey {
    wakeup_id: String,
    scheduled_for: String,
}

impl OccurrenceKey {
    fn new(wakeup_id: &str, scheduled_for: DateTime<Local>) -> Self {
        Self {
            wakeup_id: wakeup_id.to_string(),
            scheduled_for: scheduled_for.to_rfc3339(),
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct InFlightWakeups(Arc<Mutex<HashSet<OccurrenceKey>>>);

#[derive(Debug, Eq, PartialEq)]
pub(super) enum InFlightReservationError {
    Duplicate,
    Capacity,
}

pub(super) struct ReconciliationCandidates {
    pub(super) decidable: Vec<(ScheduledWakeup, DateTime<Local>)>,
    pub(super) has_in_flight: bool,
}

pub(super) struct InFlightWakeupGuard {
    registry: InFlightWakeups,
    key: Option<OccurrenceKey>,
}

impl InFlightWakeups {
    pub(super) fn reserve(
        &self,
        wakeup_id: &str,
        scheduled_for: DateTime<Local>,
    ) -> Result<InFlightWakeupGuard, InFlightReservationError> {
        let key = OccurrenceKey::new(wakeup_id, scheduled_for);
        let mut entries = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if entries.contains(&key) {
            return Err(InFlightReservationError::Duplicate);
        }
        if entries.len() >= SCHEDULED_WAKEUPS_CAPACITY {
            return Err(InFlightReservationError::Capacity);
        }
        entries.insert(OccurrenceKey {
            wakeup_id: key.wakeup_id.clone(),
            scheduled_for: key.scheduled_for.clone(),
        });
        Ok(InFlightWakeupGuard {
            registry: self.clone(),
            key: Some(key),
        })
    }

    pub(super) fn partition(
        &self,
        candidates: Vec<(ScheduledWakeup, DateTime<Local>)>,
    ) -> ReconciliationCandidates {
        let entries = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let mut decidable = Vec::with_capacity(candidates.len());
        let mut has_in_flight = false;
        for (wakeup, scheduled_for) in candidates {
            if entries.contains(&OccurrenceKey::new(&wakeup.id, scheduled_for)) {
                has_in_flight = true;
            } else {
                decidable.push((wakeup, scheduled_for));
            }
        }
        ReconciliationCandidates {
            decidable,
            has_in_flight,
        }
    }

    fn remove(&self, key: &OccurrenceKey) {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(key);
    }
}

impl Drop for InFlightWakeupGuard {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.registry.remove(&key);
        }
    }
}

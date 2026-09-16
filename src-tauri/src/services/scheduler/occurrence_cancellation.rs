use super::work_supervision::SCHEDULED_WAKEUPS_CAPACITY;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

static REGISTRY: LazyLock<Arc<OccurrenceCancellation>> =
    LazyLock::new(|| Arc::new(OccurrenceCancellation::default()));

#[derive(Default)]
pub(super) struct OccurrenceCancellation {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    active: HashMap<Uuid, ActiveOccurrence>,
    blocked_owners: HashSet<String>,
    blocked_automations: HashSet<Uuid>,
    extensions_closed: bool,
    globally_paused: bool,
}

struct ActiveOccurrence {
    automation_id: Uuid,
    owner_id: Option<String>,
    cancel: CancellationToken,
}

pub(super) struct OccurrenceGuard {
    occurrence_id: Uuid,
    registry: Arc<OccurrenceCancellation>,
}

pub(super) fn admit(
    occurrence_id: Uuid,
    automation_id: Uuid,
    owner_id: Option<&str>,
    parent: &CancellationToken,
) -> Result<(OccurrenceGuard, CancellationToken), ()> {
    REGISTRY.admit(occurrence_id, automation_id, owner_id, parent)
}

pub(super) fn revoke_owner(owner_id: &str) {
    REGISTRY.revoke_owner(owner_id);
}

pub(super) fn allow_owner(owner_id: &str) {
    REGISTRY.allow_owner(owner_id);
}

pub(super) fn cancel_automation(automation_id: Uuid) {
    REGISTRY.cancel_automation(automation_id);
}

pub(super) fn block_automation(automation_id: Uuid) {
    REGISTRY.block_automation(automation_id);
}

pub(super) fn allow_automation(automation_id: Uuid) {
    REGISTRY.allow_automation(automation_id);
}

pub(super) fn cancel_all() {
    REGISTRY.set_globally_paused(true);
}

pub(super) fn resume_all() {
    REGISTRY.set_globally_paused(false);
}

pub(super) fn owner_is_blocked(owner_id: &str) -> bool {
    REGISTRY.owner_is_blocked(owner_id)
}

impl OccurrenceCancellation {
    pub(super) fn admit(
        self: &Arc<Self>,
        occurrence_id: Uuid,
        automation_id: Uuid,
        owner_id: Option<&str>,
        parent: &CancellationToken,
    ) -> Result<(OccurrenceGuard, CancellationToken), ()> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.active.len() >= SCHEDULED_WAKEUPS_CAPACITY
            || state.active.contains_key(&occurrence_id)
            || state.blocked_automations.contains(&automation_id)
            || state.globally_paused
            || owner_id
                .is_some_and(|id| state.extensions_closed || state.blocked_owners.contains(id))
        {
            return Err(());
        }
        let cancel = parent.child_token();
        state.active.insert(
            occurrence_id,
            ActiveOccurrence {
                automation_id,
                owner_id: owner_id.map(str::to_string),
                cancel: cancel.clone(),
            },
        );
        Ok((
            OccurrenceGuard {
                occurrence_id,
                registry: Arc::clone(self),
            },
            cancel,
        ))
    }

    pub(super) fn revoke_owner(&self, owner_id: &str) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.blocked_owners.contains(owner_id)
            || state.blocked_owners.len() < crate::services::extensions::MAX_DISCOVERED_PLUGINS
        {
            state.blocked_owners.insert(owner_id.to_string());
        } else {
            // A corrupted registry over capacity must close admissions, never reopen them.
            state.extensions_closed = true;
        }
        for entry in state
            .active
            .values()
            .filter(|entry| entry.owner_id.as_deref() == Some(owner_id))
        {
            entry.cancel.cancel();
        }
    }

    pub(super) fn allow_owner(&self, owner_id: &str) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .blocked_owners
            .remove(owner_id);
    }

    fn owner_is_blocked(&self, owner_id: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .blocked_owners
            .contains(owner_id)
    }

    fn cancel_automation(&self, automation_id: Uuid) {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        for entry in state
            .active
            .values()
            .filter(|entry| entry.automation_id == automation_id)
        {
            entry.cancel.cancel();
        }
    }

    pub(super) fn block_automation(&self, automation_id: Uuid) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.blocked_automations.contains(&automation_id)
            || state.blocked_automations.len() < SCHEDULED_WAKEUPS_CAPACITY
        {
            state.blocked_automations.insert(automation_id);
        } else {
            state.globally_paused = true;
        }
        for entry in state
            .active
            .values()
            .filter(|entry| entry.automation_id == automation_id)
        {
            entry.cancel.cancel();
        }
    }

    pub(super) fn allow_automation(&self, automation_id: Uuid) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .blocked_automations
            .remove(&automation_id);
    }

    pub(super) fn set_globally_paused(&self, paused: bool) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.globally_paused = paused;
        if !paused {
            return;
        }
        for entry in state.active.values() {
            entry.cancel.cancel();
        }
    }
}

impl Drop for OccurrenceGuard {
    fn drop(&mut self) {
        self.registry
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .active
            .remove(&self.occurrence_id);
    }
}

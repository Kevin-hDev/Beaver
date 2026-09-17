use super::{EventDelivery, EventRouter};
use std::sync::atomic::Ordering;

impl EventRouter {
    pub(in crate::services::extensions) fn activity(
        &self,
    ) -> crate::services::extensions::types::ExtensionEventActivity {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let mut activity = crate::services::extensions::types::ExtensionEventActivity::default();
        for entry in state.deliveries.values() {
            activity.queued = activity.queued.saturating_add(entry.host_activity.queued);
            activity.delivered = activity
                .delivered
                .saturating_add(entry.host_activity.delivered);
            activity.dropped = activity
                .dropped
                .saturating_add(entry.host_activity.dropped)
                .saturating_add(entry.delivery.dropped_count());
            activity.timed_out = activity
                .timed_out
                .saturating_add(entry.host_activity.timed_out)
                .saturating_add(entry.delivery.timed_out_count());
            activity.active_handlers = activity
                .active_handlers
                .saturating_add(entry.host_activity.active_handlers);
        }
        activity.clamp();
        activity
    }

    pub(in crate::services::extensions) fn update_host_activity(
        &self,
        identity: &super::super::host_identity::HostIdentity,
        generation: u64,
        activity: crate::services::extensions::types::ExtensionEventActivity,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = state.deliveries.get_mut(identity) {
            if entry.generation == generation {
                entry.host_activity = activity;
            }
        }
    }
}

impl EventDelivery {
    fn dropped_count(&self) -> u64 {
        self.counters.dropped.load(Ordering::Acquire)
    }

    fn timed_out_count(&self) -> u64 {
        self.counters.timed_out.load(Ordering::Acquire)
    }
}

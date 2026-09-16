use super::{EventDelivery, EventRouter};
use std::sync::atomic::Ordering;

const MAX_SAFE_JAVASCRIPT_INTEGER: u64 = 9_007_199_254_740_991;

impl EventRouter {
    pub(in crate::services::extensions) fn activity(
        &self,
    ) -> crate::services::extensions::types::ExtensionEventActivity {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let mut activity = crate::services::extensions::types::ExtensionEventActivity::default();
        for entry in state.deliveries.values() {
            activity.queued = activity.queued.saturating_add(entry.delivery.queued_count());
            activity.delivered = activity
                .delivered
                .saturating_add(entry.delivery.delivered_count());
            activity.dropped = activity.dropped.saturating_add(entry.delivery.dropped_count());
            activity.timed_out = activity
                .timed_out
                .saturating_add(entry.delivery.timed_out_count());
            activity.active_handlers = activity
                .active_handlers
                .saturating_add(entry.delivery.active_handler_count());
        }
        activity.queued = activity.queued.min(MAX_SAFE_JAVASCRIPT_INTEGER);
        activity.delivered = activity.delivered.min(MAX_SAFE_JAVASCRIPT_INTEGER);
        activity.dropped = activity.dropped.min(MAX_SAFE_JAVASCRIPT_INTEGER);
        activity.timed_out = activity.timed_out.min(MAX_SAFE_JAVASCRIPT_INTEGER);
        activity.active_handlers = activity.active_handlers.min(MAX_SAFE_JAVASCRIPT_INTEGER);
        activity
    }
}

impl EventDelivery {
    fn queued_count(&self) -> u64 {
        self.counters.queued.load(Ordering::Acquire)
    }

    fn delivered_count(&self) -> u64 {
        self.counters.delivered.load(Ordering::Acquire)
    }

    fn dropped_count(&self) -> u64 {
        self.counters.dropped.load(Ordering::Acquire)
    }

    fn timed_out_count(&self) -> u64 {
        self.counters.timed_out.load(Ordering::Acquire)
    }

    fn active_handler_count(&self) -> u64 {
        self.counters.active_handlers.load(Ordering::Acquire)
    }
}

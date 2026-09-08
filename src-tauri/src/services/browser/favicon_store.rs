use super::{
    favicon_events::{snapshot, BrowserFaviconSnapshot},
    favicon_policy::{MAX_PENDING_SNAPSHOTS, MAX_REVISION},
    favicon_state::FaviconState,
};
use std::sync::Mutex;

#[derive(Default)]
struct Inner {
    state: FaviconState,
    before: Vec<BrowserFaviconSnapshot>,
    pending: Vec<BrowserFaviconSnapshot>,
}

#[derive(Default)]
pub(super) struct FaviconStore(Mutex<Inner>);

impl FaviconStore {
    pub(super) fn access<R>(
        &self,
        notify: bool,
        operation: impl FnOnce(&mut FaviconState) -> R,
    ) -> (R, Vec<BrowserFaviconSnapshot>) {
        let mut inner = match self.0.lock() {
            Ok(inner) => inner,
            Err(poison) => {
                let mut inner = poison.into_inner();
                let mut previous = std::mem::take(&mut inner.before);
                previous.extend(snapshots(&inner.state));
                previous.append(&mut inner.pending);

                // Never resume corrupted state. Keep final invalidations until an
                // app handle is available, including when a read first sees poison.
                inner.state = FaviconState::default();
                inner.state.revision = MAX_REVISION;
                inner.pending.clear();
                for old in previous {
                    if inner.pending.len() == MAX_PENDING_SNAPSHOTS {
                        break;
                    }
                    if !inner
                        .pending
                        .iter()
                        .any(|s| s.conversation_id == old.conversation_id)
                    {
                        let empty = snapshot(&inner.state, &old.conversation_id);
                        inner.pending.push(empty);
                    }
                }
                self.0.clear_poison();
                log::error!("[browser] favicon state disabled reason=poisoned_mutex");
                inner
            }
        };
        let before = snapshots(&inner.state);
        // Preserve pre-operation identities even if a panic partially clears state.
        inner.before = before.clone();
        let result = operation(&mut inner.state);
        inner.before.clear();
        for next in changes(&before, &inner.state) {
            inner
                .pending
                .retain(|old| old.conversation_id != next.conversation_id);
            // Pending notifications are bounded by the same global view budget.
            if inner.pending.len() >= MAX_PENDING_SNAPSHOTS {
                inner.pending.remove(0);
            }
            inner.pending.push(next);
        }
        let pending = if notify {
            std::mem::take(&mut inner.pending)
        } else {
            Vec::new()
        };
        (result, pending)
    }
}

fn snapshots(state: &FaviconState) -> Vec<BrowserFaviconSnapshot> {
    let mut ids: Vec<_> = state.entries.iter().map(|e| &e.key.session_id).collect();
    ids.sort();
    ids.dedup();
    ids.into_iter().map(|id| snapshot(state, id)).collect()
}

fn changes(before: &[BrowserFaviconSnapshot], state: &FaviconState) -> Vec<BrowserFaviconSnapshot> {
    let mut ids: Vec<_> = before.iter().map(|s| s.conversation_id.as_str()).collect();
    ids.extend(state.entries.iter().map(|e| e.key.session_id.as_str()));
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .filter_map(|id| {
            let next = snapshot(state, id);
            let old = before.iter().find(|s| s.conversation_id == id);
            let changed = old.map_or(!next.icons.is_empty(), |s| {
                s.icons != next.icons
                    || (s.revision != MAX_REVISION && next.revision == MAX_REVISION)
            });
            changed.then_some(next)
        })
        .collect()
}

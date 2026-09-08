use super::{
    browser_view_key::BrowserViewKey, favicon_policy::*, favicon_state::FaviconState,
    favicon_store::FaviconStore, favicon_task_gate::TaskReservation,
};
use std::{sync::atomic::AtomicBool, time::Instant};

fn key(n: usize) -> BrowserViewKey {
    BrowserViewKey {
        session_id: format!("session-{n}"),
        tab_id: n.to_string(),
    }
}
fn queue(state: &mut FaviconState, n: usize) {
    state.begin_document(key(n), 1);
    state.replace_candidates(
        &key(n),
        1,
        vec![
            "https://example.org/one".into(),
            "https://example.org/two".into(),
        ],
    );
}
fn image(state: &mut FaviconState, n: usize) {
    queue(state, n);
    let now = Instant::now();
    let job = state.take_ready(now).remove(0);
    state.complete(&job, Some("png".into()), now);
    state.finish_callback(&job);
}
#[test]
fn unavailable_host_preserves_candidate_and_failure_tries_next() {
    let mut state = FaviconState::default();
    queue(&mut state, 0);
    let now = Instant::now();
    assert!(state.take_available(now, |_, _| false).is_empty());
    let first = state.take_ready(now).remove(0);
    assert!(first.url.ends_with("/one"));
    state.complete(&first, None, now);
    state.finish_callback(&first);
    let second = state.take_ready(now).remove(0);
    assert!(second.url.ends_with("/two"));
    // A duplicate release of the first URL must not release the second permit.
    state.finish_callback(&first);
    assert!(state.take_ready(now).is_empty());
    let mut wrong = second.clone();
    wrong.ticket.request += 1;
    state.finish_callback(&wrong);
    assert!(state.take_ready(now).is_empty());
    state.complete(&second, Some("png".into()), now);
    assert_eq!(state.entries[0].png.as_deref(), Some("png"));
}
#[test]
fn global_cache_budget_applies_across_sessions() {
    let mut state = FaviconState::default();
    for n in 0..MAX_FAVICONS + 1 {
        queue(&mut state, n);
    }
    assert_eq!(state.entries.len(), MAX_FAVICONS);
    assert!(!state.entries.iter().any(|e| e.key == key(0)));
    assert!(state.entries.iter().any(|e| e.key == key(MAX_FAVICONS)));
}
#[test]
fn notifications_only_cover_changed_conversations_and_evictions() {
    let store = FaviconStore::default();
    let (_, events) = store.access(true, |state| {
        image(state, 0);
        image(state, 1);
    });
    assert_eq!(events.len(), 2);
    let (_, events) = store.access(true, |state| state.begin_document(key(0), 1));
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].conversation_id, key(0).session_id);
    assert!(events[0].icons.is_empty());
    assert!(store.access(true, |_| {}).1.is_empty());
    assert!(store
        .access(true, |state| state.begin_document(key(0), 1))
        .1
        .is_empty());
    let (_, events) = store.access(true, |state| {
        for n in 2..MAX_FAVICONS + 2 {
            queue(state, n);
        }
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].conversation_id, key(1).session_id);
    assert!(events[0].icons.is_empty());
}
#[test]
fn poisoned_store_invalidates_once_and_never_restarts_work() {
    let store = FaviconStore::default();
    store.access(true, |state| image(state, 0));
    let _ = std::panic::catch_unwind(|| {
        store.access(true, |state| {
            state.entries.clear();
            panic!("injected")
        })
    });
    // A read cannot consume the invalidation intended for the UI.
    assert_eq!(store.access(false, |state| state.revision).0, MAX_REVISION);
    let (_, events) = store.access(true, |state| queue(state, 0));
    assert_eq!(events.len(), 1);
    assert!(events[0].icons.is_empty());
    assert_eq!(events[0].revision, MAX_REVISION);
    assert!(store
        .access(true, |state| state.take_ready(Instant::now()))
        .0
        .is_empty());
    assert!(store.access(true, |_| {}).1.is_empty());
}
#[test]
fn dropped_tasks_rearm_without_releasing_a_newer_reservation() {
    let gate = AtomicBool::new(false);
    let first = TaskReservation::acquire(&gate).unwrap();
    assert!(TaskReservation::acquire(&gate).is_none());
    first.release();
    let second = TaskReservation::acquire(&gate).unwrap();
    drop(first);
    assert!(TaskReservation::acquire(&gate).is_none());
    drop(second);
    assert!(TaskReservation::acquire(&gate).is_some());
}
#[test]
fn closing_and_closed_views_reject_late_navigation() {
    let mut state = super::view_state::ViewState::default();
    assert!(!state.is_live());
    state.begin_creation();
    assert!(state.is_live());
    state.mark_ready();
    assert!(state.is_live());
    state.begin_closing();
    assert!(!state.is_live());
    state.mark_closed();
    assert!(!state.is_live());
}
#[test]
fn exact_png_limit_is_accepted() {
    let encoded = super::favicon_png::encode(64, 64, MAX_PNG_BYTES, |bytes| {
        bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        MAX_PNG_BYTES
    });
    assert!(encoded.is_some());
}
#[test]
fn valid_public_snapshot_read_uses_runtime_store() {
    let value = super::favicon_events::read_snapshot("1a2b3c4d").unwrap();
    assert_eq!(value.conversation_id, "1a2b3c4d");
    assert!(value.icons.is_empty());
}

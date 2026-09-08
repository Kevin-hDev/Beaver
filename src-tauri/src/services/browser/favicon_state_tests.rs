use super::{browser_view_key::BrowserViewKey, favicon_policy::*, favicon_state::FaviconState};
use std::time::Instant;

fn key(n: usize) -> BrowserViewKey {
    BrowserViewKey {
        session_id: format!("session-{}", n % 2),
        tab_id: n.to_string(),
    }
}
fn queue(state: &mut FaviconState, n: usize, epoch: u64) {
    state.begin_document(key(n), epoch);
    state.replace_candidates(&key(n), epoch, vec!["https://example.org/icon.png".into()]);
}
#[test]
fn navigation_and_recreated_views_reject_old_results() {
    let mut state = FaviconState::default();
    let now = Instant::now();
    queue(&mut state, 0, 1);
    let old = state.take_ready(now).remove(0);
    queue(&mut state, 0, 1); // A -> B
    queue(&mut state, 0, 1); // B -> A, even the same URL must not revive the old ticket.
    assert!(!state.is_current(&old, now));
    state.release_view(&key(0), 1);
    queue(&mut state, 0, 2);
    state.release_view(&key(0), 1);
    assert_eq!(state.entries.len(), 1);
    assert!(!state.is_current(&old, now));
}
#[test]
fn permits_survive_timeout_eviction_and_closure() {
    let mut state = FaviconState::default();
    let now = Instant::now();
    for n in 0..MAX_DOWNLOADS {
        queue(&mut state, n, n as u64 + 1);
    }
    let jobs = state.take_ready(now);
    assert_eq!(jobs.len(), MAX_DOWNLOADS);
    for n in 0..MAX_DOWNLOADS {
        state.release_view(&key(n), n as u64 + 1);
    }
    queue(&mut state, 9, 20);
    assert!(state.take_ready(now + DOWNLOAD_DEADLINE).is_empty());
    assert!(!state.is_current(&jobs[0], now + DOWNLOAD_DEADLINE));
    state.finish_callback(&jobs[0]);
    assert_eq!(state.take_ready(now).len(), 1);
}
#[test]
fn newest_list_wins_and_one_download_per_view() {
    let mut state = FaviconState::default();
    let now = Instant::now();
    queue(&mut state, 0, 1);
    let old = state.take_ready(now).remove(0);
    for name in ["first", "last"] {
        state.replace_candidates(&key(0), 1, vec![format!("https://example.org/{name}")]);
    }
    assert!(state.take_ready(now).is_empty());
    state.complete(&old, Some("obsolete".into()), now);
    assert!(state.entries[0].png.is_none());
    state.finish_callback(&old);
    let job = state.take_ready(now).remove(0);
    assert!(job.url.ends_with("/last"));
    state.complete(&job, Some("new".into()), now);
    assert_eq!(state.entries[0].png.as_deref(), Some("new"));
    assert!(!state.is_current(&job, now));
}
#[test]
fn cache_is_bounded_and_eviction_changes_revision() {
    let mut state = FaviconState::default();
    for n in 0..MAX_FAVICONS {
        queue(&mut state, n, n as u64 + 1);
    }
    let revision = state.revision;
    queue(&mut state, MAX_FAVICONS, 30);
    assert_eq!(state.entries.len(), MAX_FAVICONS);
    assert!(!state.entries.iter().any(|e| e.key == key(0)));
    assert!(state.revision > revision);
}
#[test]
fn overflow_fails_closed() {
    let mut state = FaviconState::default();
    queue(&mut state, 0, 1);
    let now = Instant::now();
    let job = state.take_ready(now).remove(0);
    state.revision = MAX_REVISION - 1;
    state.begin_document(key(0), 1);
    assert!(state.entries.is_empty());
    assert!(!state.is_current(&job, now));
}
#[test]
fn candidates_are_validated_deduplicated_and_bounded() {
    let urls = vec![
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:image/png;base64,AA",
        "https://u:p@example.org/",
        "http://localhost:8080/i",
        "http://localhost:8080/i",
        "https://cdn.example.org/a",
        "https://example.org/b",
        "https://example.org/ignored",
    ];
    assert_eq!(candidates(urls.into_iter().map(str::to_owned)).len(), 3);
    assert!(!valid_dimensions(0, 32));
    assert!(!valid_dimensions(65, 32));
    assert!(valid_dimensions(64, 64));
}

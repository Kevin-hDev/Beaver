use super::{
    browser_view_key::BrowserViewKey,
    runtime_revision::{RuntimeRevisionCache, RuntimeStamp},
    session_types::MAX_BROWSER_TABS,
};

#[test]
fn stale_runtime_updates_are_rejected() {
    let key = view_key(1);
    let mut cache = RuntimeRevisionCache::default();
    assert!(cache.accept_update(key.clone(), stamp(1, 2)));
    assert!(!cache.accept_update(key.clone(), stamp(1, 1)));
    assert!(cache.accept_update(key, stamp(1, 3)));
}

#[test]
fn release_blocks_late_callbacks_but_not_a_new_view_epoch() {
    let key = view_key(2);
    let mut cache = RuntimeRevisionCache::default();
    assert!(cache.accept_update(key.clone(), stamp(4, 1)));
    assert!(cache.accept_release(key.clone(), stamp(4, 2)));
    assert!(!cache.accept_update(key.clone(), stamp(4, 3)));
    assert!(cache.accept_update(key, stamp(5, 1)));
}

#[test]
fn runtime_revision_cache_is_bounded() {
    let mut cache = RuntimeRevisionCache::default();
    for index in 0..=MAX_BROWSER_TABS {
        assert!(cache.accept_update(view_key(index), stamp(1, 1)));
    }
    assert_eq!(cache.len(), MAX_BROWSER_TABS);
}

fn stamp(epoch: u64, revision: u64) -> RuntimeStamp {
    RuntimeStamp::new(epoch, revision).unwrap()
}

fn view_key(index: usize) -> BrowserViewKey {
    BrowserViewKey::new(format!("{index:032x}"), format!("{:032x}", index + 100)).unwrap()
}

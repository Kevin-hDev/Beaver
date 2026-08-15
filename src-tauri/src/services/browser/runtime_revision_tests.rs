use super::{
    browser_view_key::BrowserViewKey,
    runtime_revision::{RuntimeRevisionCache, RuntimeStamp},
    session_types::{BrowserRuntimeTabUpdate, MAX_BROWSER_TABS},
};

#[test]
fn out_of_order_updates_preserve_independent_fields() {
    let key = view_key(0);
    let mut cache = RuntimeRevisionCache::default();
    let mut loading = BrowserRuntimeTabUpdate {
        loading: Some(false),
        ..Default::default()
    };
    let mut title = BrowserRuntimeTabUpdate {
        title: Some("Beaver CEF smoke".to_string()),
        ..Default::default()
    };

    assert!(cache.filter_update(key.clone(), stamp(1, 2), &mut loading));
    assert!(cache.filter_update(key, stamp(1, 1), &mut title));
    assert_eq!(loading.loading, Some(false));
    assert_eq!(title.title.as_deref(), Some("Beaver CEF smoke"));
}

#[test]
fn stale_runtime_updates_are_rejected() {
    let key = view_key(1);
    let mut cache = RuntimeRevisionCache::default();
    assert!(filter_title(&mut cache, key.clone(), stamp(1, 2)));
    assert!(!filter_title(&mut cache, key.clone(), stamp(1, 1)));
    assert!(filter_title(&mut cache, key, stamp(1, 3)));
}

#[test]
fn stale_field_is_removed_without_discarding_a_fresh_field() {
    let key = view_key(3);
    let mut cache = RuntimeRevisionCache::default();
    assert!(filter_title(&mut cache, key.clone(), stamp(1, 2)));
    let mut mixed = BrowserRuntimeTabUpdate {
        title: Some("stale".to_string()),
        loading: Some(false),
        ..Default::default()
    };

    assert!(cache.filter_update(key, stamp(1, 1), &mut mixed));
    assert_eq!(mixed.title, None);
    assert_eq!(mixed.loading, Some(false));
}

#[test]
fn release_blocks_late_callbacks_but_not_a_new_view_epoch() {
    let key = view_key(2);
    let mut cache = RuntimeRevisionCache::default();
    assert!(filter_title(&mut cache, key.clone(), stamp(4, 1)));
    assert!(cache.accept_release(key.clone(), stamp(4, 2)));
    assert!(!filter_title(&mut cache, key.clone(), stamp(4, 3)));
    assert!(filter_title(&mut cache, key, stamp(5, 1)));
}

#[test]
fn runtime_revision_cache_is_bounded() {
    let mut cache = RuntimeRevisionCache::default();
    for index in 0..=MAX_BROWSER_TABS {
        assert!(filter_title(&mut cache, view_key(index), stamp(1, 1)));
    }
    assert_eq!(cache.len(), MAX_BROWSER_TABS);
}

fn filter_title(
    cache: &mut RuntimeRevisionCache,
    key: BrowserViewKey,
    stamp: RuntimeStamp,
) -> bool {
    let mut update = BrowserRuntimeTabUpdate {
        title: Some("title".to_string()),
        ..Default::default()
    };
    cache.filter_update(key, stamp, &mut update)
}

fn stamp(epoch: u64, revision: u64) -> RuntimeStamp {
    RuntimeStamp::new(epoch, revision).unwrap()
}

fn view_key(index: usize) -> BrowserViewKey {
    BrowserViewKey::new(format!("{index:032x}"), format!("{:032x}", index + 100)).unwrap()
}

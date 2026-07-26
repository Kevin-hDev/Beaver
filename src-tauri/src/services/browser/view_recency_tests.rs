use super::{browser_view_key::BrowserViewKey, view_recency::ViewRecency};

#[test]
fn keeps_ten_views_and_evicts_the_oldest_inactive_one() {
    let mut recency = ViewRecency::default();
    for index in 0..10 {
        assert!(recency.touch(key(index), None).is_none());
    }
    let protected = key(0);
    let evicted = recency.touch(key(10), Some(&protected)).unwrap();
    assert_eq!(evicted, key(1));
    assert_eq!(recency.len(), 10);
}

#[test]
fn touching_an_existing_view_does_not_grow_the_collection() {
    let mut recency = ViewRecency::default();
    recency.touch(key(1), None);
    recency.touch(key(1), None);
    assert_eq!(recency.len(), 1);
    recency.remove(&key(1));
    assert_eq!(recency.len(), 0);
}

fn key(index: usize) -> BrowserViewKey {
    BrowserViewKey::new(format!("{index:032x}"), format!("{:032x}", index + 100)).unwrap()
}

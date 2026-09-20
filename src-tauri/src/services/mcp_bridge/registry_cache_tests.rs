use std::time::{Duration, Instant};

use super::registry_cache::{CacheState, FALLBACK_TTL};
use super::transport::McpToolCatalog;

fn catalog(ttl: Option<Duration>) -> McpToolCatalog {
    McpToolCatalog {
        tools: Vec::new(),
        cache_ttl: ttl,
    }
}

#[test]
fn old_account_cannot_repopulate_the_cache() {
    let mut cache = CacheState::default();
    let old_generation = cache.generation();
    cache.invalidate("notion");
    let now = Instant::now();
    assert!(cache
        .publish("notion", old_generation, catalog(Some(FALLBACK_TTL)), now)
        .is_err());
    assert!(cache.get("notion", cache.generation(), now).is_none());
}

#[test]
fn expiration_and_no_cache_are_exact() {
    let mut cache = CacheState::default();
    let now = Instant::now();
    let generation = cache.generation();
    cache
        .publish("notion", generation, catalog(Some(FALLBACK_TTL)), now)
        .unwrap();
    assert!(cache
        .get(
            "notion",
            generation,
            now + FALLBACK_TTL - Duration::from_nanos(1)
        )
        .is_some());
    assert!(cache
        .get("notion", generation, now + FALLBACK_TTL)
        .is_none());
    assert!(cache
        .publish("notion", generation + 1, catalog(None), now)
        .is_err());
    assert!(cache.get("notion", generation, now).is_some());
    cache
        .publish("notion", generation, catalog(None), now)
        .unwrap();
    assert!(cache.get("notion", generation, now).is_none());
}

#[test]
fn thirty_third_connector_evicts_oldest_publication() {
    let mut cache = CacheState::default();
    let now = Instant::now();
    let generation = cache.generation();
    for index in 0..33 {
        cache
            .publish(
                &format!("connector-{index}"),
                generation,
                catalog(Some(FALLBACK_TTL)),
                now + Duration::from_millis(index),
            )
            .unwrap();
    }
    assert!(cache.get("connector-0", generation, now).is_none());
    assert!(cache.get("connector-1", generation, now).is_some());
    assert!(cache.get("connector-32", generation, now).is_some());
}

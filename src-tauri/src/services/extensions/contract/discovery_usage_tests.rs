use super::*;

#[test]
fn score_loses_twenty_percent_per_week() {
    let entry = UsageEntry {
        score: 10.0,
        updated_at: 1_000,
    };
    let after_one_week = decayed(&entry, 1_000 + WEEK_SECONDS as i64);

    assert!((after_one_week - 8.0).abs() < 0.000_001);
}

#[test]
fn pruning_removes_tiny_and_invalid_entries() {
    let mut ledger = UsageLedger::default();
    ledger.entries.insert(
        "example.old".to_string(),
        UsageEntry {
            score: 0.001,
            updated_at: 1_000,
        },
    );
    ledger.entries.insert(
        "bad id".to_string(),
        UsageEntry {
            score: 5.0,
            updated_at: 1_000,
        },
    );

    prune(&mut ledger, 1_000);

    assert!(ledger.entries.is_empty());
}

#[test]
fn eviction_removes_the_lowest_decayed_score() {
    let mut ledger = UsageLedger::default();
    ledger.entries.insert(
        "example.low".to_string(),
        UsageEntry {
            score: 1.0,
            updated_at: 1_000,
        },
    );
    ledger.entries.insert(
        "example.high".to_string(),
        UsageEntry {
            score: 5.0,
            updated_at: 1_000,
        },
    );

    evict_lowest(&mut ledger, 1_000);

    assert!(!ledger.entries.contains_key("example.low"));
    assert!(ledger.entries.contains_key("example.high"));
}

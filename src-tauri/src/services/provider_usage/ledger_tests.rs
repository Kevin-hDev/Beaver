use super::*;
use crate::services::provider_usage::ledger_aggregate;
use crate::services::provider_usage::pricing::ResolvedCost;
use crate::services::provider_usage::request_usage::RequestUsage;

#[test]
fn invalid_file_recovers_to_empty_ledger() {
    let ledger = decode(b"not-json");
    assert!(ledger.connections.is_empty());
}

#[test]
fn stale_or_future_remote_data_is_rejected() {
    let now = 2_000_000;
    assert!(is_recent_timestamp(now - 86_400, now));
    assert!(!is_recent_timestamp(now - 86_401, now));
    assert!(!is_recent_timestamp(now + 301, now));
    assert!(!is_recent_timestamp(0, now));
}

#[test]
fn days_are_bounded_and_oldest_is_evicted() {
    let mut days = BTreeMap::new();
    for index in 0..=DAY_LIMIT {
        days.insert(format!("{index:04}-01-01"), UsageBreakdown::default());
    }
    prune_days(&mut days);
    assert_eq!(days.len(), DAY_LIMIT);
    assert!(!days.contains_key("0000-01-01"));
}

#[test]
fn connections_are_bounded_and_oldest_is_evicted() {
    let mut ledger = Ledger::default();
    for index in 0..CONNECTION_LIMIT {
        ledger.connections.insert(
            format!("connection-{index}"),
            ConnectionLedger {
                updated_at: index as i64,
                ..Default::default()
            },
        );
    }
    connection_mut(&mut ledger, "new-connection", 100);
    assert_eq!(ledger.connections.len(), CONNECTION_LIMIT);
    assert!(!ledger.connections.contains_key("connection-0"));
}

#[test]
fn clearing_remote_preserves_local_totals() {
    let mut ledger = Ledger::default();
    let mut connection = ConnectionLedger {
        last_remote: Some(RemoteData::default()),
        last_remote_generation: Some(8),
        ..Default::default()
    };
    connection.all_time.totals.request_count = 7;
    ledger.connections.insert("xai".into(), connection);
    assert!(clear_remote_in(&mut ledger, "xai", 42));
    let connection = ledger.connections.get("xai").unwrap();
    assert!(connection.last_remote.is_none());
    assert!(connection.last_remote_generation.is_none());
    assert_eq!(connection.all_time.totals.request_count, 7);
    assert_eq!(connection.updated_at, 42);
}

#[test]
fn cache_and_reasoning_tokens_are_subtotals() {
    let usage = RequestUsage {
        input_tokens: Some(100),
        output_tokens: Some(50),
        cached_input_tokens: Some(40),
        reasoning_output_tokens: Some(20),
        total_tokens: Some(150),
        ..Default::default()
    };
    let mut total = UsageBreakdown::default();
    ledger_aggregate::add(
        &mut total,
        UsageOrigin::ManualChat,
        UsageWorkload::Primary,
        &usage,
        ResolvedCost {
            micros: Some(10),
            exact: true,
        },
    );
    assert_eq!(total.totals.tokens.total_tokens, 150);
    assert_eq!(total.totals.tokens.cached_input_tokens, 40);
    assert_eq!(total.totals.tokens.reasoning_output_tokens, 20);
    assert_eq!(total.totals.cost_usd_micros, 10);
    assert_eq!(total.totals.exact_cost_request_count, 1);
    assert_eq!(total.totals.usage_request_count, 1);

    ledger_aggregate::add(
        &mut total,
        UsageOrigin::ManualChat,
        UsageWorkload::Primary,
        &RequestUsage::default(),
        ResolvedCost::default(),
    );
    assert_eq!(total.totals.request_count, 2);
    assert_eq!(total.totals.usage_request_count, 1);
    assert_eq!(total.totals.tokens.total_tokens, 150);
}

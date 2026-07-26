use super::*;

fn meta() -> ForecastAnalysisMeta {
    serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Analyse",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos-bolt-tiny",
        "provider": "local",
        "horizon": 1,
        "frequency": "D",
        "points": 4,
        "mape": null,
        "session_id": null
    }))
    .unwrap()
}

#[test]
fn legacy_array_is_hydrated_only_during_migration() {
    let data = serde_json::to_vec(&vec![meta()]).unwrap();

    let state = parse(&data).unwrap();

    assert!(state.needs_hydration);
    assert_eq!(state.entries.len(), 1);
}

#[test]
fn versioned_index_is_lightweight_on_every_regular_read() {
    let data = serde_json::to_vec(&ForecastIndex {
        schema_version: INDEX_SCHEMA_VERSION,
        entries: vec![meta()],
    })
    .unwrap();

    let state = parse(&data).unwrap();

    assert!(!state.needs_hydration);
    assert_eq!(state.entries[0].scenarios_count, 0);
}

#[test]
fn corrupt_duplicate_entries_are_rejected() {
    let item = meta();
    let data = serde_json::to_vec(&vec![item.clone(), item]).unwrap();

    assert!(parse(&data).is_err());
}

#[test]
fn inserting_past_the_limit_evicts_only_the_oldest_analysis() {
    let mut entries = (0..MAX_STORED_ANALYSES)
        .map(|_| {
            let mut item = meta();
            item.id = uuid::Uuid::new_v4().to_string();
            item
        })
        .collect::<Vec<_>>();
    let oldest_id = entries[0].id.clone();
    let mut inserted = meta();
    inserted.id = uuid::Uuid::new_v4().to_string();
    let inserted_id = inserted.id.clone();

    let removed = upsert_entries(&mut entries, inserted);

    assert_eq!(removed, vec![oldest_id]);
    assert_eq!(entries.len(), MAX_STORED_ANALYSES);
    assert_eq!(entries.last().map(|item| &item.id), Some(&inserted_id));
}

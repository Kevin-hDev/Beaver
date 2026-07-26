use super::types::{default_revision, default_schema_version, ForecastResult};

#[test]
fn legacy_analysis_gets_explicit_defaults() {
    let analysis: ForecastResult = serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Legacy",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos-bolt-small",
        "provider": "chronos-bolt",
        "horizon": 1,
        "frequency": "D",
        "input_summary": {"points": 1, "start": "2025-01-01", "end": "2025-01-01"},
        "predictions": [{"date": "2025-01-02", "value": 1.0}],
        "quantiles": {"q10": [0.5], "q50": [1.0], "q90": [1.5]}
    }))
    .unwrap();

    assert_eq!(analysis.schema_version, default_schema_version());
    assert_eq!(analysis.revision, default_revision());
    assert!(analysis.advanced_analytics.is_none());
    assert!(analysis.ensemble.is_none());
    assert_eq!(
        analysis.provenance.selection_source,
        super::provenance_types::ForecastSelectionSource::Manual
    );
}

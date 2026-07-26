use super::data_quality::DataProfile;
use super::evaluation::types::BacktestIndexSummary;
use super::types::ForecastAnalysisMeta;

pub fn comparable(
    entries: Vec<ForecastAnalysisMeta>,
    profile: &DataProfile,
) -> Result<Vec<BacktestIndexSummary>, String> {
    if profile.fingerprint.is_empty() {
        return Ok(Vec::new());
    }
    let mut summaries: Vec<_> = entries
        .into_iter()
        .filter(|entry| {
            entry.data_fingerprint == profile.fingerprint
                && entry.horizon == profile.horizon
                && entry.frequency == profile.frequency
                && entry
                    .confidence_level
                    .zip(profile.confidence_level)
                    .is_some_and(|(entry_level, profile_level)| {
                        (entry_level - profile_level).abs() < 0.000_001
                    })
        })
        .filter_map(|entry| entry.backtest)
        .collect();
    summaries.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    summaries.truncate(super::limits::MAX_AUTO_BACKTEST_SUMMARIES);
    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> DataProfile {
        serde_json::from_value(serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "created_at": "2026-01-01T00:00:00Z",
            "fingerprint": "a".repeat(64),
            "valid": true,
            "target_column": "value",
            "date_column": "date",
            "series_column": null,
            "covariate_columns": [],
            "frequency": "D",
            "horizon": 12,
            "confidence_level": 0.8,
            "row_count": 20,
            "history_points": 20,
            "future_rows": 0,
            "series_count": 1,
            "series_ids": [],
            "history_points_by_series": {},
            "start": "2025-01-01",
            "end": "2025-01-20",
            "missing_periods": 0,
            "outlier_count": 0,
            "issues": []
        }))
        .unwrap()
    }

    fn meta(fingerprint: &str) -> ForecastAnalysisMeta {
        ForecastAnalysisMeta {
            id: uuid::Uuid::new_v4().to_string(),
            name: "analysis".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            model: "chronos-bolt-tiny".into(),
            provider: "chronos-bolt".into(),
            horizon: 12,
            frequency: "D".into(),
            confidence_level: Some(0.8),
            points: 20,
            mape: None,
            session_id: None,
            scenarios_count: 0,
            data_profile_id: Some("550e8400-e29b-41d4-a716-446655440000".into()),
            data_fingerprint: fingerprint.into(),
            backtest: Some(BacktestIndexSummary::default()),
        }
    }

    #[test]
    fn only_reuses_evidence_for_the_exact_input() {
        let profile = profile();
        let entries = vec![meta(&"b".repeat(64)), meta(&profile.fingerprint)];

        assert_eq!(comparable(entries, &profile).unwrap().len(), 1);
    }

    #[test]
    fn a_new_audit_can_reuse_the_same_input_fingerprint() {
        let profile = profile();
        let mut entry = meta(&profile.fingerprint);
        entry.data_profile_id = Some(uuid::Uuid::new_v4().to_string());

        assert_eq!(comparable(vec![entry], &profile).unwrap().len(), 1);
    }

    #[test]
    fn evidence_collection_is_bounded() {
        let profile = profile();
        let entries = (0..50).map(|_| meta(&profile.fingerprint)).collect();

        assert_eq!(
            comparable(entries, &profile).unwrap().len(),
            super::super::limits::MAX_AUTO_BACKTEST_SUMMARIES
        );
    }

    #[test]
    fn newest_evaluation_is_returned_first() {
        let profile = profile();
        let mut old = meta(&profile.fingerprint);
        old.backtest.as_mut().unwrap().created_at = "2026-01-01T00:00:00Z".into();
        let mut recent = meta(&profile.fingerprint);
        recent.backtest.as_mut().unwrap().created_at = "2026-02-01T00:00:00Z".into();

        let result = comparable(vec![recent, old], &profile).unwrap();

        assert_eq!(result[0].created_at, "2026-02-01T00:00:00Z");
    }
}

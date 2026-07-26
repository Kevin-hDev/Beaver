use super::limits::MAX_QUANTILE_LEVELS;

pub fn requested_levels(confidence: f64) -> Vec<f64> {
    let lower = stable_level((1.0 - confidence) / 2.0);
    let upper = stable_level(1.0 - lower);
    let mut levels = vec![lower, 0.5, upper];
    levels.sort_by(f64::total_cmp);
    levels.dedup_by(|left, right| (*left - *right).abs() < 0.000_001);
    levels.truncate(MAX_QUANTILE_LEVELS);
    levels
}

pub fn configured_levels(
    confidence: f64,
    config: &serde_json::Map<String, serde_json::Value>,
) -> Vec<f64> {
    let required = requested_levels(confidence);
    let mut extras = Vec::new();
    if let Some(configured) = config
        .get("quantiles")
        .and_then(serde_json::Value::as_array)
    {
        extras.extend(
            configured
                .iter()
                .filter_map(serde_json::Value::as_f64)
                .filter(|level| {
                    level.is_finite()
                        && *level > 0.0
                        && *level < 1.0
                        && !required
                            .iter()
                            .any(|item| (*item - *level).abs() < 0.000_001)
                }),
        );
    }
    extras.sort_by(f64::total_cmp);
    extras.dedup_by(|left, right| (*left - *right).abs() < 0.000_001);
    extras.truncate(MAX_QUANTILE_LEVELS.saturating_sub(required.len()));
    let mut levels = required;
    levels.extend(extras);
    levels.sort_by(f64::total_cmp);
    levels
}

pub fn lower_level(confidence: f64) -> f64 {
    stable_level((1.0 - confidence) / 2.0)
}

pub fn upper_level(confidence: f64) -> f64 {
    stable_level(1.0 - lower_level(confidence))
}

pub fn quantile_key(level: f64) -> String {
    let basis_points = (level * 10_000.0).round() as u32;
    if basis_points.is_multiple_of(100) {
        return format!("q{:02}", basis_points / 100);
    }
    format!("q{basis_points:04}")
}

fn stable_level(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_central_interval_from_confidence() {
        assert_eq!(requested_levels(0.9), vec![0.05, 0.5, 0.95]);
        assert_eq!(quantile_key(lower_level(0.8)), "q10");
        assert_eq!(quantile_key(lower_level(0.99)), "q0050");
    }

    #[test]
    fn keeps_configured_levels_without_losing_selected_interval() {
        let config = serde_json::json!({ "quantiles": [0.1, 0.5, 0.9] });
        let levels = configured_levels(0.9, config.as_object().unwrap());
        assert_eq!(levels, vec![0.05, 0.1, 0.5, 0.9, 0.95]);
    }

    #[test]
    fn never_truncates_the_selected_interval() {
        let configured: Vec<_> = (1..=20).map(|value| value as f64 / 100.0).collect();
        let config = serde_json::json!({ "quantiles": configured });
        let levels = configured_levels(0.9, config.as_object().unwrap());

        assert_eq!(levels.len(), MAX_QUANTILE_LEVELS);
        assert!(levels.contains(&0.05));
        assert!(levels.contains(&0.5));
        assert!(levels.contains(&0.95));
    }
}

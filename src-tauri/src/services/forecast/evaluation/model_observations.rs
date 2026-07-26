use super::folds::RollingFold;
use super::metrics::{self, Observation};
use super::types::BacktestFoldMetric;
use crate::services::forecast::types::ForecastResult;
use std::collections::BTreeMap;

pub(super) fn collect(
    fold: &RollingFold,
    forecast: &ForecastResult,
    period: usize,
) -> Result<(Vec<Observation>, BacktestFoldMetric), String> {
    let scales: BTreeMap<String, f64> = fold
        .series
        .iter()
        .map(|series| {
            (
                series.series_id.clone().unwrap_or_default(),
                metrics::scale(&series.training, period),
            )
        })
        .collect();
    let predicted: BTreeMap<String, (f64, f64, f64)> = forecast
        .predictions
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let lower = forecast
                .quantiles
                .q10
                .get(index)
                .copied()
                .unwrap_or(point.value);
            let upper = forecast
                .quantiles
                .q90
                .get(index)
                .copied()
                .unwrap_or(point.value);
            (
                key(point.series_id.as_deref(), &point.date),
                (point.value, lower, upper),
            )
        })
        .collect();
    let observations: Vec<_> = fold
        .actual
        .iter()
        .map(|actual| {
            let (point, lower, upper) = predicted
                .get(&key(actual.series_id.as_deref(), &actual.date))
                .copied()
                .ok_or("incomplete_predictions")?;
            let scale = scales
                .get(actual.series_id.as_deref().unwrap_or_default())
                .copied()
                .ok_or("missing_series")?;
            Ok(Observation {
                actual: actual.value,
                predicted: point,
                lower,
                upper,
                scale,
                fold: fold.index,
            })
        })
        .collect::<Result<_, String>>()?;
    let mae = observations
        .iter()
        .map(|item| (item.actual - item.predicted).abs())
        .sum::<f64>()
        / observations.len().max(1) as f64;
    let train_points = fold.series.iter().map(|series| series.training.len()).sum();
    let metric = BacktestFoldMetric {
        index: fold.index,
        train_points,
        test_points: observations.len(),
        mae,
    };
    Ok((observations, metric))
}

fn key(series_id: Option<&str>, date: &str) -> String {
    format!("{}\u{1f}{date}", series_id.unwrap_or_default())
}

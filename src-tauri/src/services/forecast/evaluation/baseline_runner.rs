use super::baselines::Baseline;
use super::folds::BacktestPlan;
use super::metrics::{self, Observation};
use super::types::{BacktestFailure, BacktestFoldMetric, BacktestKind, ModelBacktestResult};
use std::time::Instant;

pub(super) fn evaluate(
    baseline: Baseline,
    plan: &BacktestPlan,
    period: usize,
    confidence: f64,
) -> ModelBacktestResult {
    let started = Instant::now();
    let outcome = collect_observations(baseline, plan, period, confidence).and_then(
        |(observations, folds)| {
            let metrics = metrics::summarize(&observations, confidence)
                .ok_or_else(|| "invalid_backtest_data".to_string())?;
            Ok((
                metrics,
                metrics::calibration(&observations, confidence),
                folds,
            ))
        },
    );
    match outcome {
        Ok((metrics, calibration, folds)) => ModelBacktestResult {
            model_id: baseline.id().to_string(),
            kind: BacktestKind::Baseline,
            metrics: Some(metrics),
            calibration,
            folds,
            duration_ms: elapsed_ms(started),
            max_memory_mb: None,
            rank: None,
            beats_best_baseline: None,
            warning: None,
            failure: None,
        },
        Err(code) => {
            let failure = BacktestFailure::from_code(&code);
            ModelBacktestResult {
                model_id: baseline.id().to_string(),
                kind: BacktestKind::Baseline,
                metrics: None,
                calibration: None,
                folds: Vec::new(),
                duration_ms: elapsed_ms(started),
                max_memory_mb: None,
                rank: None,
                beats_best_baseline: None,
                warning: Some(failure.code.clone()),
                failure: Some(failure),
            }
        }
    }
}

fn collect_observations(
    baseline: Baseline,
    plan: &BacktestPlan,
    period: usize,
    confidence: f64,
) -> Result<(Vec<Observation>, Vec<BacktestFoldMetric>), String> {
    let mut observations = Vec::new();
    let mut fold_metrics = Vec::new();
    for fold in &plan.folds {
        let start = observations.len();
        let train_points = fold.series.iter().map(|series| series.training.len()).sum();
        for series in &fold.series {
            let predictions = baseline.forecast(&series.training, series.actual.len(), period)?;
            let residuals = baseline.residuals(&series.training, period);
            let half_width = metrics::quantile(&residuals, confidence);
            let scale = metrics::scale(&series.training, period);
            observations.extend(series.actual.iter().zip(predictions).map(
                |(actual, predicted)| Observation {
                    actual: *actual,
                    predicted,
                    lower: predicted - half_width,
                    upper: predicted + half_width,
                    scale,
                    fold: fold.index,
                },
            ));
        }
        let current = &observations[start..];
        let mae = current
            .iter()
            .map(|item| (item.actual - item.predicted).abs())
            .sum::<f64>()
            / current.len().max(1) as f64;
        fold_metrics.push(BacktestFoldMetric {
            index: fold.index,
            train_points,
            test_points: current.len(),
            mae,
        });
    }
    Ok((observations, fold_metrics))
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

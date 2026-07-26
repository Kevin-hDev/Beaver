use std::cmp::Ordering;

use super::types::{BacktestMetrics, IntervalCalibration, ModelBacktestResult};

pub(crate) fn compare_results(left: &ModelBacktestResult, right: &ModelBacktestResult) -> Ordering {
    match (&left.metrics, &right.metrics) {
        (Some(left_metrics), Some(right_metrics)) => compare_quality(
            left_metrics,
            left.calibration.as_ref(),
            left.duration_ms,
            left.max_memory_mb,
            right_metrics,
            right.calibration.as_ref(),
            right.duration_ms,
            right.max_memory_mb,
        )
        .then_with(|| left.model_id.cmp(&right.model_id)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left.model_id.cmp(&right.model_id),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compare_quality(
    left: &BacktestMetrics,
    left_calibration: Option<&IntervalCalibration>,
    left_duration_ms: u64,
    left_memory_mb: Option<u64>,
    right: &BacktestMetrics,
    right_calibration: Option<&IntervalCalibration>,
    right_duration_ms: u64,
    right_memory_mb: Option<u64>,
) -> Ordering {
    left.mase
        .total_cmp(&right.mase)
        .then_with(|| {
            optional_metric(left.quantile_loss).total_cmp(&optional_metric(right.quantile_loss))
        })
        .then_with(|| coverage_gap(left_calibration).total_cmp(&coverage_gap(right_calibration)))
        .then_with(|| left.smape.total_cmp(&right.smape))
        .then_with(|| left.mae.total_cmp(&right.mae))
        .then_with(|| left.rmse.total_cmp(&right.rmse))
        .then_with(|| left.bias.abs().total_cmp(&right.bias.abs()))
        .then_with(|| left.stability.total_cmp(&right.stability))
        .then_with(|| {
            left_memory_mb
                .unwrap_or(u64::MAX)
                .cmp(&right_memory_mb.unwrap_or(u64::MAX))
        })
        .then_with(|| left_duration_ms.cmp(&right_duration_ms))
}

fn optional_metric(value: Option<f64>) -> f64 {
    value
        .filter(|item| item.is_finite())
        .unwrap_or(f64::INFINITY)
}

fn coverage_gap(calibration: Option<&IntervalCalibration>) -> f64 {
    calibration.map_or(f64::INFINITY, |value| {
        (value.measured_coverage - value.theoretical_coverage).abs()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(quantile_loss: f64) -> BacktestMetrics {
        BacktestMetrics {
            mase: 0.5,
            smape: 1.0,
            mae: 1.0,
            rmse: 1.0,
            bias: 0.0,
            stability: 0.0,
            quantile_loss: Some(quantile_loss),
        }
    }

    #[test]
    fn probabilistic_quality_breaks_equal_accuracy_ties() {
        assert!(compare_quality(
            &metrics(0.1),
            None,
            20,
            Some(500),
            &metrics(0.2),
            None,
            10,
            Some(100),
        )
        .is_lt());
    }

    #[test]
    fn observed_memory_breaks_otherwise_equal_ties() {
        let metrics = metrics(0.1);
        assert!(
            compare_quality(&metrics, None, 20, Some(100), &metrics, None, 20, Some(500),).is_lt()
        );
    }
}

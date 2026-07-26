use super::types::{BacktestMetrics, IntervalCalibration};

#[derive(Debug, Clone)]
pub(super) struct Observation {
    pub actual: f64,
    pub predicted: f64,
    pub lower: f64,
    pub upper: f64,
    pub scale: f64,
    pub fold: usize,
}

pub(super) fn summarize(observations: &[Observation], confidence: f64) -> Option<BacktestMetrics> {
    if observations.is_empty() || observations.iter().any(invalid) {
        return None;
    }
    let count = observations.len() as f64;
    let absolute: Vec<f64> = observations
        .iter()
        .map(|item| (item.predicted - item.actual).abs())
        .collect();
    let mae = absolute.iter().sum::<f64>() / count;
    let rmse = (observations
        .iter()
        .map(|item| (item.predicted - item.actual).powi(2))
        .sum::<f64>()
        / count)
        .sqrt();
    let bias = observations
        .iter()
        .map(|item| item.predicted - item.actual)
        .sum::<f64>()
        / count;
    let smape = observations.iter().map(smape_item).sum::<f64>() / count;
    let mase = observations
        .iter()
        .map(|item| (item.predicted - item.actual).abs() / item.scale)
        .sum::<f64>()
        / count;
    let fold_maes = fold_maes(observations);
    let stability = standard_deviation(&fold_maes);
    let quantile_loss = mean_quantile_loss(observations, confidence)?;
    Some(BacktestMetrics {
        mase,
        smape,
        mae,
        rmse,
        bias,
        stability,
        quantile_loss: Some(quantile_loss),
    })
}

fn mean_quantile_loss(observations: &[Observation], confidence: f64) -> Option<f64> {
    if !crate::services::forecast::interval_capability::valid_input_level(confidence) {
        return None;
    }
    let lower_level = crate::services::forecast::intervals::lower_level(confidence);
    let upper_level = crate::services::forecast::intervals::upper_level(confidence);
    let total = observations
        .iter()
        .map(|item| {
            pinball(item.actual, item.lower, lower_level)
                + pinball(item.actual, item.predicted, 0.5)
                + pinball(item.actual, item.upper, upper_level)
        })
        .sum::<f64>();
    let value = total / (observations.len() * 3) as f64;
    value.is_finite().then_some(value)
}

fn pinball(actual: f64, predicted_quantile: f64, level: f64) -> f64 {
    let error = actual - predicted_quantile;
    if error >= 0.0 {
        level * error
    } else {
        (1.0 - level) * -error
    }
}

pub(super) fn calibration(
    observations: &[Observation],
    confidence: f64,
) -> Option<IntervalCalibration> {
    if observations.is_empty() || observations.iter().any(invalid) {
        return None;
    }
    let count = observations.len();
    let covered = observations
        .iter()
        .filter(|item| item.actual >= item.lower && item.actual <= item.upper)
        .count();
    let mean_width = observations
        .iter()
        .map(|item| item.upper - item.lower)
        .sum::<f64>()
        / count as f64;
    let residuals: Vec<f64> = observations
        .iter()
        .map(|item| (item.actual - item.predicted).abs())
        .collect();
    Some(IntervalCalibration {
        theoretical_coverage: confidence,
        measured_coverage: covered as f64 / count as f64,
        mean_width,
        residual_half_width: quantile(&residuals, confidence),
        sample_count: count,
    })
}

pub(super) fn scale(training: &[f64], period: usize) -> f64 {
    let lag = if period > 0 && training.len() > period {
        period
    } else {
        1
    };
    let differences: Vec<f64> = training
        .iter()
        .skip(lag)
        .zip(training.iter())
        .map(|(current, previous)| (current - previous).abs())
        .collect();
    let value = differences.iter().sum::<f64>() / differences.len().max(1) as f64;
    if value.is_finite() && value > f64::EPSILON {
        value
    } else {
        1.0
    }
}

pub(super) fn quantile(values: &[f64], level: f64) -> f64 {
    let mut sorted: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.sort_by(f64::total_cmp);
    let index = ((sorted.len() - 1) as f64 * level.clamp(0.0, 1.0)).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn invalid(item: &Observation) -> bool {
    !item.actual.is_finite()
        || !item.predicted.is_finite()
        || !item.lower.is_finite()
        || !item.upper.is_finite()
        || item.lower > item.predicted
        || item.predicted > item.upper
        || !item.scale.is_finite()
        || item.scale <= 0.0
}

fn smape_item(item: &Observation) -> f64 {
    let denominator = item.actual.abs() + item.predicted.abs();
    if denominator <= f64::EPSILON {
        0.0
    } else {
        200.0 * (item.predicted - item.actual).abs() / denominator
    }
}

fn fold_maes(observations: &[Observation]) -> Vec<f64> {
    let max_fold = observations.iter().map(|item| item.fold).max().unwrap_or(0);
    (0..=max_fold)
        .filter_map(|fold| {
            let errors: Vec<f64> = observations
                .iter()
                .filter(|item| item.fold == fold)
                .map(|item| (item.predicted - item.actual).abs())
                .collect();
            (!errors.is_empty()).then(|| errors.iter().sum::<f64>() / errors.len() as f64)
        })
        .collect()
}

fn standard_deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    (values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64)
        .sqrt()
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;

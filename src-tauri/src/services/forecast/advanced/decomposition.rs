use std::collections::BTreeMap;

use super::stats;
use super::{AnalyticsStatus, DecompositionPoint, SeriesDecomposition};
use crate::services::forecast::evaluation::seasonal_period;
use crate::services::forecast::types::{ForecastResult, Prediction};

pub(super) fn analyze(result: &ForecastResult) -> Vec<SeriesDecomposition> {
    let mut grouped = BTreeMap::<String, Vec<&Prediction>>::new();
    for point in &result.input_data.history {
        grouped
            .entry(point.series_id.clone().unwrap_or_default())
            .or_default()
            .push(point);
    }
    grouped
        .into_iter()
        .map(|(id, points)| decompose((!id.is_empty()).then_some(id), &points, &result.frequency))
        .collect()
}

fn decompose(
    series_id: Option<String>,
    source: &[&Prediction],
    frequency: &str,
) -> SeriesDecomposition {
    if source.len() < 5 {
        return SeriesDecomposition {
            series_id,
            status: AnalyticsStatus::InsufficientData,
            method: "unavailable".into(),
            period: 1,
            seasonal_strength: None,
            points: Vec::new(),
        };
    }
    let values: Vec<_> = source.iter().map(|point| point.value).collect();
    let requested_period = seasonal_period(frequency);
    let seasonal = requested_period > 1 && values.len() >= requested_period.saturating_mul(2);
    let period = if seasonal { requested_period } else { 1 };
    let trend = moving_average(&values, if seasonal { requested_period } else { 5 });
    let effects = seasonal_effects(&values, &trend, period);
    let points: Vec<_> = source
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let seasonal_value = effects[index % period];
            DecompositionPoint {
                date: point.date.clone(),
                observed: point.value,
                trend: trend[index],
                seasonal: seasonal_value,
                residual: point.value - trend[index] - seasonal_value,
            }
        })
        .collect();
    SeriesDecomposition {
        series_id,
        status: AnalyticsStatus::Ready,
        method: if seasonal {
            "classical_additive".into()
        } else {
            "moving_average_trend".into()
        },
        period,
        seasonal_strength: seasonal.then(|| seasonal_strength(&points)),
        points,
    }
}

fn moving_average(values: &[f64], requested_window: usize) -> Vec<f64> {
    let window = requested_window.max(3).min(values.len());
    let left = (window - 1) / 2;
    let right = window / 2;
    (0..values.len())
        .map(|index| {
            let start = index.saturating_sub(left);
            let end = index.saturating_add(right + 1).min(values.len());
            stats::mean(&values[start..end])
        })
        .collect()
}

fn seasonal_effects(values: &[f64], trend: &[f64], period: usize) -> Vec<f64> {
    if period == 1 {
        return vec![0.0];
    }
    let mut phases = vec![Vec::new(); period];
    for (index, value) in values.iter().enumerate() {
        phases[index % period].push(value - trend[index]);
    }
    let mut effects: Vec<_> = phases.iter().map(|phase| stats::mean(phase)).collect();
    let center = stats::mean(&effects);
    for effect in &mut effects {
        *effect -= center;
    }
    effects
}

fn seasonal_strength(points: &[DecompositionPoint]) -> f64 {
    let residuals: Vec<_> = points.iter().map(|point| point.residual).collect();
    let remainder: Vec<_> = points
        .iter()
        .map(|point| point.residual + point.seasonal)
        .collect();
    let denominator = stats::variance(&remainder);
    if denominator <= f64::EPSILON {
        0.0
    } else {
        (1.0 - stats::variance(&residuals) / denominator).clamp(0.0, 1.0)
    }
}

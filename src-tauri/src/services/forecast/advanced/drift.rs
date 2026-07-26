use std::collections::BTreeMap;

use super::stats;
use super::{AnalyticsStatus, DriftReport};
use crate::services::forecast::types::{ForecastResult, Prediction};

pub(super) fn analyze(result: &ForecastResult) -> Vec<DriftReport> {
    let mut grouped = BTreeMap::<String, Vec<&Prediction>>::new();
    for point in &result.input_data.history {
        grouped
            .entry(point.series_id.clone().unwrap_or_default())
            .or_default()
            .push(point);
    }
    grouped
        .into_iter()
        .map(|(id, points)| inspect((!id.is_empty()).then_some(id), &points))
        .collect()
}

fn inspect(series_id: Option<String>, points: &[&Prediction]) -> DriftReport {
    if points.len() < 24 {
        return unavailable(series_id);
    }
    let window = (points.len() / 3).clamp(8, 256);
    let reference: Vec<_> = points[..window].iter().map(|point| point.value).collect();
    let recent: Vec<_> = points[points.len() - window..]
        .iter()
        .map(|point| point.value)
        .collect();
    let reference_variance = stats::variance(&reference);
    let recent_variance = stats::variance(&recent);
    let pooled_scale = ((reference_variance + recent_variance) / 2.0).sqrt();
    let mean_shift = if pooled_scale <= f64::EPSILON {
        0.0
    } else {
        (stats::mean(&recent) - stats::mean(&reference)) / pooled_scale
    };
    let (variance_ratio, variance_shift) = variance_change(reference_variance, recent_variance);
    let trend_shift = if pooled_scale <= f64::EPSILON {
        0.0
    } else {
        (stats::slope(&recent) - stats::slope(&reference)).abs() * window as f64 / pooled_scale
    };
    let distribution_shift = stats::ks_distance(&reference, &recent);
    let score = mean_shift
        .abs()
        .max(variance_shift)
        .max(trend_shift)
        .max(distribution_shift * 2.0);
    DriftReport {
        series_id,
        status: AnalyticsStatus::Ready,
        method: "windowed_distribution_shift".into(),
        reference_points: window,
        recent_points: window,
        score: Some(score),
        mean_shift: Some(mean_shift),
        variance_ratio,
        trend_shift: Some(trend_shift),
        distribution_shift: Some(distribution_shift),
        detected: score >= 1.0,
        severity: if score >= 2.0 {
            "high"
        } else if score >= 1.0 {
            "medium"
        } else {
            "none"
        }
        .into(),
    }
}

fn variance_change(reference: f64, recent: f64) -> (Option<f64>, f64) {
    if reference <= f64::EPSILON {
        return if recent <= f64::EPSILON {
            (Some(1.0), 0.0)
        } else {
            (None, 3.0)
        };
    }
    let ratio = recent / reference;
    if !ratio.is_finite() || ratio <= 0.0 {
        (None, 3.0)
    } else {
        (Some(ratio), ratio.ln().abs())
    }
}

fn unavailable(series_id: Option<String>) -> DriftReport {
    DriftReport {
        series_id,
        status: AnalyticsStatus::InsufficientData,
        method: "windowed_distribution_shift".into(),
        reference_points: 0,
        recent_points: 0,
        score: None,
        mean_shift: None,
        variance_ratio: None,
        trend_shift: None,
        distribution_shift: None,
        detected: false,
        severity: "unavailable".into(),
    }
}

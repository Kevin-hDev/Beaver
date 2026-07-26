use super::{AdvancedAnalytics, AnalyticsStatus, DriftReport};

pub(super) fn apply(analytics: &mut AdvancedAnalytics) {
    for decomposition in &mut analytics.decomposition {
        decomposition.seasonal_strength = finite(decomposition.seasonal_strength);
        let invalid = decomposition.period == 0
            || decomposition.points.iter().any(|point| {
                ![point.observed, point.trend, point.seasonal, point.residual]
                    .iter()
                    .all(|value| value.is_finite())
            });
        if invalid {
            decomposition.status = AnalyticsStatus::InsufficientData;
            decomposition.method = "unavailable".into();
            decomposition.seasonal_strength = None;
            decomposition.points.clear();
        }
    }
    analytics.anomalies.retain(|item| {
        [item.observed, item.expected, item.residual, item.score]
            .iter()
            .all(|value| value.is_finite())
    });
    sanitize_importance(analytics);
    for report in &mut analytics.drift {
        if !drift_is_finite(report) {
            clear_drift(report);
        }
    }
}

fn sanitize_importance(analytics: &mut AdvancedAnalytics) {
    let report = &mut analytics.variable_importance;
    report.baseline_mae = finite(report.baseline_mae);
    report.items.retain(|item| {
        [
            item.score,
            item.normalized_score,
            item.validation_mae,
            item.permuted_mae,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    if report.status == AnalyticsStatus::Ready && report.items.is_empty() {
        report.status = AnalyticsStatus::InsufficientData;
        report.reliability = "unavailable".into();
    }
}

fn drift_is_finite(report: &DriftReport) -> bool {
    [
        report.score,
        report.mean_shift,
        report.variance_ratio,
        report.trend_shift,
        report.distribution_shift,
    ]
    .iter()
    .flatten()
    .all(|value| value.is_finite())
}

fn clear_drift(report: &mut DriftReport) {
    report.status = AnalyticsStatus::InsufficientData;
    report.score = None;
    report.mean_shift = None;
    report.variance_ratio = None;
    report.trend_shift = None;
    report.distribution_shift = None;
    report.detected = false;
    report.severity = "unavailable".into();
}

fn finite(value: Option<f64>) -> Option<f64> {
    value.filter(|item| item.is_finite())
}

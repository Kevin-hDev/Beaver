use super::{variable_samples, variable_scoring, AnalyticsStatus};
use super::{VariableImportance, VariableImportanceReport};
use crate::services::forecast::types::ForecastResult;

pub(super) fn analyze(result: &ForecastResult) -> VariableImportanceReport {
    if result.covariates_used.is_empty() {
        return empty(AnalyticsStatus::NotApplicable);
    }
    let mut items: Vec<VariableImportance> = Vec::new();
    let mut validation_points = 0usize;
    let mut baseline_mae = None;
    for name in &result.covariates_used {
        let (training, validation) = variable_samples::for_column(result, name);
        if training.len() < 12 || validation.len() < 6 {
            continue;
        }
        let candidate = variable_scoring::importance(name, &training, &validation);
        validation_points = validation_points.max(validation.len());
        baseline_mae = Some(
            baseline_mae.unwrap_or(0.0_f64).max(
                validation
                    .iter()
                    .map(|sample| sample.delta.abs())
                    .sum::<f64>()
                    / validation.len() as f64,
            ),
        );
        items.push(candidate);
    }
    if items.is_empty() {
        return empty(AnalyticsStatus::InsufficientData);
    }
    let total: f64 = items.iter().map(|item| item.score).sum();
    for item in &mut items {
        item.normalized_score = if total > f64::EPSILON {
            item.score / total
        } else {
            0.0
        };
    }
    items.sort_by(|left, right| right.score.total_cmp(&left.score));
    VariableImportanceReport {
        status: AnalyticsStatus::Ready,
        method: "chronological_permutation_on_naive_residual".into(),
        reliability: reliability(validation_points).into(),
        scope: "all_series".into(),
        validation_points,
        baseline_mae,
        items,
    }
}

fn reliability(validation_points: usize) -> &'static str {
    if validation_points >= 30 {
        "high"
    } else if validation_points >= 12 {
        "moderate"
    } else {
        "low"
    }
}

fn empty(status: AnalyticsStatus) -> VariableImportanceReport {
    VariableImportanceReport {
        status,
        method: "chronological_permutation_on_naive_residual".into(),
        reliability: "unavailable".into(),
        scope: "all_series".into(),
        validation_points: 0,
        baseline_mae: None,
        items: Vec::new(),
    }
}

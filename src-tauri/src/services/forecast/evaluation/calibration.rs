use super::types::ModelBacktestResult;
use crate::services::forecast::types::ForecastResult;

pub(super) fn apply(
    analysis: &mut ForecastResult,
    results: &[ModelBacktestResult],
) -> Result<(), String> {
    let Some(calibration) = results
        .iter()
        .find(|result| {
            result.kind == super::types::BacktestKind::Model && result.model_id == analysis.model
        })
        .and_then(|result| result.calibration.as_ref())
        .filter(|calibration| calibration.sample_count >= 3)
    else {
        return Ok(());
    };
    let half_width = calibration.residual_half_width;
    if !half_width.is_finite() || half_width < 0.0 {
        return Ok(());
    }
    analysis.quantiles.q10 = analysis
        .predictions
        .iter()
        .enumerate()
        .map(|(index, point)| {
            analysis
                .quantiles
                .q10
                .get(index)
                .copied()
                .unwrap_or(point.value)
                .min(point.value - half_width)
        })
        .collect();
    analysis.quantiles.q50 = analysis
        .predictions
        .iter()
        .map(|point| point.value)
        .collect();
    analysis.quantiles.q90 = analysis
        .predictions
        .iter()
        .enumerate()
        .map(|(index, point)| {
            analysis
                .quantiles
                .q90
                .get(index)
                .copied()
                .unwrap_or(point.value)
                .max(point.value + half_width)
        })
        .collect();
    crate::services::forecast::target_domain::apply_saved_non_negative_floor(analysis);
    crate::services::forecast::result_validation::validate_stored_quantiles(analysis)
}

#[cfg(test)]
#[path = "calibration_tests.rs"]
mod tests;

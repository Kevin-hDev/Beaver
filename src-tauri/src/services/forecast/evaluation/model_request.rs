use super::folds::RollingFold;
use crate::services::forecast::types::{ForecastRequest, ForecastResult};

pub(super) fn build(
    analysis: &ForecastResult,
    fold: &RollingFold,
    model_id: &str,
    horizon: usize,
) -> Result<ForecastRequest, String> {
    let date_column = analysis
        .input_data
        .date_column
        .clone()
        .ok_or("legacy_columns_unavailable")?;
    if analysis.target_column.trim().is_empty() {
        return Err("legacy_columns_unavailable".into());
    }
    let data =
        serde_json::to_string(&fold.rows).map_err(|_| "invalid_backtest_data".to_string())?;
    Ok(ForecastRequest {
        data: Some(data),
        file_path: None,
        data_profile_id: None,
        target_column: analysis.target_column.clone(),
        date_column,
        series_column: analysis.input_data.series_column.clone(),
        covariate_columns: analysis.covariates_used.clone(),
        horizon: u32::try_from(horizon).map_err(|_| "invalid_backtest_horizon")?,
        frequency: analysis.frequency.clone(),
        model: Some(model_id.to_string()),
        confidence_level: analysis.confidence_level,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    })
}

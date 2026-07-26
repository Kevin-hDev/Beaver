pub const MAX_INLINE_DATA_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_SPREADSHEET_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_INPUT_ROWS: usize = 5_000;
pub const MAX_INPUT_COLUMNS: usize = 256;
pub const MAX_CELL_CHARS: usize = 32_768;
pub const MAX_COLUMN_CHARS: usize = 80;
pub const MAX_COVARIATES: usize = 64;
pub const MAX_MODEL_ID_CHARS: usize = 80;
pub const MAX_SERIES: usize = 256;
pub const MAX_HORIZON: u32 = 5_000;
pub const MAX_PREDICTIONS: usize = 100_000;
pub const MAX_RESPONSE_BYTES: usize = crate::services::secure_http::MAX_AUTHENTICATED_BODY_LIMIT;
pub const MAX_MODEL_DETAILS_METADATA_BYTES: usize = 1024 * 1024;
pub const MAX_MODEL_DETAILS_MARKDOWN_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_STORED_ANALYSIS_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ANALYSIS_INDEX_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_QUANTILE_LEVELS: usize = 16;
pub const MAX_DATA_PROFILES: usize = 20;
pub const MAX_STORED_ANALYSES: usize = 500;
pub const MAX_PROFILE_ISSUES: usize = 50;
pub const MAX_ISSUE_SAMPLES: usize = 10;
pub const MAX_TOOL_PREDICTIONS: usize = 200;
pub const MAX_TOOL_ANALYSES: usize = 100;
pub const MAX_TOOL_ANNOTATIONS: usize = 50;
pub const MAX_AUTO_CANDIDATES: usize = 5;
pub const MAX_AUTO_REASONS: usize = 8;
pub const MAX_SELECTION_TICKETS: usize = 32;
pub const MAX_SELECTION_REASON_CODES: usize = 8;
pub const MAX_AUTO_BACKTEST_SUMMARIES: usize = 20;
pub const MAX_TOOL_MODELS: usize = 64;
pub const MAX_PATH_CHARS: usize = 4_096;
pub const MAX_BACKTEST_MODELS: usize = 5;
pub const MAX_BACKTEST_WINDOWS: usize = 5;
pub const MAX_BACKTEST_HORIZON: usize = 256;
pub const MAX_BACKTEST_RESULTS: usize = 9;
pub const MAX_ENSEMBLE_MODELS: usize = 4;
pub const MAX_CONCURRENT_BACKTESTS: usize = 1;
pub const MAX_UNKNOWN_RESOURCE_RAM_MB: u32 = 2_048;

pub struct ToolSchemaLimits {
    pub inline_data_chars: usize,
    pub path_chars: usize,
    pub id_chars: usize,
    pub column_chars: usize,
    pub covariates: usize,
    pub horizon: u32,
    pub frequency_chars: usize,
}

impl Default for ToolSchemaLimits {
    fn default() -> Self {
        Self {
            inline_data_chars: MAX_INLINE_DATA_BYTES,
            path_chars: MAX_PATH_CHARS,
            id_chars: 64,
            column_chars: MAX_COLUMN_CHARS,
            covariates: MAX_COVARIATES,
            horizon: MAX_HORIZON,
            frequency_chars: 3,
        }
    }
}

pub fn validate_prediction_budget(series_count: usize, horizon: u32) -> Result<usize, String> {
    if series_count == 0 || series_count > MAX_SERIES {
        return Err("Nombre de séries invalide".into());
    }
    let total = series_count
        .checked_mul(horizon as usize)
        .ok_or("Volume de prédictions invalide")?;
    if total > MAX_PREDICTIONS {
        return Err("Volume de prédictions trop important".into());
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prediction_budget_accepts_boundary() {
        assert_eq!(validate_prediction_budget(20, 5_000), Ok(100_000));
    }

    #[test]
    fn prediction_budget_rejects_overflow() {
        assert!(validate_prediction_budget(21, 5_000).is_err());
        assert!(validate_prediction_budget(MAX_SERIES + 1, 1).is_err());
    }
}

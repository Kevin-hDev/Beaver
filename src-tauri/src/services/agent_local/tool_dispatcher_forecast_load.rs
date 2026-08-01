use super::types_tools::ToolResult;
use crate::services::forecast::data_profiles::DataProfileLoadError;
use crate::services::forecast::data_quality::DataProfile;
use crate::services::forecast::storage::ForecastLoadError;
use crate::services::forecast::types::ForecastResult;

pub(super) async fn load(id: &str) -> Result<ForecastResult, ToolResult> {
    crate::services::forecast::storage::load_classified(id)
        .await
        .map_err(map_error)
}

pub(super) async fn load_profile(id: &str) -> Result<DataProfile, ToolResult> {
    crate::services::forecast::data_profiles::load_profile_classified(id)
        .await
        .map_err(map_profile_error)
}

fn map_error(error: ForecastLoadError) -> ToolResult {
    match error {
        ForecastLoadError::InvalidId => ToolResult::validation(
            "forecast_analysis_id_invalid",
            error.message(),
        ),
        ForecastLoadError::NotFound => ToolResult::not_found(
            "forecast_analysis_not_found",
            error.message(),
        )
        .with_error_hint(
            "Utiliser forecast_read sans analysis_id pour relire les analyses disponibles.",
        ),
        ForecastLoadError::Unavailable => ToolResult::unavailable(
            "forecast_analysis_unavailable",
            error.message(),
            true,
        ),
        ForecastLoadError::Corrupt => ToolResult::internal(
            "forecast_analysis_corrupt",
            error.message(),
            false,
        )
        .with_error_hint(
            "Utiliser forecast_read sans analysis_id et choisir une autre analyse exploitable.",
        ),
    }
}

fn map_profile_error(error: DataProfileLoadError) -> ToolResult {
    match error {
        DataProfileLoadError::InvalidId => ToolResult::validation(
            "forecast_data_profile_id_invalid",
            error.message(),
        ),
        DataProfileLoadError::NotFound => ToolResult::not_found(
            "forecast_data_profile_not_found",
            error.message(),
        )
        .with_error_hint("Relancer forecast_data_audit pour créer un profil à jour."),
        DataProfileLoadError::Unavailable => ToolResult::unavailable(
            "forecast_data_profile_unavailable",
            error.message(),
            true,
        ),
        DataProfileLoadError::Corrupt => ToolResult::internal(
            "forecast_data_profile_corrupt",
            error.message(),
            false,
        )
        .with_error_hint("Relancer forecast_data_audit pour remplacer le profil invalide."),
    }
}

#[cfg(test)]
mod tests {
    use super::super::tool_result_contract::ToolErrorCategory;
    use super::*;

    #[test]
    fn load_failures_drive_distinct_recovery_paths() {
        let invalid = map_error(ForecastLoadError::InvalidId);
        let missing = map_error(ForecastLoadError::NotFound);
        let corrupt = map_error(ForecastLoadError::Corrupt);

        assert_eq!(invalid.error.unwrap().category, ToolErrorCategory::Validation);
        assert_eq!(missing.error.unwrap().category, ToolErrorCategory::NotFound);
        assert!(!corrupt.error.unwrap().retryable);
    }

    #[test]
    fn profile_failures_drive_distinct_recovery_paths() {
        let invalid = map_profile_error(DataProfileLoadError::InvalidId);
        let missing = map_profile_error(DataProfileLoadError::NotFound);
        let corrupt = map_profile_error(DataProfileLoadError::Corrupt);

        assert_eq!(invalid.error.unwrap().category, ToolErrorCategory::Validation);
        assert_eq!(missing.error.unwrap().category, ToolErrorCategory::NotFound);
        assert!(!corrupt.error.unwrap().retryable);
    }
}

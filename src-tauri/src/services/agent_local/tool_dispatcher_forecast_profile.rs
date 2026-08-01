use super::forecast_error::ForecastErrorKind;
use crate::services::forecast::data_profiles::{
    DataProfileHydrateError, DataProfileLoadError,
};

pub(super) fn classify(
    error: DataProfileHydrateError,
) -> (ForecastErrorKind, &'static str) {
    let kind = match error {
        DataProfileHydrateError::Ambiguous => {
            ForecastErrorKind::Validation("forecast_data_profile_ambiguous")
        }
        DataProfileHydrateError::Incompatible => {
            ForecastErrorKind::Validation("forecast_data_profile_incompatible")
        }
        DataProfileHydrateError::Load(DataProfileLoadError::InvalidId) => {
            ForecastErrorKind::Validation("forecast_data_profile_id_invalid")
        }
        DataProfileHydrateError::Load(DataProfileLoadError::NotFound) => {
            ForecastErrorKind::NotFound("forecast_data_profile_not_found")
        }
        DataProfileHydrateError::Load(DataProfileLoadError::Unavailable) => {
            ForecastErrorKind::Unavailable("forecast_data_profile_unavailable", true)
        }
        DataProfileHydrateError::Load(DataProfileLoadError::Corrupt) => {
            ForecastErrorKind::Internal("forecast_data_profile_corrupt", false)
        }
    };
    (kind, error.message())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydrate_failures_keep_their_exact_kind() {
        let (invalid, _) = classify(DataProfileHydrateError::Load(
            DataProfileLoadError::InvalidId,
        ));
        let (missing, _) = classify(DataProfileHydrateError::Load(
            DataProfileLoadError::NotFound,
        ));

        assert!(matches!(invalid, ForecastErrorKind::Validation(_)));
        assert!(matches!(missing, ForecastErrorKind::NotFound(_)));
    }
}

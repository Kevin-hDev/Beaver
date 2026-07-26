use crate::services::forecast::data_quality::DataProfile;
use crate::services::forecast::registry::{self, ForecastRuntimeSpec};
use crate::services::forecast::types::ForecastRequest;
use crate::services::forecast::validation;

pub fn resolve<'a>(
    request: &'a ForecastRequest,
    profile: &DataProfile,
) -> Result<(&'a str, &'static ForecastRuntimeSpec), String> {
    let model_id = validation::model_id(request)?;
    let runtime = registry::find_runtime(model_id)
        .filter(|runtime| registry::has_predict_adapter(runtime))
        .ok_or_else(|| "Moteur indisponible".to_string())?;
    if profile.future_rows > 0
        && !request.covariate_columns.is_empty()
        && !runtime.capabilities.future_covariates
    {
        return Err("Variables futures non supportées par ce moteur".to_string());
    }
    Ok((model_id, runtime))
}

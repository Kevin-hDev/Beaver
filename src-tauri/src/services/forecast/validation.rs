use super::limits;
use super::{catalog, registry, types::ForecastRequest};
const ALLOWED_FREQUENCIES: &[&str] = &[
    "10S", "15S", "30S", "S", "T", "min", "H", "D", "B", "W", "M", "Q", "Y",
];

pub fn model_id(request: &ForecastRequest) -> Result<&str, String> {
    let id = request
        .model
        .as_deref()
        .ok_or("Aucun modèle Forecast sélectionné")?;
    validate_model_id(id)?;
    Ok(id)
}

pub fn validate_model_id(id: &str) -> Result<(), String> {
    validate_model_id_format(id)?;
    if catalog::find_model(id).is_none() {
        return Err("Modèle inconnu".into());
    }
    Ok(())
}

pub fn validate_model_id_format(id: &str) -> Result<(), String> {
    if id.is_empty() || id.chars().count() > limits::MAX_MODEL_ID_CHARS {
        return Err("Modèle invalide".into());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err("Modèle invalide".into());
    }
    Ok(())
}

pub fn validate_runnable_model_id(id: &str) -> Result<(), String> {
    validate_model_id(id)?;
    if registry::find_runtime(id).is_none() {
        return Err("Moteur indisponible".into());
    }
    Ok(())
}

pub fn validate_request(request: &ForecastRequest) -> Result<(), String> {
    validate_data_request(request)?;
    super::validation_selection::validate_selection_metadata(request)?;
    let model_id = model_id(request)?;
    validate_runnable_model_id(model_id)?;
    let spec = catalog::find_model(model_id).ok_or("Modèle inconnu")?;
    let runtime = registry::find_runtime(model_id).ok_or("Moteur indisponible")?;
    validate_confidence_support(runtime, request.confidence_level)?;

    if request.series_column.is_some() && !runtime.capabilities.multi_series {
        return Err("Multi-séries non supporté par ce moteur".into());
    }
    if !request.covariate_columns.is_empty() && !runtime.capabilities.past_covariates {
        return Err("Variables de contexte non supportées par ce moteur".into());
    }
    let horizon_max = effective_horizon_max(model_id, spec.horizon_max)?;
    if request.horizon == 0 || request.horizon > horizon_max {
        return Err("Horizon invalide".into());
    }
    if !supports_frequency(spec, &request.frequency) {
        return Err("Fréquence non supportée par ce moteur".into());
    }
    Ok(())
}

pub fn supports_frequency(spec: &catalog::ForecastModelSpec, frequency: &str) -> bool {
    match spec.frequencies {
        "Toutes" | "10S à Y" => ALLOWED_FREQUENCIES.contains(&frequency),
        "T à Y" => matches!(
            frequency,
            "T" | "min" | "H" | "D" | "B" | "W" | "M" | "Q" | "Y"
        ),
        _ => false,
    }
}

pub fn interval_support(model_id: &str) -> &'static str {
    super::interval_capability::legacy_label(model_id)
}

pub fn supports_confidence(model_id: &str, confidence: f64) -> bool {
    super::interval_capability::supports(model_id, confidence)
}

fn validate_confidence_support(
    runtime: &registry::ForecastRuntimeSpec,
    confidence: f64,
) -> Result<(), String> {
    if supports_confidence(runtime.model_id, confidence) {
        return Ok(());
    }
    Err("Niveau de confiance non supporté par ce moteur".into())
}

pub fn validate_data_request(request: &ForecastRequest) -> Result<(), String> {
    validate_column(&request.target_column)?;
    validate_column(&request.date_column)?;
    if request.target_column == request.date_column {
        return Err("Colonnes invalides".into());
    }
    if let Some(series_column) = request.series_column.as_ref() {
        validate_column(series_column)?;
        if series_column == &request.target_column || series_column == &request.date_column {
            return Err("Colonne série invalide".into());
        }
    }
    if request.covariate_columns.len() > limits::MAX_COVARIATES {
        return Err("Trop de covariables".into());
    }
    let mut unique_covariates = std::collections::BTreeSet::new();
    for column in &request.covariate_columns {
        validate_column(column)?;
        if column == &request.target_column
            || column == &request.date_column
            || request.series_column.as_ref() == Some(column)
            || !unique_covariates.insert(column)
        {
            return Err("Covariables invalides".into());
        }
    }
    if request.horizon == 0 || request.horizon > limits::MAX_HORIZON {
        return Err("Horizon invalide".into());
    }
    if !super::interval_capability::valid_input_level(request.confidence_level) {
        return Err("Niveau de confiance invalide".into());
    }
    if !ALLOWED_FREQUENCIES.contains(&request.frequency.as_str()) {
        return Err("Fréquence invalide".into());
    }
    match (&request.data, &request.file_path) {
        (None, None) => Err("Données manquantes".into()),
        (Some(data), _) if data.len() > limits::MAX_INLINE_DATA_BYTES => {
            Err("Données trop volumineuses".into())
        }
        _ => Ok(()),
    }
}

pub fn effective_horizon_max(model_id: &str, catalog_max: u32) -> Result<u32, String> {
    let override_max = super::model_config::effective_values(model_id)?
        .get("horizon_max_override")
        .and_then(ValueExt::as_u32)
        .unwrap_or(0);
    if override_max == 0 {
        return Ok(catalog_max.min(limits::MAX_HORIZON));
    }
    Ok(override_max.min(catalog_max).min(limits::MAX_HORIZON))
}

trait ValueExt {
    fn as_u32(&self) -> Option<u32>;
}

impl ValueExt for serde_json::Value {
    fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|value| u32::try_from(value).ok())
    }
}

fn validate_column(column: &str) -> Result<(), String> {
    if column.is_empty() || column.chars().count() > limits::MAX_COLUMN_CHARS {
        return Err("Colonne invalide".into());
    }
    if column
        .chars()
        .any(|c| c.is_control() || matches!(c, '/' | '\\' | '\0'))
    {
        return Err("Colonne invalide".into());
    }
    Ok(())
}

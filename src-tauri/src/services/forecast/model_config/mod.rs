mod params;
mod schema;
mod storage;
mod validate;

use super::{catalog, model_manager, registry};
pub use schema::ParamKind;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize)]
pub struct ForecastConfigParam {
    pub id: &'static str,
    pub kind: ParamKind,
    pub label_key: String,
    pub description_key: String,
    pub default_value: Value,
    pub value: Option<Value>,
    pub effective_value: Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub options: Vec<String>,
}

#[derive(Serialize)]
pub struct ForecastModelConfig {
    pub model_id: String,
    pub family_id: String,
    pub params: Vec<ForecastConfigParam>,
    pub inherited: Vec<ForecastConfigParam>,
}

pub fn get(model_id: &str) -> Result<ForecastModelConfig, String> {
    super::validation::validate_runnable_model_id(model_id)?;
    let runtime = registry::find_runtime(model_id).ok_or("Moteur indisponible")?;
    let saved = validated_saved(model_id, runtime)?;
    Ok(build_response(model_id, runtime, saved))
}

pub fn set(model_id: &str, values: Map<String, Value>) -> Result<ForecastModelConfig, String> {
    require_configurable(model_id)?;
    let runtime = registry::find_runtime(model_id).ok_or("Moteur indisponible")?;
    let specs = schema::specs_for_runtime(runtime);
    let clean = validate::sanitize(&specs, values)?;
    storage::write_model(model_id, clean)?;
    get(model_id)
}

pub fn effective_values(model_id: &str) -> Result<Map<String, Value>, String> {
    let runtime = registry::find_runtime(model_id).ok_or("Moteur indisponible")?;
    let saved = validated_saved(model_id, runtime)?;
    let mut effective = Map::new();
    for spec in schema::specs_for_runtime(runtime) {
        effective.insert(
            spec.id.to_string(),
            saved.get(spec.id).cloned().unwrap_or(spec.default_value),
        );
    }
    Ok(effective)
}

fn validated_saved(
    model_id: &str,
    runtime: &registry::ForecastRuntimeSpec,
) -> Result<Map<String, Value>, String> {
    let specs = schema::specs_for_runtime(runtime);
    validate::sanitize(&specs, storage::read_model(model_id)?)
}

fn require_configurable(model_id: &str) -> Result<(), String> {
    super::validation::validate_runnable_model_id(model_id)?;
    let model = catalog::find_model(model_id).ok_or("Modèle inconnu")?;
    if model.is_cloud {
        let configured = crate::services::api_keys::list_configured()
            .iter()
            .any(|id| id == model.provider_id);
        return configured
            .then_some(())
            .ok_or_else(|| "Provider Forecast non configuré".to_string());
    }
    model_manager::is_installed(model_id)
        .then_some(())
        .ok_or_else(|| "Modèle non installé".to_string())
}

fn build_response(
    model_id: &str,
    runtime: &registry::ForecastRuntimeSpec,
    saved: Map<String, Value>,
) -> ForecastModelConfig {
    let params = schema::specs_for_runtime(runtime)
        .into_iter()
        .map(|spec| {
            let value = saved.get(spec.id).cloned();
            let effective_value = value.clone().unwrap_or_else(|| spec.default_value.clone());
            ForecastConfigParam {
                id: spec.id,
                kind: spec.kind,
                label_key: format!("forecast.modelConfig.params.{}.label", spec.id),
                description_key: format!("forecast.modelConfig.params.{}.description", spec.id),
                default_value: spec.default_value,
                value,
                effective_value,
                min: spec.min,
                max: spec.max,
                options: spec
                    .options
                    .iter()
                    .map(|option| (*option).to_string())
                    .collect(),
            }
        })
        .collect();
    ForecastModelConfig {
        model_id: model_id.to_string(),
        family_id: runtime.family_id.to_string(),
        params,
        inherited: inherited_params(),
    }
}

fn inherited_params() -> Vec<ForecastConfigParam> {
    let advanced = crate::services::config::read_config()
        .map(|config| config.advanced)
        .unwrap_or_default();
    vec![
        inherited("device", Value::String(advanced.hardware_accel)),
        inherited("model_unload_timeout", Value::String(advanced.keep_alive)),
    ]
}

fn inherited(id: &'static str, effective_value: Value) -> ForecastConfigParam {
    ForecastConfigParam {
        id,
        kind: ParamKind::Select,
        label_key: format!("forecast.modelConfig.params.{id}.label"),
        description_key: format!("forecast.modelConfig.params.{id}.description"),
        default_value: effective_value.clone(),
        value: None,
        effective_value,
        min: None,
        max: None,
        options: Vec::new(),
    }
}

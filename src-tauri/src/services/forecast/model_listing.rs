use super::{catalog, model_manager, registry};
use model_manager::ModelReadiness;
use serde_json::Value;

pub fn list_models() -> Value {
    let configured = crate::services::api_keys::list_configured();
    let providers: Vec<Value> = catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| {
            let mut value = serde_json::to_value(p).unwrap_or_default();
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    "configured".into(),
                    Value::Bool(configured.iter().any(|id| id == p.id)),
                );
            }
            value
        })
        .collect();
    let models: Vec<Value> = catalog::FORECAST_MODELS
        .iter()
        .map(|m| {
            let mut value = serde_json::to_value(m).unwrap_or_default();
            if let Some(object) = value.as_object_mut() {
                enrich_model_object(object, m, &configured);
            }
            value
        })
        .collect();
    serde_json::json!({
        "providers": providers,
        "models": models,
        "configured_provider_ids": configured
    })
}

fn enrich_model_object(
    object: &mut serde_json::Map<String, Value>,
    model: &catalog::ForecastModelSpec,
    configured: &[String],
) {
    let runtime = registry::find_runtime(model.id);
    let readiness = model_manager::readiness(model.id);
    let installed_model = readiness != ModelReadiness::NotInstalled;
    let provider_configured = configured.iter().any(|id| id == model.provider_id);
    let runtime_ready = runtime_ready_state(runtime, provider_configured, readiness);
    if let Some(runtime) = runtime {
        object.insert(
            "engine_kind".into(),
            serde_json::to_value(runtime.engine_kind).unwrap_or_default(),
        );
        object.insert(
            "capabilities".into(),
            serde_json::to_value(runtime.capabilities).unwrap_or_default(),
        );
        object.insert(
            "config_params".into(),
            serde_json::to_value(runtime.config_params).unwrap_or_default(),
        );
    } else {
        object.insert("capabilities".into(), empty_capabilities());
        object.insert("config_params".into(), Value::Array(Vec::new()));
    }
    object.insert(
        "family_id".into(),
        Value::String(model.family_id.to_string()),
    );
    object.insert(
        "interval_support".into(),
        Value::String(super::validation::interval_support(model.id).to_string()),
    );
    object.insert(
        "interval_capability".into(),
        serde_json::to_value(super::interval_capability::for_model(model.id)).unwrap_or_default(),
    );
    object.insert("installed".into(), Value::Bool(installed_model));
    object.insert("installable".into(), Value::Bool(installable(model)));
    object.insert(
        "runnable".into(),
        Value::Bool(runnable_state(
            model,
            runtime,
            installed_model,
            provider_configured,
            runtime_ready,
        )),
    );
    object.insert("runtime_ready".into(), Value::Bool(runtime_ready));
    object.insert(
        "readiness_state".into(),
        Value::String(readiness_state(model, runtime, provider_configured, readiness).into()),
    );
    object.insert(
        "size_on_disk".into(),
        Value::Number(model_manager::get_model_size(model.id).into()),
    );
    object.insert(
        "provider_configured".into(),
        Value::Bool(provider_configured),
    );
}

fn empty_capabilities() -> Value {
    serde_json::json!({
        "past_covariates": false,
        "future_covariates": false,
        "multi_series": false,
        "multivariate": false,
        "probabilistic": false,
        "backtesting_ready": false,
        "anomalies_ready": false,
        "fine_tuning_ready": false,
    })
}

fn installable(model: &catalog::ForecastModelSpec) -> bool {
    !model.is_cloud && (model.hf_repo.is_some() || model.github_repo.is_some())
}

fn runnable_state(
    model: &catalog::ForecastModelSpec,
    runtime: Option<&registry::ForecastRuntimeSpec>,
    installed: bool,
    provider_configured: bool,
    runtime_ready: bool,
) -> bool {
    match runtime {
        Some(spec) if registry::is_cloud(spec) => {
            registry::has_predict_adapter(spec) && provider_configured
        }
        Some(spec) => {
            registry::has_predict_adapter(spec) && !model.is_cloud && installed && runtime_ready
        }
        None => false,
    }
}

fn runtime_ready_state(
    runtime: Option<&registry::ForecastRuntimeSpec>,
    provider_configured: bool,
    readiness: ModelReadiness,
) -> bool {
    let Some(spec) = runtime else {
        return false;
    };
    if registry::is_cloud(spec) {
        return registry::has_predict_adapter(spec) && provider_configured;
    }
    registry::has_predict_adapter(spec) && readiness == ModelReadiness::Ready
}

fn readiness_state(
    model: &catalog::ForecastModelSpec,
    runtime: Option<&registry::ForecastRuntimeSpec>,
    provider_configured: bool,
    readiness: ModelReadiness,
) -> &'static str {
    if !model.is_cloud {
        return readiness.as_str();
    }
    match runtime {
        Some(spec) if registry::has_predict_adapter(spec) && provider_configured => "ready",
        Some(spec) if registry::has_predict_adapter(spec) => "provider_required",
        _ => "unsupported",
    }
}

#[cfg(test)]
#[path = "model_listing_tests.rs"]
mod tests;

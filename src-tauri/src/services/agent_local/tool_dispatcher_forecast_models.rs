use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::{
    data_profiles, hardware_profile, limits, model_listing, selection_policy, selection_tickets,
    storage, validation,
};
use serde_json::Value;

pub async fn handle(args: &Value, session_id: &str) -> ToolResult {
    let listing = model_listing::list_models();
    let Some(models) = listing["models"].as_array() else {
        return ToolResult::err("Catalogue Forecast indisponible");
    };
    let policy = match selection_policy::get() {
        Ok(policy) => policy,
        Err(error) => return ToolResult::err(error),
    };
    let forced_model = (policy.mode == selection_policy::ForecastSelectionMode::Manual)
        .then_some(policy.manual_model_id.as_deref())
        .flatten();
    let mut compact: Vec<Value> = models
        .iter()
        .filter_map(|model| compact_model(model, forced_model))
        .collect();
    compact.sort_by_key(model_sort_key);
    compact.truncate(limits::MAX_TOOL_MODELS);
    let forced_model_state = forced_model
        .and_then(|id| models.iter().find(|model| model["id"].as_str() == Some(id)))
        .and_then(|model| compact_model(model, forced_model));
    let installed_model_ids: Vec<&str> = compact
        .iter()
        .filter(|model| model["installed"].as_bool().unwrap_or(false))
        .filter_map(|model| model["id"].as_str())
        .collect();
    let runnable_model_ids: Vec<&str> = compact
        .iter()
        .filter(|model| model["runnable"].as_bool().unwrap_or(false))
        .filter_map(|model| model["id"].as_str())
        .collect();
    let payload = match policy.mode {
        selection_policy::ForecastSelectionMode::Manual => serde_json::json!({
            "selection_policy": {
                "mode": "manual",
                "forced_model": forced_model,
                "forced_model_state": forced_model_state
            },
            "summary": {
                "installed_model_ids": installed_model_ids,
                "runnable_model_ids": runnable_model_ids
            },
            "models": compact,
            "usage": "Compare the audited confidence_level with forced_model_state.interval_capability. Use the forced model only when it supports the exact level. Never round an explicit request; ask the user to change the level or selected model if they are incompatible."
        }),
        selection_policy::ForecastSelectionMode::Auto => {
            let Some(profile_id) = args["data_profile_id"].as_str() else {
                return ToolResult::err("Profil de données requis pour le mode Auto");
            };
            let profile = match data_profiles::load_profile(profile_id).await {
                Ok(profile) => profile,
                Err(error) => return ToolResult::err(error),
            };
            if profile.confidence_level.is_none() {
                return ToolResult::err(
                    "Profil Forecast obsolète : relancer forecast_data_audit",
                );
            }
            let hardware = hardware_profile::detect();
            let evidence = match storage::comparable_backtests(&profile).await {
                Ok(evidence) => evidence,
                Err(error) => return ToolResult::err(error),
            };
            let requested_model_id = match requested_model_id(args) {
                Ok(requested) => requested,
                Err(error) => return ToolResult::err(error),
            };
            let selection =
                crate::services::forecast::auto_selection::select_with_requested_model(
                models,
                &profile,
                policy.allow_cloud_in_auto,
                hardware,
                &evidence,
                requested_model_id,
            );
            let selection_id = match selection_tickets::issue(
                session_id,
                profile_id,
                &profile.fingerprint,
                &selection,
            ) {
                Ok(id) => id,
                Err(error) => return ToolResult::err(error),
            };
            let usage = if selection
                .requested_model
                .as_ref()
                .is_some_and(|requested| requested.status == "candidate")
            {
                "Use the explicitly requested candidate. It supports the profile's exact confidence_level; pass that level unchanged with selection_source='explicit_user_override' and selection_reason_codes=['user_requested'] to forecast. The model and its runtime are already ready."
            } else if selection.requested_model.is_some() {
                "The explicitly requested model was excluded. Explain requested_model.exclusion_reason and do not silently replace it. Use another candidate only after the user accepts."
            } else if selection.basis == "rolling_backtest" {
                "Choose only one returned candidate. Every candidate supports the profile's exact confidence_level; pass it unchanged. Follow the returned multi-metric ranking and require it to beat the best baseline, unless the user's explicit speed, local, cloud, or cost need justifies another safe candidate. Pass selection_id, selection_source, and short selection_reason_codes to forecast."
            } else {
                "Choose only one returned candidate. Every candidate supports the profile's exact confidence_level; pass it unchanged. This ranking uses capabilities and current resources, so do not call it the best model. Pass selection_id, selection_source, and short selection_reason_codes to forecast."
            };
            serde_json::json!({
                "selection_policy": {
                    "mode": "auto",
                    "cloud_allowed": policy.allow_cloud_in_auto
                },
                "task_profile": {
                    "history_points": profile.history_points,
                    "series_count": profile.series_count,
                    "horizon": profile.horizon,
                    "confidence_level": profile.confidence_level,
                    "frequency": profile.frequency,
                    "past_covariates": !profile.covariate_columns.is_empty(),
                    "future_covariates": profile.future_rows > 0 && !profile.covariate_columns.is_empty(),
                    "probabilistic_required": true
                },
                "hardware_profile": {
                    "scope": "forecast_only",
                    "gpu_memory_kind": hardware.gpu_memory_kind,
                    "gpu_memory_total_mb": hardware.vram_total_mb,
                    "gpu_memory_available_mb": hardware.vram_available_mb,
                    "ram_available_mb": hardware.ram_available_mb
                },
                "selection_id": selection_id,
                "candidates": selection.candidates,
                "requested_model": selection.requested_model,
                "selection_basis": selection.basis,
                "usage": usage
            })
        }
    };
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => ToolResult::ok(json),
        Err(_) => ToolResult::err("Catalogue Forecast indisponible"),
    }
}

fn requested_model_id(args: &Value) -> Result<Option<&str>, String> {
    match args.get("requested_model_id") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(id)) => {
            let id = id.trim();
            if id.is_empty() {
                return Ok(None);
            }
            validation::validate_model_id_format(id)?;
            Ok(Some(id))
        }
        Some(_) => Err("Modèle demandé invalide".to_string()),
    }
}

fn compact_model(model: &Value, forced_model: Option<&str>) -> Option<Value> {
    let id = model["id"].as_str()?;
    Some(serde_json::json!({
        "id": id,
        "selected": forced_model == Some(id),
        "name": model["display_name"].as_str().unwrap_or(""),
        "provider": model["provider_id"].as_str().unwrap_or(""),
        "family": model["family_id"].as_str().unwrap_or(""),
        "installed": model["installed"].as_bool().unwrap_or(false),
        "runnable": model["runnable"].as_bool().unwrap_or(false),
        "runtime_ready": model["runtime_ready"].as_bool().unwrap_or(false),
        "provider_configured": model["provider_configured"].as_bool().unwrap_or(false),
        "is_cloud": model["is_cloud"].as_bool().unwrap_or(false),
        "interval_support": crate::services::forecast::validation::interval_support(id),
        "interval_capability": crate::services::forecast::interval_capability::for_model(id),
        "capabilities": model["capabilities"].clone()
    }))
}

fn model_sort_key(model: &Value) -> (bool, bool, bool, String) {
    (
        !model["selected"].as_bool().unwrap_or(false),
        !model["runnable"].as_bool().unwrap_or(false),
        !model["installed"].as_bool().unwrap_or(false),
        model["id"].as_str().unwrap_or_default().to_string(),
    )
}

#[cfg(test)]
#[path = "tool_dispatcher_forecast_models_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "tool_dispatcher_forecast_models_request_tests.rs"]
mod request_tests;

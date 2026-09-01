use super::tool_dispatcher_forecast_models_support::{
    compact_model, model_sort_key, requested_model_id,
};
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::{
    hardware_profile, limits, model_listing, selection_policy, selection_tickets, storage,
};
use serde_json::Value;

pub async fn handle(args: &Value, session_id: &str) -> ToolResult {
    let listing = model_listing::list_models();
    let Some(models) = listing["models"].as_array() else {
        return ToolResult::internal(
            "forecast_model_catalog_invalid",
            "Catalogue Forecast indisponible",
            false,
        );
    };
    let policy = match selection_policy::get() {
        Ok(policy) => policy,
        Err(error) => {
            return ToolResult::internal("forecast_selection_policy_unavailable", error, true)
        }
    };
    let forced_model = (policy.mode == selection_policy::ForecastSelectionMode::Manual)
        .then_some(policy.manual_model_id.as_deref())
        .flatten();
    let mut compact: Vec<Value> = models
        .iter()
        .filter_map(|model| compact_model(model, forced_model))
        .collect();
    compact.sort_by_key(model_sort_key);
    let compact_truncated = compact.len() > limits::MAX_TOOL_MODELS;
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
                "runnable_model_ids": runnable_model_ids,
                "total_models": models.len(),
                "truncated": compact_truncated
            },
            "models": compact,
            "usage": "Compare the audited confidence_level with forced_model_state.interval_capability. Use the forced model only when it supports the exact level. Never round an explicit request; ask the user to change the level or selected model if they are incompatible."
        }),
        selection_policy::ForecastSelectionMode::Auto => {
            let Some(profile_id) = args["data_profile_id"].as_str() else {
                return ToolResult::validation(
                    "forecast_data_profile_required",
                    "Profil de données requis pour le mode Auto",
                );
            };
            let profile =
                match super::tool_dispatcher_forecast_load::load_profile(session_id, profile_id)
                    .await
                {
                    Ok(profile) => profile,
                    Err(error) => return error,
                };
            if profile.confidence_level.is_none() {
                return ToolResult::conflict(
                    "forecast_data_profile_stale",
                    "Profil Forecast obsolète : relancer forecast_data_audit",
                );
            }
            let hardware = hardware_profile::detect();
            let evidence = match storage::comparable_backtests(session_id, &profile).await {
                Ok(evidence) => evidence,
                Err(error) => {
                    return ToolResult::internal(
                        "forecast_backtest_evidence_unavailable",
                        error,
                        true,
                    )
                }
            };
            let requested_model_id = match requested_model_id(args) {
                Ok(requested) => requested,
                Err(error) => {
                    return ToolResult::validation("forecast_requested_model_invalid", error)
                }
            };
            let selection = crate::services::forecast::auto_selection::select_with_requested_model(
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
                Err(error) => return ToolResult::internal(
                    "forecast_selection_ticket_failed",
                    error,
                    false,
                )
                .with_error_hint(
                    "Relancer forecast_data_audit puis forecast_models pour reconstruire la sélection.",
                ),
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
    let result_truncated =
        policy.mode == selection_policy::ForecastSelectionMode::Manual && compact_truncated;
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => {
            let mut result = ToolResult::ok(json);
            result.mark_truncated(result_truncated);
            result
        }
        Err(_) => ToolResult::internal(
            "forecast_model_catalog_serialization_failed",
            "Catalogue Forecast indisponible",
            false,
        ),
    }
}

#[cfg(test)]
#[path = "tool_dispatcher_forecast_models_request_tests.rs"]
mod request_tests;
#[cfg(test)]
#[path = "tool_dispatcher_forecast_models_tests.rs"]
mod tests;

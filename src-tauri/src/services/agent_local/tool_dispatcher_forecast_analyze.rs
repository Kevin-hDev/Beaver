use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::types::MAX_ANNOTATIONS;
use crate::services::forecast::{scenarios, sidecar, storage};
use serde_json::Value;
use tauri::Manager;

pub async fn handle(args: &Value) -> ToolResult {
    let analysis_id = match args["analysis_id"].as_str() {
        Some(id) => id,
        None => return ToolResult::validation(
            "forecast_analysis_id_required",
            "Paramètre analysis_id requis",
        ),
    };
    let action = match args["action"].as_str() {
        Some(a) => a,
        None => return ToolResult::validation(
            "forecast_analysis_action_required",
            "Paramètre action requis",
        ),
    };

    let analysis = match super::tool_dispatcher_forecast_load::load(analysis_id).await {
        Ok(a) => a,
        Err(error) => return error,
    };

    match action {
        "annotate" => annotate(analysis, args).await,
        "scenario" => scenario_create(analysis_id, &args["params"]).await,
        "scenario_update" => scenario_update(analysis_id, &args["params"]).await,
        "scenario_delete" => scenario_delete(analysis_id, &args["params"]).await,
        "ensemble" => ensemble_create(analysis_id, &args["params"]).await,
        _ => ToolResult::validation("forecast_analysis_action_unsupported", format!(
            "Action '{action}' pas encore implémentée. Actions disponibles: annotate, scenario, scenario_update, scenario_delete, ensemble"
        )),
    }
}

async fn annotate(
    mut analysis: crate::services::forecast::types::ForecastResult,
    args: &Value,
) -> ToolResult {
    let text = args["params"]["text"].as_str().unwrap_or("");
    let date = args["params"]["date"].as_str().unwrap_or("");
    let Ok(text) = super::tool_dispatcher_forecast_annotation::clean_text(text) else {
        return ToolResult::validation(
            "forecast_annotation_text_invalid",
            "Paramètres d'annotation manquants. Utiliser params.text et params.date.",
        );
    };
    let Ok(date) = super::tool_dispatcher_forecast_annotation::clean_date(date) else {
        return ToolResult::validation(
            "forecast_annotation_date_invalid",
            "Paramètres d'annotation manquants. Utiliser params.text et params.date.",
        );
    };
    if analysis.annotations.len() >= MAX_ANNOTATIONS {
        return ToolResult::conflict(
            "forecast_annotation_limit_reached",
            "Limite d'annotations atteinte",
        );
    }
    analysis
        .annotations
        .push(crate::services::forecast::types::Annotation {
            id: uuid::Uuid::new_v4().to_string(),
            date,
            text,
            source: crate::services::forecast::types::AnnotationSource::Llm,
            note_title: None,
            note_type: None,
            note_content: None,
            note_created_at: None,
            note_updated_at: None,
        });
    match storage::save(&mut analysis).await {
        Ok(_) => ToolResult::ok("Annotation ajoutée"),
        Err(error) => ToolResult::internal(
            "forecast_annotation_save_failed",
            format!("Sauvegarde annotation: {error}"),
            false,
        )
        .with_error_hint("Relire l'analyse avant d'ajouter une nouvelle annotation."),
    }
}

async fn scenario_create(analysis_id: &str, params: &Value) -> ToolResult {
    let request = match super::tool_dispatcher_forecast_scenario_params::create_request(
        analysis_id,
        params,
    ) {
        Ok(request) => request,
        Err(error) => return ToolResult::validation("forecast_scenario_invalid", error),
    };
    let Some(chronos) = forecast_chronos() else {
        return forecast_service_unavailable();
    };
    let operation_sidecar = chronos.clone();
    save_scenario_result(
        chronos
            .run_cancellable(move || async move {
                scenarios::create(request, Some(&operation_sidecar)).await
            })
            .await,
    )
}

async fn scenario_update(analysis_id: &str, params: &Value) -> ToolResult {
    let request = match super::tool_dispatcher_forecast_scenario_params::update_request(
        analysis_id,
        params,
    ) {
        Ok(request) => request,
        Err(error) => return ToolResult::validation("forecast_scenario_invalid", error),
    };
    let Some(chronos) = forecast_chronos() else {
        return forecast_service_unavailable();
    };
    let operation_sidecar = chronos.clone();
    save_scenario_result(
        chronos
            .run_cancellable(move || async move {
                scenarios::update(request, Some(&operation_sidecar)).await
            })
            .await,
    )
}

async fn scenario_delete(analysis_id: &str, params: &Value) -> ToolResult {
    let scenario_id = params["scenario_id"].as_str().unwrap_or("");
    if scenario_id.is_empty() {
        return ToolResult::validation(
            "forecast_scenario_id_required",
            "Paramètres de scénario manquants. Utiliser params.scenario_id.",
        );
    }
    save_scenario_result(scenarios::delete(analysis_id, scenario_id).await)
}

async fn ensemble_create(analysis_id: &str, params: &Value) -> ToolResult {
    let model_ids = match params.get("model_ids") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(values)) if values.len() <= crate::services::forecast::limits::MAX_ENSEMBLE_MODELS => {
            let Some(ids) = values.iter().map(Value::as_str).collect::<Option<Vec<_>>>() else {
                return ToolResult::validation(
                    "forecast_ensemble_models_invalid",
                    "Liste de modèles d'ensemble invalide",
                );
            };
            if ids.iter().any(|id| {
                id.chars().count() > crate::services::forecast::limits::MAX_MODEL_ID_CHARS
            }) {
                return ToolResult::validation(
                    "forecast_ensemble_models_invalid",
                    "Liste de modèles d'ensemble invalide",
                );
            }
            ids.into_iter().map(str::to_string).collect()
        }
        _ => return ToolResult::validation(
            "forecast_ensemble_models_invalid",
            "Liste de modèles d'ensemble invalide",
        ),
    };
    let Some(chronos) = forecast_chronos() else {
        return forecast_service_unavailable();
    };
    let analysis_id = analysis_id.to_string();
    let operation_sidecar = chronos.clone();
    save_scenario_result(
        chronos
            .run_cancellable(move || async move {
                crate::services::forecast::advanced::ensemble::create(
                    &analysis_id,
                    &model_ids,
                    Some(&operation_sidecar),
                )
                .await
            })
            .await,
    )
}

fn save_scenario_result(
    result: Result<crate::services::forecast::types::ForecastResult, String>,
) -> ToolResult {
    match result {
        Ok(updated) => {
            if let Some(app) = super::app_handle_global::get() {
                crate::services::forecast::events::emit_updated(app, &updated);
            }
            match super::tool_dispatcher_forecast_output::analysis_payload(&updated, 0, 100) {
                Ok(json) => ToolResult::ok(json),
                Err(error) => ToolResult::internal(
                    "forecast_scenario_result_serialization_failed",
                    error,
                    false,
                )
                .with_error_hint("Relire l'analyse : la modification a déjà été enregistrée."),
            }
        }
        Err(error) => ToolResult::external(
            "forecast_scenario_mutation_failed",
            error,
            false,
        )
        .with_error_hint("Relire l'analyse avant de répéter cette modification de scénario."),
    }
}

fn forecast_chronos() -> Option<sidecar::ChronosSidecar> {
    let app = super::app_handle_global::get()?;
    Some(app.state::<sidecar::ChronosSidecar>().inner().clone())
}

fn forecast_service_unavailable() -> ToolResult {
    ToolResult::unavailable(
        "forecast_service_unavailable",
        "Service Forecast indisponible",
        true,
    )
}

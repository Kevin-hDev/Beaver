use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::types::MAX_ANNOTATIONS;
use crate::services::forecast::{scenarios, sidecar, storage};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use serde_json::Value;
use tauri::Manager;

const MAX_ANNOTATION_TEXT_CHARS: usize = 500;
const MAX_ANNOTATION_DATE_CHARS: usize = 80;

pub async fn handle(args: &Value) -> ToolResult {
    let analysis_id = match args["analysis_id"].as_str() {
        Some(id) => id,
        None => return ToolResult::err("Paramètre analysis_id requis"),
    };
    let action = match args["action"].as_str() {
        Some(a) => a,
        None => return ToolResult::err("Paramètre action requis"),
    };

    let analysis = match storage::load(analysis_id).await {
        Ok(a) => a,
        Err(e) => return ToolResult::err(format!("Analyse introuvable: {e}")),
    };

    match action {
        "annotate" => annotate(analysis, args).await,
        "scenario" => scenario_create(analysis_id, &args["params"]).await,
        "scenario_update" => scenario_update(analysis_id, &args["params"]).await,
        "scenario_delete" => scenario_delete(analysis_id, &args["params"]).await,
        "ensemble" => ensemble_create(analysis_id, &args["params"]).await,
        _ => ToolResult::err(format!(
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
    let Ok(text) = clean_annotation_text(text) else {
        return ToolResult::err(
            "Paramètres d'annotation manquants. Utiliser params.text et params.date.",
        );
    };
    let Ok(date) = clean_annotation_date(date) else {
        return ToolResult::err(
            "Paramètres d'annotation manquants. Utiliser params.text et params.date.",
        );
    };
    if analysis.annotations.len() >= MAX_ANNOTATIONS {
        return ToolResult::err("Limite d'annotations atteinte");
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
        Err(e) => ToolResult::err(format!("Sauvegarde annotation: {e}")),
    }
}

fn clean_annotation_text(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > MAX_ANNOTATION_TEXT_CHARS
        || trimmed.contains('\0')
    {
        return Err("Annotation invalide".into());
    }
    Ok(trimmed.to_string())
}

fn clean_annotation_date(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > MAX_ANNOTATION_DATE_CHARS
        || trimmed.contains('\0')
        || trimmed.contains(['\n', '\r'])
        || !is_supported_annotation_date(trimmed)
    {
        return Err("Date d'annotation invalide".into());
    }
    Ok(trimmed.to_string())
}

fn is_supported_annotation_date(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok()
        || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").is_ok()
}

async fn scenario_create(analysis_id: &str, params: &Value) -> ToolResult {
    let request = match super::tool_dispatcher_forecast_scenario_params::create_request(
        analysis_id,
        params,
    ) {
        Ok(request) => request,
        Err(e) => return ToolResult::err(e),
    };
    let chronos = forecast_chronos();
    save_scenario_result(scenarios::create(request, chronos.as_deref()).await)
}

async fn scenario_update(analysis_id: &str, params: &Value) -> ToolResult {
    let request = match super::tool_dispatcher_forecast_scenario_params::update_request(
        analysis_id,
        params,
    ) {
        Ok(request) => request,
        Err(e) => return ToolResult::err(e),
    };
    let chronos = forecast_chronos();
    save_scenario_result(scenarios::update(request, chronos.as_deref()).await)
}

async fn scenario_delete(analysis_id: &str, params: &Value) -> ToolResult {
    let scenario_id = params["scenario_id"].as_str().unwrap_or("");
    if scenario_id.is_empty() {
        return ToolResult::err("Paramètres de scénario manquants. Utiliser params.scenario_id.");
    }
    save_scenario_result(scenarios::delete(analysis_id, scenario_id).await)
}

async fn ensemble_create(analysis_id: &str, params: &Value) -> ToolResult {
    let model_ids = match params.get("model_ids") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(values)) if values.len() <= crate::services::forecast::limits::MAX_ENSEMBLE_MODELS => {
            let Some(ids) = values.iter().map(Value::as_str).collect::<Option<Vec<_>>>() else {
                return ToolResult::err("Liste de modèles d'ensemble invalide");
            };
            if ids.iter().any(|id| {
                id.chars().count() > crate::services::forecast::limits::MAX_MODEL_ID_CHARS
            }) {
                return ToolResult::err("Liste de modèles d'ensemble invalide");
            }
            ids.into_iter().map(str::to_string).collect()
        }
        _ => return ToolResult::err("Liste de modèles d'ensemble invalide"),
    };
    let chronos = forecast_chronos();
    save_scenario_result(
        crate::services::forecast::advanced::ensemble::create(
            analysis_id,
            &model_ids,
            chronos.as_deref(),
        )
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
                Err(e) => ToolResult::err(e),
            }
        }
        Err(e) => ToolResult::err(e),
    }
}

fn forecast_chronos() -> Option<tauri::State<'static, sidecar::ChronosSidecar>> {
    let app = super::app_handle_global::get()?;
    Some(app.state::<sidecar::ChronosSidecar>())
}

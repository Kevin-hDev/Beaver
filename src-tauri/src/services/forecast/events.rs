use super::types::ForecastResult;
use tauri::{AppHandle, Emitter};

pub fn emit_created(app: &AppHandle, analysis: &ForecastResult) {
    emit(
        app,
        "forecast-analysis-created",
        &analysis.id,
        analysis.session_id.as_deref(),
        Some(analysis.revision),
    );
}

pub fn emit_updated(app: &AppHandle, analysis: &ForecastResult) {
    emit_updated_id(
        app,
        &analysis.id,
        analysis.session_id.as_deref(),
        Some(analysis.revision),
    );
}

pub fn emit_updated_id(
    app: &AppHandle,
    analysis_id: &str,
    session_id: Option<&str>,
    revision: Option<u32>,
) {
    emit(
        app,
        "forecast-analysis-updated",
        analysis_id,
        session_id,
        revision,
    );
}

pub fn emit_deleted(app: &AppHandle, analysis_id: &str) {
    let _ = app.emit(
        "forecast-analysis-deleted",
        serde_json::json!({ "analysis_id": analysis_id }),
    );
}

fn emit(
    app: &AppHandle,
    event: &str,
    analysis_id: &str,
    session_id: Option<&str>,
    revision: Option<u32>,
) {
    let _ = app.emit(
        event,
        serde_json::json!({
            "analysis_id": analysis_id,
            "session_id": session_id,
            "revision": revision,
        }),
    );
}

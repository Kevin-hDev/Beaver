use super::types::ForecastResult;
use crate::services::workspace_scope::WorkspaceScope;
use tauri::{AppHandle, Emitter};

pub fn emit_created(app: &AppHandle, analysis: &ForecastResult) {
    emit(
        app,
        "forecast-analysis-created",
        &analysis.id,
        analysis.session_id.as_deref(),
        &analysis.workspace,
        Some(analysis.revision),
    );
}

pub fn emit_updated(app: &AppHandle, analysis: &ForecastResult) {
    emit_updated_id(
        app,
        &analysis.id,
        analysis.session_id.as_deref(),
        &analysis.workspace,
        Some(analysis.revision),
    );
}

pub fn emit_updated_id(
    app: &AppHandle,
    analysis_id: &str,
    session_id: Option<&str>,
    workspace: &WorkspaceScope,
    revision: Option<u32>,
) {
    emit(
        app,
        "forecast-analysis-updated",
        analysis_id,
        session_id,
        workspace,
        revision,
    );
}

pub fn emit_deleted(app: &AppHandle, analysis_id: &str, workspace: &WorkspaceScope) {
    let _ = app.emit(
        "forecast-analysis-deleted",
        serde_json::json!({ "analysis_id": analysis_id, "workspace": workspace }),
    );
}

fn emit(
    app: &AppHandle,
    event: &str,
    analysis_id: &str,
    session_id: Option<&str>,
    workspace: &WorkspaceScope,
    revision: Option<u32>,
) {
    let _ = app.emit(
        event,
        serde_json::json!({
            "analysis_id": analysis_id,
            "session_id": session_id,
            "workspace": workspace,
            "revision": revision,
        }),
    );
}

use crate::models::agent_turn_contract::{ChatStreamAdmission, ChatStreamRequestInput};
use serde::{Deserialize, Serialize};

const MAX_FIXTURE_OPERATIONS: usize = 8;

#[path = "reasoning_fixture_scenarios.rs"]
mod fixture_scenarios;

#[derive(Debug, Deserialize)]
pub struct FixtureOperation {
    tool_id: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SanitizedFixtureReport {
    schema_version: u8,
    fixture_id: String,
    route: String,
    model: String,
    region: String,
    reasoning_mode: String,
    generated_at: String,
    scenarios: Vec<fixture_scenarios::FixtureScenario>,
}

#[tauri::command]
pub async fn export_reasoning_fixture_report(
    session_id: String,
    region: String,
) -> Result<SanitizedFixtureReport, String> {
    export_reasoning_fixture_report_with_variant(session_id, region, None).await
}

pub(crate) async fn export_reasoning_fixture_report_with_variant(
    session_id: String,
    region: String,
    variant: Option<&str>,
) -> Result<SanitizedFixtureReport, String> {
    crate::services::agent_local::session_id::validate_session_id(&session_id)
        .map_err(|_| unavailable())?;
    let session = crate::services::agent_local::session_store::get(&session_id)
        .await
        .map_err(|_| unavailable())?;
    fixture_scenarios::validate_session(&session)?;
    let generated_at = chrono::Utc::now();
    if variant.is_some_and(|variant| session.reasoning_mode.as_deref() != Some(variant)) {
        return Err(unavailable());
    }
    let fixture_id = match variant {
        Some(variant) => crate::services::reasoning_fixture_store::derive_fixture_id_with_variant(
            &session.provider,
            &session.model,
            variant,
            &region,
            generated_at.date_naive(),
        ),
        None => crate::services::reasoning_fixture_store::derive_fixture_id(
            &session.provider,
            &session.model,
            &region,
            generated_at.date_naive(),
        ),
    }
    .map_err(|_| unavailable())?;
    let scenarios = fixture_scenarios::collect(&session);
    if !fixture_scenarios::proves_required_scenarios(&scenarios) {
        return Err(unavailable());
    }
    let report = SanitizedFixtureReport {
        schema_version: 1,
        fixture_id: fixture_id.clone(),
        route: session.provider,
        model: session.model,
        region,
        reasoning_mode: session
            .reasoning_mode
            .clone()
            .unwrap_or_else(|| "off".to_string()),
        generated_at: generated_at.to_rfc3339(),
        scenarios,
    };
    let bytes = serde_json::to_vec(&report).map_err(|_| unavailable())?;
    crate::services::reasoning_fixture_store::write_report(&session_id, &fixture_id, bytes)
        .await
        .map_err(|_| unavailable())?;
    Ok(report)
}

/// Exécute un lot debug borné. L'outil lui-même garde la allowlist et les
/// schémas fermés ; l'IPC ne choisit jamais une commande générale.
#[tauri::command]
pub async fn run_reasoning_fixture_tools(
    operations: Vec<FixtureOperation>,
) -> Result<Vec<serde_json::Value>, String> {
    run_fixture_operations(operations).await
}

/// Lance une conversation Agent Local réservée aux fixtures debug. Son contexte
/// d'outils est créé ici et ne peut donc jamais être activé par le chat normal.
#[tauri::command]
pub async fn run_reasoning_fixture_agent_local(
    app: tauri::AppHandle,
    streams: tauri::State<'_, crate::ActiveStreams>,
    run_id: String,
    request: ChatStreamRequestInput,
) -> Result<ChatStreamAdmission, String> {
    crate::services::reasoning_fixture_run_dedup::start_once(&run_id, || async {
        crate::commands::agent_chat_run::start_fixture(
            app,
            crate::commands::agent_chat_run::ChatStreamRequest::from_input(request),
            &streams,
        )
        .await
        .map_err(|_| unavailable())
    })
    .await
}

async fn run_fixture_operations(
    operations: Vec<FixtureOperation>,
) -> Result<Vec<serde_json::Value>, String> {
    if operations.is_empty() || operations.len() > MAX_FIXTURE_OPERATIONS {
        return Err(unavailable());
    }
    let mut run = crate::services::reasoning_fixture_run::FixtureRunContext::start()
        .await
        .map_err(|_| unavailable())?;
    let mut results = Vec::with_capacity(operations.len());
    for operation in operations {
        results.push(
            run.dispatch(&operation.tool_id, &operation.arguments)
                .await
                .map_err(|_| unavailable())?,
        );
    }
    Ok(results)
}

pub(super) fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}

#[cfg(test)]
#[path = "reasoning_fixture_tests.rs"]
mod tests;

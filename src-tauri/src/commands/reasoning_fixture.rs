use crate::models::agent_turn_contract::{ChatStreamAdmission, ChatStreamRequestInput};
use serde::{Deserialize, Serialize};

const MAX_SCENARIOS: usize = 64;
const MAX_FIXTURE_OPERATIONS: usize = 8;

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
    reasoning_mode: String,
    generated_at: String,
    scenarios: Vec<FixtureScenario>,
}

#[derive(Debug, Serialize)]
struct FixtureScenario {
    status: &'static str,
    request_count: usize,
    reasoning_event_count: usize,
    decisions: Vec<String>,
}

#[tauri::command]
pub async fn export_reasoning_fixture_report(
    session_id: String,
) -> Result<SanitizedFixtureReport, String> {
    crate::services::agent_local::session_id::validate_session_id(&session_id)
        .map_err(|_| unavailable())?;
    let session = crate::services::agent_local::session_store::get(&session_id)
        .await
        .map_err(|_| unavailable())?;
    validate_session(&session)?;
    let fixture_id = uuid::Uuid::new_v4().to_string();
    let scenarios = scenarios(&session);
    let report = SanitizedFixtureReport {
        schema_version: 1,
        fixture_id: fixture_id.clone(),
        route: session.provider,
        model: session.model,
        reasoning_mode: session
            .reasoning_mode
            .clone()
            .unwrap_or_else(|| "off".to_string()),
        generated_at: chrono::Utc::now().to_rfc3339(),
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
    request: ChatStreamRequestInput,
) -> Result<ChatStreamAdmission, String> {
    crate::commands::agent_chat_run::start_fixture(
        app,
        crate::commands::agent_chat_run::ChatStreamRequest::from_input(request),
        &streams,
    )
    .await
    .map_err(|_| unavailable())
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

fn validate_session(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> Result<(), String> {
    let users = session
        .messages
        .iter()
        .filter(|message| message.role == "user")
        .count();
    let has_tool = session
        .diagnostic_runs
        .iter()
        .any(|run| !run.recent_tools.is_empty() || run.last_tool.is_some());
    let has_model_result = session
        .diagnostic_runs
        .iter()
        .any(|run| run.events.iter().any(|event| event.phase == "model_result"));
    let has_payload = session.diagnostic_runs.iter().any(|run| {
        run.events
            .iter()
            .any(|event| event.phase == "provider_payload")
    });
    (users >= 2
        && session.diagnostic_runs.len() >= 2
        && has_tool
        && has_model_result
        && has_payload
        && session.diagnostic_runs.len() <= MAX_SCENARIOS)
        .then_some(())
        .ok_or_else(unavailable)
}

fn scenarios(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> Vec<FixtureScenario> {
    session
        .diagnostic_runs
        .iter()
        .take(MAX_SCENARIOS)
        .map(|run| {
            let decisions = run
                .events
                .iter()
                .filter(|event| event.phase == "reasoning")
                .map(|event| event.message.clone())
                .collect::<Vec<_>>();
            FixtureScenario {
                status: if run.status == "completed" {
                    "passe"
                } else {
                    "bloque"
                },
                request_count: 1,
                reasoning_event_count: decisions.len(),
                decisions,
            }
        })
        .collect()
}

fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_never_serializes_a_session_id() {
        let report = SanitizedFixtureReport {
            schema_version: 1,
            fixture_id: uuid::Uuid::new_v4().to_string(),
            route: "ollama".to_string(),
            model: "test".to_string(),
            reasoning_mode: "auto".to_string(),
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            scenarios: Vec::new(),
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("session_id"));
    }

    #[tokio::test]
    async fn bounded_fixture_operations_run_only_through_the_isolated_toolset() {
        let results = run_fixture_operations(vec![
            FixtureOperation {
                tool_id: "fixture.write_note".to_string(),
                arguments: serde_json::json!({ "value": "fixture" }),
            },
            FixtureOperation {
                tool_id: "fixture.read_note".to_string(),
                arguments: serde_json::json!({}),
            },
        ])
        .await
        .expect("fixture operations");

        assert_eq!(
            results,
            vec![
                serde_json::json!({ "written": true }),
                serde_json::json!({ "value": "fixture" })
            ]
        );
    }
}

//! Exécution directe des seuls outils de fixture, hors dispatcher Agent Local.

use super::stream_events::AgentEventEmitter;
use super::tool_execution_outcome::ToolExecutionOutcome;
use super::tool_executor_results::push_tool_result;
use super::types_ollama::ChatMessage;
use super::types_tools::ToolResult;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::reasoning_fixture_run::FixtureRunContext;
use tokio_util::sync::CancellationToken;

pub async fn execute(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    tool_call_ids: &[String],
    run: &mut FixtureRunContext,
    cancel: &CancellationToken,
) -> ToolExecutionOutcome {
    let mut outcome = ToolExecutionOutcome::default();
    for (index, (name, arguments)) in tool_calls.iter().enumerate() {
        let result = if cancel.is_cancelled() {
            ToolResult::cancelled("Annulé.")
        } else {
            match run.dispatch(name, arguments).await {
                Ok(value) => ToolResult::ok(render(value)),
                Err(_) => unavailable(),
            }
        };
        outcome.record(push_tool_result(
            on_event,
            messages,
            name,
            result,
            index,
            tool_call_ids.get(index).map(String::as_str),
            None,
            Vec::new(),
        ));
    }
    outcome
}

fn render(value: serde_json::Value) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
}

fn unavailable() -> ToolResult {
    ToolResult::error(
        "Outil de fixture indisponible.",
        "fixture_tool_unavailable",
        ToolErrorCategory::Permission,
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::stream_events::AgentEventEmitter;
    use serde_json::json;

    #[tokio::test]
    async fn dispatches_only_the_run_owned_fixture_tools() {
        let emitter = AgentEventEmitter::test("fixture-session".to_string());
        let root = {
            let mut fixture_run = FixtureRunContext::start().await.unwrap();
            let root = fixture_run.root_for_test();
            let mut messages = Vec::new();
            let cancel = CancellationToken::new();
            let calls = vec![
                (
                    "fixture.write_note".to_string(),
                    json!({ "value": "fixture" }),
                ),
                ("fixture.read_note".to_string(), json!({})),
                ("bash".to_string(), json!({ "command": "pwd" })),
            ];

            let mut outcome = execute(
                &emitter,
                &mut messages,
                &calls,
                &[],
                &mut fixture_run,
                &cancel,
            )
            .await;

            assert!(!outcome.apply_follow_ups(&mut messages).unwrap());
            assert_eq!(messages.len(), 3);
            assert!(messages[0].content.contains("written"));
            assert!(messages[1].content.contains("fixture"));
            assert!(messages[2].content.contains("fixture_tool_unavailable"));
            root
        };
        assert!(!root.exists());
    }
}

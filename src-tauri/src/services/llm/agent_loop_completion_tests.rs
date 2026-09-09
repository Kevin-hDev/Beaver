use super::{run, ApiRequestParams};
use crate::services::agent_local::{
    context_usage_buckets::ContextUsageSeed, session_store, stream_events::AgentEventEmitter,
    subagent_hidden_reports, subagent_orchestration::ParentSubagentOrchestrator,
    tool_artifact_preview::ToolResultPreviewBatch, types_ollama::ChatMessage,
};
use crate::services::llm::{fast_mode::FastModeRequest, stream_test_transport::StreamScenario};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn api_empty_answer_keeps_reports_until_a_real_success_without_retry() {
    let session = session_store::create_full(
        "Completion report admission",
        "mistral-small-latest",
        "mistral",
        false,
        None,
    )
    .await
    .unwrap();
    subagent_hidden_reports::append(
        &session.id,
        subagent_hidden_reports::build_report(
            uuid::Uuid::new_v4().to_string(),
            "Fixture".into(),
            "explorer".into(),
            "completed".into(),
            "Synthetic report".into(),
        ),
    )
    .await
    .unwrap();
    let mut observations = Vec::new();
    for content in ["", "OK"] {
        let scenario = StreamScenario::start_with_fragments(&session.id, vec![
            serde_json::json!({"choices":[{"delta":{"content":content},"finish_reason":"stop"}]}).to_string(),
        ]).await.unwrap();
        let emitter = AgentEventEmitter::test(session.id.clone());
        let mut messages = vec![ChatMessage::user("Summarize the report.".into())];
        let mut subagents = ParentSubagentOrchestrator::new(&session.id).await;
        let result = run(ApiRequestParams {
            on_event: &emitter,
            provider_id: "mistral",
            model: "mistral-small-latest",
            fast_mode: FastModeRequest::Standard,
            messages: &mut messages,
            tools: &[],
            think: false,
            reasoning_mode: None,
            session_id: &session.id,
            request_id: &uuid::Uuid::new_v4().to_string(),
            cancel: CancellationToken::new(),
            configured_context: 32_768,
            plan_mode_active: false,
            turn: 0,
            subagents: &mut subagents,
            context_usage_seed: ContextUsageSeed::default(),
            tool_result_previews: &ToolResultPreviewBatch::default(),
            continuation_target: None,
        })
        .await
        .unwrap();
        observations.push((
            result.result.completion_error,
            subagent_hidden_reports::peek_reports(&session.id)
                .await
                .len(),
            scenario.payloads().len(),
        ));
    }
    session_store::delete_one(&session.id).await.unwrap();
    assert_eq!(
        observations,
        vec![(Some("provider_empty_response"), 1, 1), (None, 0, 1)]
    );
}

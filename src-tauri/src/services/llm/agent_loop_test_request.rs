use super::agent_loop_request_types::ApiRequestOutput;
use crate::services::agent_local::generation_metrics::GenerationAggregate;
use crate::services::agent_local::subagent_orchestration::ParentSubagentOrchestrator;
use crate::services::agent_local::types_ollama::ChatMessage;
use tokio_util::sync::CancellationToken;

pub(super) async fn run(
    request_id: &str,
    request_cancel: &CancellationToken,
    subagents: &mut ParentSubagentOrchestrator,
    messages: &[ChatMessage],
    completion_cancel: &CancellationToken,
) -> Result<Option<ApiRequestOutput>, String> {
    let Some(result) =
        crate::services::agent_local::agent_loop_test_provider::next(request_id, request_cancel)
    else {
        return Ok(None);
    };
    let result = result?;
    subagents
        .complete_model_request(true, completion_cancel, messages)
        .await?;
    let mut generation = GenerationAggregate::default();
    generation.add_result(&result);
    Ok(Some(ApiRequestOutput {
        result,
        plan_active: false,
        interrupted: false,
        input_tokens: 0,
        generation,
    }))
}

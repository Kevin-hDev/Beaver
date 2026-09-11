use super::agent_loop_ollama_request::OllamaRequestOutput;
use super::generation_metrics::GenerationAggregate;
use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::types_ollama::ChatMessage;
use tokio_util::sync::CancellationToken;

pub(super) async fn run(
    request_id: &str,
    request_cancel: &CancellationToken,
    subagents: &mut ParentSubagentOrchestrator,
    messages: &[ChatMessage],
    completion_cancel: &CancellationToken,
) -> Result<Option<OllamaRequestOutput>, String> {
    let Some(result) = super::agent_loop_test_provider::next(request_id, request_cancel) else {
        return Ok(None);
    };
    let result = result?;
    subagents
        .complete_model_request(true, completion_cancel, messages)
        .await?;
    let mut generation = GenerationAggregate::default();
    generation.add_result(&result);
    Ok(Some(OllamaRequestOutput {
        result,
        eager_handle: tokio::spawn(async { std::collections::HashMap::new() }),
        plan_active: false,
        interrupted: false,
        input_tokens: 0,
        generation,
    }))
}

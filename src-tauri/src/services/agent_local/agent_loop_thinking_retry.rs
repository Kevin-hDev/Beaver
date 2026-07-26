use super::eager_dispatch;
use super::ollama_retry_indicator::{send_retry_indicator, REASON_THINKING_ONLY};
use super::ollama_thinking_retry::{build_thinking_disabled_retry, is_thinking_only_dead_end};
use super::stream_diagnostics_model as model_diag;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatRequest, StreamResult};
use super::types_stream::StreamOutcome;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::compress::realtime_budget::RealtimeBudget;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub type EagerHandle = JoinHandle<HashMap<usize, ToolResult>>;

pub struct ThinkingRetryParams<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub request: &'a ChatRequest,
    pub result: StreamResult,
    pub eager_handle: EagerHandle,
    pub turn: usize,
    pub working_dir: PathBuf,
    pub session_id: String,
    pub request_id: String,
    pub cancel: CancellationToken,
    pub plan_active: bool,
    pub realtime_budget: Option<RealtimeBudget>,
}

pub struct ThinkingRetryOutput {
    pub result: StreamResult,
    pub eager_handle: EagerHandle,
    pub interrupted: bool,
}

pub async fn retry_if_needed(
    params: ThinkingRetryParams<'_>,
) -> Result<ThinkingRetryOutput, String> {
    let Some(retry_req) = retry_request(params.request, &params.result) else {
        return Ok(ThinkingRetryOutput {
            result: params.result,
            eager_handle: params.eager_handle,
            interrupted: false,
        });
    };

    eprintln!(
        "[agent-loop] thinking-only détecté (turn {}), retry think=false",
        params.turn
    );
    super::stream_diagnostics::mark_phase(
        &params.session_id,
        &params.request_id,
        "model_stream",
        "Retry thinking-only (think=false).",
    )
    .await;
    if !params.plan_active {
        send_retry_indicator(params.on_event, REASON_THINKING_ONLY, 1, 1);
    }

    params.eager_handle.abort();
    let (retry_tx, retry_rx) = tokio::sync::mpsc::unbounded_channel();
    let eager_handle = tokio::spawn(eager_dispatch::collect_eager_results(
        retry_rx,
        params.working_dir,
        params.session_id.clone(),
        params.request_id.clone(),
    ));
    let retry_outcome = super::ollama_stream::stream_chat_with_tool_notify(
        params.on_event,
        &retry_req,
        params.cancel,
        retry_tx,
        params.plan_active,
        params.realtime_budget,
    )
    .await?;
    let (result, interrupted) = split_retry_outcome(retry_outcome);
    model_diag::record_model_result(&params.session_id, &params.request_id, params.turn, &result)
        .await;

    Ok(ThinkingRetryOutput {
        result,
        eager_handle,
        interrupted,
    })
}

fn split_retry_outcome(outcome: StreamOutcome) -> (StreamResult, bool) {
    let interrupted = outcome.is_interrupted();
    (outcome.into_result(), interrupted)
}

fn retry_request(request: &ChatRequest, result: &StreamResult) -> Option<ChatRequest> {
    if is_thinking_only_dead_end(result) {
        return build_thinking_disabled_retry(request);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_thinking_retry_keeps_interrupted_outcome() {
        let (result, interrupted) = split_retry_outcome(
            StreamOutcome::InterruptedForCompression(StreamResult {
                content: "partiel".into(),
                ..Default::default()
            }),
        );

        assert_eq!(result.content, "partiel");
        assert!(interrupted);
    }
}

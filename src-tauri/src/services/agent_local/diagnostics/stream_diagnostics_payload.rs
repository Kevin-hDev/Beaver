use super::stream_diagnostics_support as support;

#[cfg(test)]
#[path = "stream_diagnostics_payload_tests.rs"]
mod tests;

#[path = "stream_diagnostics_payload_stats.rs"]
mod payload_stats;
use payload_stats::payload_stats;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PayloadStats {
    items: usize,
    assistant_items: usize,
    reasoning_fields: usize,
    reasoning_chars: usize,
    assistant_content_chars: usize,
    assistant_content_nulls: usize,
    tool_calls: usize,
    tool_results: usize,
    available_tools: usize,
    instructions_chars: usize,
}

pub async fn record_provider_payload(
    session_id: Option<&str>,
    request_id: Option<&str>,
    provider_id: &str,
    kind: &str,
    payload: &serde_json::Value,
) {
    let (Some(session_id), Some(request_id)) = (session_id, request_id) else {
        return;
    };
    let stats = payload_stats(kind, payload);
    let _ = support::update_run(session_id, request_id, |_session, run| {
        let request = run
            .events
            .iter()
            .filter(|event| event.phase == "provider_payload")
            .count()
            + 1;
        let message = format!(
            "provider_payload provider={} kind={} request={} items={} assistant={} reasoning_fields={} reasoning_chars={} assistant_content_chars={} content_nulls={} tool_calls={} tool_results={} available_tools={} instructions_chars={}",
            provider_id,
            kind,
            request,
            stats.items,
            stats.assistant_items,
            stats.reasoning_fields,
            stats.reasoning_chars,
            stats.assistant_content_chars,
            stats.assistant_content_nulls,
            stats.tool_calls,
            stats.tool_results,
            stats.available_tools,
            stats.instructions_chars
        );
        run.phase = "provider_payload".to_string();
        run.safe_summary = Some(support::clip(&message));
        support::push_event(run, "provider_payload", &message, None, None);
    })
    .await;
}

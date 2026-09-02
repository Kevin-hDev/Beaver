use super::extension_tool_diagnostic::{
    structured, ExtensionDiagnosticOrigin, ExtensionDiagnosticReason, ExtensionToolDiagnostic,
};
use super::types_diagnostics::AgentExtensionDiagnostic;

pub async fn pending_extension_searches(
    session_id: &str,
    request_id: &str,
) -> Vec<AgentExtensionDiagnostic> {
    let Ok(session) = super::session_store::get(session_id).await else {
        return Vec::new();
    };
    let Some(run) = session
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
    else {
        return Vec::new();
    };
    let after = run
        .events
        .iter()
        .rposition(|event| {
            event.extension.as_ref().is_some_and(|diagnostic| {
                diagnostic.origin == ExtensionDiagnosticOrigin::Refreshed.as_str()
            })
        })
        .map_or(0, |index| index.saturating_add(1));
    run.events[after..]
        .iter()
        .filter_map(|event| event.extension.as_ref())
        .filter(|diagnostic| {
            diagnostic.origin == ExtensionDiagnosticOrigin::Search.as_str()
                && diagnostic
                    .correlation_id
                    .as_deref()
                    .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
        })
        .take(super::tool_executor_parallel_batch::MAX_PARALLEL)
        .cloned()
        .collect()
}

pub async fn record_extension_refreshes(
    session_id: &str,
    request_id: &str,
    pending: Vec<AgentExtensionDiagnostic>,
    added_names: &[String],
    provider_id: &str,
    alias_context: &[serde_json::Value],
) {
    let related_search_ids = pending
        .iter()
        .take(super::tool_executor_parallel_batch::MAX_PARALLEL)
        .filter_map(|diagnostic| diagnostic.correlation_id.clone())
        .collect::<Vec<_>>();
    if related_search_ids.is_empty() {
        return;
    }
    let mut extension = structured(&ExtensionToolDiagnostic {
        origin: ExtensionDiagnosticOrigin::Refreshed,
        reason: ExtensionDiagnosticReason::DiscoveryResult,
        correlation_id: None,
        plugin_ids: &[],
        tool_names: added_names,
        provider_id,
        alias_context,
        outcomes: &[],
        additional_tool_count: 0,
        added_tool_count: added_names.len(),
    });
    extension.related_search_ids = related_search_ids;
    let _ = super::stream_diagnostics_support::update_run(
        session_id,
        request_id,
        |_session, run| {
            super::stream_diagnostics_support::push_event(
                run,
                ExtensionDiagnosticOrigin::Refreshed.as_str(),
                "Extension tools.",
                None,
                None,
            );
            if let Some(event) = run.events.last_mut() {
                event.extension = Some(extension.clone());
            }
        },
    )
    .await;
}

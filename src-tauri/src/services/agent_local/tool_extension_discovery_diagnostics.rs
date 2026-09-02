use super::stream_diagnostics::{
    ExtensionDiagnosticOrigin, ExtensionDiagnosticOutcome, ExtensionDiagnosticReason,
    ExtensionToolDiagnostic,
};
use super::tool_extension_discovery_result::DiscoveryLine;

pub(super) async fn record(
    session_id: &str,
    request_id: &str,
    search_id: &str,
    lines: &[DiscoveryLine],
    provider_id: &str,
    tool_delta: usize,
) {
    let plugin_ids = lines
        .iter()
        .take(crate::services::extensions::MAX_SEARCH_RESULTS)
        .map(|line| line.plugin_id.clone())
        .collect::<Vec<_>>();
    let outcomes = lines
        .iter()
        .take(crate::services::extensions::MAX_SEARCH_RESULTS)
        .map(|line| ExtensionDiagnosticOutcome {
            plugin_id: line.plugin_id.clone(),
            reason: line.status.diagnostic_reason(),
        })
        .collect::<Vec<_>>();
    super::stream_diagnostics::record_extension_tools(
        session_id,
        request_id,
        ExtensionToolDiagnostic {
            origin: ExtensionDiagnosticOrigin::Search,
            reason: aggregate_reason(&outcomes),
            correlation_id: Some(search_id),
            plugin_ids: &plugin_ids,
            tool_names: &[],
            provider_id,
            alias_context: &[],
            outcomes: &outcomes,
            additional_tool_count: 0,
            added_tool_count: tool_delta,
        },
    )
    .await;
}

fn aggregate_reason(outcomes: &[ExtensionDiagnosticOutcome]) -> ExtensionDiagnosticReason {
    if outcomes
        .iter()
        .any(|outcome| outcome.reason == ExtensionDiagnosticReason::GlobalCapacity)
    {
        ExtensionDiagnosticReason::GlobalCapacity
    } else if outcomes
        .iter()
        .any(|outcome| outcome.reason == ExtensionDiagnosticReason::ProviderCapacity)
    {
        ExtensionDiagnosticReason::ProviderCapacity
    } else {
        ExtensionDiagnosticReason::DiscoveryResult
    }
}

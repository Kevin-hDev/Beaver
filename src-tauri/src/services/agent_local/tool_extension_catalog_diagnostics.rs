use crate::services::extensions::InspectionStatus;

pub(super) async fn record(
    session_id: &str,
    request_id: Option<&str>,
    outcomes: &[(String, InspectionStatus)],
) {
    let Some(request_id) = request_id else {
        return;
    };
    let plugin_ids = outcomes
        .iter()
        .take(crate::services::extensions::MAX_INSPECTED_EXTENSIONS)
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let outcomes = outcomes
        .iter()
        .take(crate::services::extensions::MAX_INSPECTED_EXTENSIONS)
        .map(|(plugin_id, status)| super::extension_tool_diagnostic::ExtensionDiagnosticOutcome {
            plugin_id: plugin_id.clone(),
            reason: match status {
                InspectionStatus::LimitedByProvider => {
                    super::extension_tool_diagnostic::ExtensionDiagnosticReason::ProviderCapacity
                }
                _ => super::extension_tool_diagnostic::ExtensionDiagnosticReason::InspectionResult,
            },
        })
        .collect::<Vec<_>>();
    let correlation_id = uuid::Uuid::new_v4().to_string();
    super::stream_diagnostics::record_extension_tools(
        session_id,
        request_id,
        super::extension_tool_diagnostic::ExtensionToolDiagnostic {
            origin: super::extension_tool_diagnostic::ExtensionDiagnosticOrigin::Inspection,
            reason: if outcomes.iter().any(|outcome| {
                outcome.reason
                    == super::extension_tool_diagnostic::ExtensionDiagnosticReason::ProviderCapacity
            }) {
                super::extension_tool_diagnostic::ExtensionDiagnosticReason::ProviderCapacity
            } else {
                super::extension_tool_diagnostic::ExtensionDiagnosticReason::InspectionResult
            },
            correlation_id: Some(&correlation_id),
            plugin_ids: &plugin_ids,
            tool_names: &[],
            provider_id: "",
            alias_context: &[],
            outcomes: &outcomes,
            additional_tool_count: 0,
            added_tool_count: 0,
        },
    )
    .await;
}

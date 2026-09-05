use super::extension_tool_set::ExtensionToolSet;
use super::stream_diagnostics::{
    ExtensionDiagnosticOrigin, ExtensionDiagnosticReason, ExtensionToolDiagnostic,
};

struct SelectionEvidence<'a> {
    masked: bool,
    active_plugin_ids: &'a [String],
    omitted_plugin_ids: &'a [String],
    protected_plugin_ids: &'a [String],
    essential_plugin_ids: &'a [String],
    discovered_plugin_ids: &'a [String],
}

struct SelectionGroup {
    reason: ExtensionDiagnosticReason,
    plugin_ids: Vec<String>,
}

fn selection_groups(
    plugins: &[super::extension_tool_selection::PluginDescriptor],
    evidence: &SelectionEvidence<'_>,
) -> Vec<SelectionGroup> {
    let mut groups: Vec<SelectionGroup> = Vec::new();
    for plugin in plugins {
        let reason = selection_reason(&plugin.id, evidence);
        if let Some(group) = groups.iter_mut().find(|group| group.reason == reason) {
            group.plugin_ids.push(plugin.id.clone());
        } else {
            groups.push(SelectionGroup {
                reason,
                plugin_ids: vec![plugin.id.clone()],
            });
        }
    }
    groups
}

fn selection_reason(
    plugin_id: &str,
    evidence: &SelectionEvidence<'_>,
) -> ExtensionDiagnosticReason {
    if evidence.omitted_plugin_ids.iter().any(|id| id == plugin_id) {
        ExtensionDiagnosticReason::ProviderCapacity
    } else if evidence
        .protected_plugin_ids
        .iter()
        .any(|id| id == plugin_id)
    {
        ExtensionDiagnosticReason::Protected
    } else if evidence
        .essential_plugin_ids
        .iter()
        .any(|id| id == plugin_id)
    {
        ExtensionDiagnosticReason::Essential
    } else if evidence
        .discovered_plugin_ids
        .iter()
        .any(|id| id == plugin_id)
    {
        ExtensionDiagnosticReason::PreviouslyDiscovered
    } else if evidence.active_plugin_ids.iter().any(|id| id == plugin_id) {
        ExtensionDiagnosticReason::CatalogVisible
    } else if evidence.masked {
        ExtensionDiagnosticReason::Masked
    } else {
        ExtensionDiagnosticReason::ProviderCapacity
    }
}

pub async fn record_selection(
    tools: &ExtensionToolSet,
    session_id: &str,
    request_id: &str,
    phase: &str,
) {
    let context = tools.diagnostic_context();
    let catalog = crate::services::extensions::catalog_snapshot();
    let evidence = SelectionEvidence {
        masked: context.masked,
        active_plugin_ids: context.active_plugin_ids,
        omitted_plugin_ids: &tools.omitted_plugin_ids,
        protected_plugin_ids: &catalog.protected_plugin_ids,
        essential_plugin_ids: &catalog.essential_plugin_ids,
        discovered_plugin_ids: context.discovered_plugin_ids,
    };
    let event_origin = origin(phase);
    for group in selection_groups(context.plugins, &evidence) {
        let names = tool_names_for_plugins(context.definitions, &group.plugin_ids);
        super::stream_diagnostics::record_extension_tools(
            session_id,
            request_id,
            ExtensionToolDiagnostic {
                origin: event_origin,
                reason: group.reason,
                correlation_id: None,
                plugin_ids: &group.plugin_ids,
                tool_names: &names,
                provider_id: context.provider_id,
                alias_context: context.definitions,
                outcomes: &[],
                additional_tool_count: 0,
                added_tool_count: 0,
            },
        )
        .await;
    }
    record_omitted_core_tools(
        tools,
        session_id,
        request_id,
        event_origin,
        context.provider_id,
        context.definitions,
    )
    .await;
}

fn tool_names_for_plugins(definitions: &[serde_json::Value], plugin_ids: &[String]) -> Vec<String> {
    definitions
        .iter()
        .filter_map(super::extension_tool_set_apply::definition_name)
        .filter(|name| {
            crate::services::extensions::plugin_id_for_tool(name)
                .is_some_and(|plugin_id| plugin_ids.contains(&plugin_id))
        })
        .take(crate::services::extensions::MAX_EXTENSION_TOOLS)
        .map(str::to_string)
        .collect()
}

async fn record_omitted_core_tools(
    tools: &ExtensionToolSet,
    session_id: &str,
    request_id: &str,
    origin: ExtensionDiagnosticOrigin,
    provider_id: &str,
    definitions: &[serde_json::Value],
) {
    if tools.omitted_tool_names.is_empty() && tools.additional_omitted_tools == 0 {
        return;
    }
    let omitted = tools.omitted_tool_names.clone();
    super::stream_diagnostics::record_extension_tools(
        session_id,
        request_id,
        ExtensionToolDiagnostic {
            origin,
            reason: ExtensionDiagnosticReason::ProviderCapacity,
            correlation_id: None,
            plugin_ids: &[],
            tool_names: &omitted,
            provider_id,
            alias_context: definitions,
            outcomes: &[],
            additional_tool_count: tools.additional_omitted_tools,
            added_tool_count: 0,
        },
    )
    .await;
}

fn origin(phase: &str) -> ExtensionDiagnosticOrigin {
    if phase == "extension_tools_refreshed" {
        ExtensionDiagnosticOrigin::Refreshed
    } else {
        ExtensionDiagnosticOrigin::Selected
    }
}

pub async fn refresh_and_record(
    tools: &mut ExtensionToolSet,
    session_id: &str,
    request_id: &str,
) -> Result<(), String> {
    let pending =
        super::stream_diagnostics::pending_extension_inspections(session_id, request_id).await;
    let before = tools.active().to_vec();
    tools.refresh_from_session(session_id).await?;
    let added_names = added_definition_names(&before, tools.active());
    record_selection(tools, session_id, request_id, "extension_tools_refreshed").await;
    let context = tools.diagnostic_context();
    super::stream_diagnostics::record_extension_refreshes(
        session_id,
        request_id,
        pending,
        &added_names,
        context.provider_id,
        tools.active(),
    )
    .await;
    Ok(())
}

fn added_definition_names(
    previous: &[serde_json::Value],
    current: &[serde_json::Value],
) -> Vec<String> {
    let previous = previous
        .iter()
        .filter_map(super::extension_tool_set_apply::definition_name)
        .collect::<std::collections::HashSet<_>>();
    current
        .iter()
        .filter_map(super::extension_tool_set_apply::definition_name)
        .filter(|name| !previous.contains(name))
        .take(crate::services::extensions::MAX_EXTENSION_TOOLS)
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
#[path = "extension_tool_set_diagnostics_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionDiagnosticOrigin {
    Selected,
    Refreshed,
    Search,
}

impl ExtensionDiagnosticOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Selected => "extension_tools_selected",
            Self::Refreshed => "extension_tools_refreshed",
            Self::Search => "extension_tool_search",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionDiagnosticReason {
    Protected,
    Essential,
    PreviouslyDiscovered,
    CatalogVisible,
    Masked,
    ProviderCapacity,
    GlobalCapacity,
    DiscoveryResult,
}

impl ExtensionDiagnosticReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Protected => "protected",
            Self::Essential => "essential",
            Self::PreviouslyDiscovered => "previously_discovered",
            Self::CatalogVisible => "catalog_visible",
            Self::Masked => "masked",
            Self::ProviderCapacity => "provider_capacity",
            Self::GlobalCapacity => "global_capacity",
            Self::DiscoveryResult => "discovery_result",
        }
    }
}

pub struct ExtensionToolDiagnostic<'a> {
    pub origin: ExtensionDiagnosticOrigin,
    pub reason: ExtensionDiagnosticReason,
    pub correlation_id: Option<&'a str>,
    pub plugin_ids: &'a [String],
    pub tool_names: &'a [String],
    pub provider_id: &'a str,
    pub alias_context: &'a [serde_json::Value],
    pub outcomes: &'a [ExtensionDiagnosticOutcome],
    pub additional_tool_count: usize,
    pub added_tool_count: usize,
}

pub struct ExtensionDiagnosticOutcome {
    pub plugin_id: String,
    pub reason: ExtensionDiagnosticReason,
}

pub fn structured(
    diagnostic: &ExtensionToolDiagnostic<'_>,
) -> super::types_diagnostics::AgentExtensionDiagnostic {
    let plugin_ids = valid_identifiers(diagnostic.plugin_ids);
    let tool_names = valid_identifiers(diagnostic.tool_names);
    let aliases = crate::services::llm::tool_schema::ToolNameMap::new(diagnostic.alias_context);
    let provider_aliases = tool_names
        .iter()
        .map(|name| aliases.wire_name_for_provider(diagnostic.provider_id, name))
        .collect::<Vec<_>>();
    let discovery_result = outcome_ids(
        diagnostic.outcomes,
        ExtensionDiagnosticReason::DiscoveryResult,
    );
    let provider_capacity = outcome_ids(
        diagnostic.outcomes,
        ExtensionDiagnosticReason::ProviderCapacity,
    );
    let global_capacity = outcome_ids(
        diagnostic.outcomes,
        ExtensionDiagnosticReason::GlobalCapacity,
    );
    super::types_diagnostics::AgentExtensionDiagnostic {
        origin: diagnostic.origin.as_str().to_string(),
        reason: diagnostic.reason.as_str().to_string(),
        correlation_id: diagnostic
            .correlation_id
            .filter(|value| uuid::Uuid::parse_str(value).is_ok())
            .map(str::to_string),
        related_search_ids: Vec::new(),
        plugin_count: plugin_ids.len(),
        plugin_ids: bounded_join(&plugin_ids),
        tool_count: tool_names
            .len()
            .saturating_add(diagnostic.additional_tool_count),
        canonical_tool_names: bounded_join(&tool_names),
        provider_aliases: bounded_join(
            &provider_aliases.iter().map(String::as_str).collect::<Vec<_>>(),
        ),
        tool_delta: diagnostic.added_tool_count,
        discovery_result_count: discovery_result.len(),
        discovery_result_plugin_ids: bounded_join(&discovery_result),
        provider_capacity_count: provider_capacity.len(),
        provider_capacity_plugin_ids: bounded_join(&provider_capacity),
        global_capacity_count: global_capacity.len(),
        global_capacity_plugin_ids: bounded_join(&global_capacity),
    }
}

fn outcome_ids(
    outcomes: &[ExtensionDiagnosticOutcome],
    reason: ExtensionDiagnosticReason,
) -> Vec<&str> {
    outcomes
        .iter()
        .take(crate::services::extensions::MAX_SEARCH_RESULTS)
        .filter(|outcome| outcome.reason == reason)
        .map(|outcome| outcome.plugin_id.as_str())
        .filter(|id| crate::services::extensions::validate_identifier(id).is_ok())
        .collect()
}

fn valid_identifiers(values: &[String]) -> Vec<&str> {
    values
        .iter()
        .filter(|value| crate::services::extensions::validate_identifier(value).is_ok())
        .map(String::as_str)
        .collect()
}

fn bounded_join(values: &[&str]) -> String {
    values
        .iter()
        .take(super::provider_tool_limits::MAX_CAPACITY_DIAGNOSTIC_ITEMS)
        .copied()
        .collect::<Vec<_>>()
        .join(",")
        .chars()
        .take(200)
        .collect::<String>()
}

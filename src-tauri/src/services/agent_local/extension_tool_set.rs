use serde_json::Value;

use super::extension_session_state::DiscoveryEpoch;
use super::extension_tool_selection::{
    decide, SelectionPolicy,
};
use super::extension_tool_set_apply::{
    active_definitions, append_capacity_notice, definition_name, plugin_descriptors,
};

pub struct PrepareContext<'a> {
    pub session_id: &'a str,
    pub provider: &'a str,
    pub model: &'a str,
    pub context_window: u64,
    pub preserve_dynamic_tools: bool,
}

pub struct ExtensionToolSet {
    all: Vec<Value>,
    active: Vec<Value>,
    managed: bool,
    masked: bool,
    provider_tool_limit: usize,
    plugin_tool_capacity: usize,
    omitted_plugin_ids: Vec<String>,
}

impl ExtensionToolSet {
    pub fn passthrough(tools: Vec<Value>) -> Self {
        Self {
            active: tools.clone(),
            all: tools,
            managed: false,
            masked: false,
            provider_tool_limit: usize::MAX,
            plugin_tool_capacity: 0,
            omitted_plugin_ids: Vec::new(),
        }
    }

    pub async fn prepare(tools: Vec<Value>, context: PrepareContext<'_>) -> Result<Self, String> {
        let plugin_definitions = tools
            .iter()
            .filter(|tool| {
                definition_name(tool).is_some_and(crate::services::extensions::is_dynamic_tool)
            })
            .cloned()
            .collect::<Vec<_>>();
        let computed_mask = super::extension_tool_mask::should_mask(
            &crate::services::extensions::extension_tool_definitions(),
            context.context_window,
        );
        let provider_limit =
            super::provider_tool_limits::for_request(context.provider, context.model);
        let core_count = tools.len().saturating_sub(plugin_definitions.len());
        let plugin_tool_capacity = provider_limit.saturating_sub(core_count.min(provider_limit));
        let catalog = crate::services::extensions::catalog_snapshot();
        let state = super::extension_session_state::configure(
            context.session_id,
            DiscoveryEpoch {
                provider: context.provider.to_string(),
                model: context.model.to_string(),
                context_window: context.context_window,
                catalog_version: catalog.version.clone(),
                masked: computed_mask,
            },
            computed_mask,
            plugin_tool_capacity,
        )
        .await?;
        let masked = state.epoch.as_ref().is_some_and(|epoch| epoch.masked)
            && !context.preserve_dynamic_tools;
        let mut result = Self {
            all: tools,
            active: Vec::new(),
            managed: true,
            masked,
            provider_tool_limit: provider_limit,
            plugin_tool_capacity,
            omitted_plugin_ids: Vec::new(),
        };
        result.apply(&state.discovered_plugin_ids);
        Ok(result)
    }

    pub fn active(&self) -> &[Value] {
        &self.active
    }

    pub fn selected_extension_names(&self) -> Vec<String> {
        if !self.managed {
            return Vec::new();
        }
        self.active
            .iter()
            .filter_map(definition_name)
            .filter(|name| crate::services::extensions::is_dynamic_tool(name))
            .map(str::to_string)
            .collect()
    }

    pub async fn refresh_from_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.managed {
            return Ok(());
        }
        let state = super::extension_session_state::read(session_id).await?;
        self.apply(&state.discovered_plugin_ids);
        Ok(())
    }

    fn apply(&mut self, discovered_plugin_ids: &[String]) {
        if !self.managed {
            return;
        }
        let catalog = crate::services::extensions::catalog_snapshot();
        let plugins = plugin_descriptors(&self.all);
        let decision = decide(
            &plugins,
            SelectionPolicy {
                masked: self.masked,
                tool_capacity: self.plugin_tool_capacity,
                ordered_plugin_ids: &catalog.ordered_plugin_ids,
                protected_plugin_ids: &catalog.protected_plugin_ids,
                essential_plugin_ids: &catalog.essential_plugin_ids,
                discovered_plugin_ids,
            },
        );
        self.active = active_definitions(&self.all, &decision, self.provider_tool_limit);
        self.omitted_plugin_ids = decision.omitted_plugin_ids;
        append_capacity_notice(&mut self.active, &self.omitted_plugin_ids);
    }
}

pub async fn record_selection(
    tools: &ExtensionToolSet,
    session_id: &str,
    request_id: &str,
    phase: &str,
) {
    let names = tools.selected_extension_names();
    if !names.is_empty() {
        super::stream_diagnostics::record_extension_tools(session_id, request_id, phase, &names)
            .await;
    }
    if !tools.omitted_plugin_ids.is_empty() {
        super::stream_diagnostics::record_extension_tools(
            session_id,
            request_id,
            "extension_plugins_omitted",
            &tools.omitted_plugin_ids,
        )
        .await;
    }
}

pub async fn refresh_and_record(
    tools: &mut ExtensionToolSet,
    session_id: &str,
    request_id: &str,
) -> Result<(), String> {
    tools.refresh_from_session(session_id).await?;
    record_selection(tools, session_id, request_id, "extension_tools_refreshed").await;
    Ok(())
}

#[cfg(test)]
#[path = "extension_tool_set_tests.rs"]
mod tests;

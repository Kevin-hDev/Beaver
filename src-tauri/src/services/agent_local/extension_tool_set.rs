use serde_json::Value;
use std::collections::HashSet;

use super::extension_session_state::DiscoveryEpoch;
use super::extension_tool_selection::decide_for_catalog;
use super::extension_tool_set_apply::{
    active_definitions, append_capacity_notice, base_tool_count, definition_name,
    plugin_descriptors,
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
    plugin_descriptors: Vec<super::extension_tool_selection::PluginDescriptor>,
    active_plugin_ids: Vec<String>,
    pub(super) omitted_plugin_ids: Vec<String>,
    pub(super) omitted_tool_names: Vec<String>,
    pub(super) additional_omitted_tools: usize,
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
            plugin_descriptors: Vec::new(),
            active_plugin_ids: Vec::new(),
            omitted_plugin_ids: Vec::new(),
            omitted_tool_names: Vec::new(),
            additional_omitted_tools: 0,
        }
    }

    pub async fn prepare(tools: Vec<Value>, context: PrepareContext<'_>) -> Result<Self, String> {
        let descriptors = plugin_descriptors(&tools);
        let computed_mask = super::extension_tool_mask::should_mask(
            &crate::services::extensions::extension_tool_definitions(),
            context.context_window,
        );
        let provider_limit =
            super::provider_tool_limits::for_request(context.provider, context.model);
        let plugin_tool_capacity =
            provider_limit.saturating_sub(base_tool_count(&tools).min(provider_limit));
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
            descriptors.clone(),
            context.preserve_dynamic_tools,
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
            plugin_descriptors: descriptors,
            active_plugin_ids: Vec::new(),
            omitted_plugin_ids: Vec::new(),
            omitted_tool_names: Vec::new(),
            additional_omitted_tools: 0,
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
        let active = self
            .active_plugin_ids
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        self.all
            .iter()
            .filter_map(definition_name)
            .filter(|name| {
                crate::services::extensions::plugin_id_for_tool(name)
                    .is_some_and(|plugin_id| active.contains(plugin_id.as_str()))
            })
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
        let decision = decide_for_catalog(
            &self.plugin_descriptors,
            &catalog,
            self.masked,
            self.plugin_tool_capacity,
            discovered_plugin_ids,
        );
        self.active_plugin_ids = decision.active_plugin_ids.clone();
        let active = active_definitions(&self.all, &decision, self.provider_tool_limit);
        self.active = active.tools;
        self.omitted_plugin_ids = decision.omitted_plugin_ids;
        self.omitted_tool_names = active.omitted_tool_names;
        self.additional_omitted_tools = active.additional_omitted_tools;
        append_capacity_notice(
            &mut self.active,
            &self.omitted_plugin_ids,
            &self.omitted_tool_names,
            self.additional_omitted_tools,
        );
    }
}

pub use super::extension_tool_set_diagnostics::{record_selection, refresh_and_record};

#[cfg(test)]
#[path = "extension_tool_set_tests.rs"]
mod tests;

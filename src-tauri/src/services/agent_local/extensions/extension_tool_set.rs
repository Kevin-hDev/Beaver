use serde_json::Value;

use super::extension_tool_selection::decide_for_catalog;
use super::extension_tool_set_apply::append_capacity_notice;

pub struct PrepareContext<'a> {
    pub session_id: &'a str,
    pub provider: &'a str,
    pub model: &'a str,
    pub context_window: u64,
    pub preserve_dynamic_tools: bool,
}

pub(super) struct DiagnosticContext<'a> {
    pub definitions: &'a [Value],
    pub plugins: &'a [super::extension_tool_selection::PluginDescriptor],
    pub active_plugin_ids: &'a [String],
    pub discovered_plugin_ids: &'a [String],
    pub masked: bool,
    pub provider_id: &'a str,
}

pub struct ExtensionToolSet {
    all: Vec<Value>,
    active: Vec<Value>,
    managed: bool,
    degradation: Option<&'static str>,
    _native_only: Option<native_only::NativeOnlyLease>,
    masked: bool,
    provider_tool_limit: usize,
    plugin_tool_capacity: usize,
    plugin_descriptors: Vec<super::extension_tool_selection::PluginDescriptor>,
    active_plugin_ids: Vec<String>,
    discovered_plugin_ids: Vec<String>,
    provider_id: String,
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
            degradation: None,
            _native_only: None,
            masked: false,
            provider_tool_limit: usize::MAX,
            plugin_tool_capacity: 0,
            plugin_descriptors: Vec::new(),
            active_plugin_ids: Vec::new(),
            discovered_plugin_ids: Vec::new(),
            provider_id: String::new(),
            omitted_plugin_ids: Vec::new(),
            omitted_tool_names: Vec::new(),
            additional_omitted_tools: 0,
        }
    }

    pub fn active(&self) -> &[Value] {
        &self.active
    }

    pub async fn refresh_from_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.managed {
            return Ok(());
        }
        let state = super::extension_session_state::read(session_id).await?;
        self.apply(&state.discovered_plugin_ids)
    }

    fn apply(&mut self, discovered_plugin_ids: &[String]) -> Result<(), String> {
        if !self.managed {
            return Ok(());
        }
        let catalog = crate::services::extensions::registry_catalog().map_err(str::to_string)?;
        self.apply_with_catalog(
            discovered_plugin_ids,
            &catalog,
            crate::services::extensions::plugin_id_for_tool,
        );
        Ok(())
    }

    fn apply_with_catalog(
        &mut self,
        discovered_plugin_ids: &[String],
        catalog: &crate::services::extensions::CatalogSnapshot,
        plugin_id_for_tool: impl Fn(&str) -> Option<String>,
    ) {
        let decision = decide_for_catalog(
            &self.plugin_descriptors,
            catalog,
            self.masked,
            self.plugin_tool_capacity,
            discovered_plugin_ids,
        );
        self.active_plugin_ids = decision.active_plugin_ids.clone();
        self.discovered_plugin_ids = discovered_plugin_ids.to_vec();
        let active = super::extension_tool_set_apply::active_definitions_with(
            &self.all,
            &decision,
            self.provider_tool_limit,
            plugin_id_for_tool,
        );
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

    pub(super) fn diagnostic_context(&self) -> DiagnosticContext<'_> {
        DiagnosticContext {
            definitions: &self.all,
            plugins: &self.plugin_descriptors,
            active_plugin_ids: &self.active_plugin_ids,
            discovered_plugin_ids: &self.discovered_plugin_ids,
            masked: self.masked,
            provider_id: &self.provider_id,
        }
    }
}

pub use super::extension_tool_set_diagnostics::{record_selection, refresh_and_record};

#[cfg(test)]
#[path = "extension_tool_set_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "extension_tool_set_resilience_tests.rs"]
mod resilience_tests;

#[path = "extension_tool_set_degraded.rs"]
mod degraded;
#[path = "extension_tool_set_prepare.rs"]
mod prepare;

#[path = "extension_tool_set_native_only.rs"]
mod native_only;
pub(crate) use native_only::native_only_for_session;

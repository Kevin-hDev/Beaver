use super::{ExtensionToolSet, PrepareContext};
use crate::services::agent_local::extension_session_state::DiscoveryEpoch;
use crate::services::agent_local::extension_tool_set_apply::{base_tool_count, plugin_descriptors};
use crate::services::extensions;
use serde_json::Value;

impl ExtensionToolSet {
    pub async fn prepare(tools: Vec<Value>, context: PrepareContext<'_>) -> Result<Self, String> {
        Self::prepare_with_registry(tools, context, extensions::registry_catalog()).await
    }

    pub(super) async fn prepare_with_registry(
        tools: Vec<Value>,
        context: PrepareContext<'_>,
        registry: Result<extensions::CatalogSnapshot, &'static str>,
    ) -> Result<Self, String> {
        crate::services::agent_local::session_store::validate_session_id(context.session_id)?;
        let descriptors = plugin_descriptors(&tools);
        let computed_mask = crate::services::agent_local::extension_tool_mask::should_mask(
            &crate::services::extensions::extension_tool_definitions(),
            context.context_window,
        );
        let route_policy =
            crate::services::llm::route_profile::tool_limit_policy(context.provider, context.model)
                .ok_or_else(|| "provider_configuration_invalid".to_string())?;
        let provider_limit =
            crate::services::agent_local::provider_tool_limits::for_policy(route_policy);
        let plugin_tool_capacity =
            provider_limit.saturating_sub(base_tool_count(&tools).min(provider_limit));
        let catalog = match registry {
            Ok(catalog) => catalog,
            Err(code) => return Self::degraded(tools, provider_limit, code, context.session_id),
        };
        let state = crate::services::agent_local::extension_session_state::configure(
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
        .await;
        let state = match state {
            Ok(state) => state,
            Err(_) => {
                return Self::degraded(
                    tools,
                    provider_limit,
                    extensions::error_codes::STATE_UNAVAILABLE,
                    context.session_id,
                )
            }
        };
        let masked = state.epoch.as_ref().is_some_and(|epoch| epoch.masked)
            && !context.preserve_dynamic_tools;
        let mut result = Self {
            all: tools,
            active: Vec::new(),
            managed: true,
            degradation: None,
            _native_only: None,
            masked,
            provider_tool_limit: provider_limit,
            plugin_tool_capacity,
            plugin_descriptors: descriptors,
            active_plugin_ids: Vec::new(),
            discovered_plugin_ids: Vec::new(),
            provider_id: context.provider.to_string(),
            omitted_plugin_ids: Vec::new(),
            omitted_tool_names: Vec::new(),
            additional_omitted_tools: 0,
        };
        result.apply(&state.discovered_plugin_ids)?;
        Ok(result)
    }
}

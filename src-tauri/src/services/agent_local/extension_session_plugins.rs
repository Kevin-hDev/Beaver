use std::collections::HashSet;

use super::extension_session_state::ExtensionSessionState;

pub fn refresh_active(state: &mut ExtensionSessionState, preserve_dynamic_tools: bool) {
    let catalog = crate::services::extensions::catalog_snapshot();
    refresh_active_with_catalog(state, preserve_dynamic_tools, &catalog);
}

pub(crate) fn refresh_active_with_catalog(
    state: &mut ExtensionSessionState,
    preserve_dynamic_tools: bool,
    catalog: &crate::services::extensions::CatalogSnapshot,
) {
    let masked = state.epoch.as_ref().is_some_and(|epoch| epoch.masked)
        && !preserve_dynamic_tools;
    state.active_plugin_ids = super::extension_tool_selection::decide_for_catalog(
        &state.plugin_descriptors,
        catalog,
        masked,
        state.plugin_tool_capacity,
        &state.discovered_plugin_ids,
    )
    .active_plugin_ids;
}

pub fn sanitize(state: &mut ExtensionSessionState) {
    let mut descriptor_ids =
        HashSet::with_capacity(crate::services::extensions::MAX_DISCOVERED_PLUGINS);
    state.plugin_descriptors.retain(|descriptor| {
        descriptor_ids.len() < crate::services::extensions::MAX_DISCOVERED_PLUGINS
            && crate::services::extensions::validate_identifier(&descriptor.id).is_ok()
            && descriptor.tool_count <= crate::services::extensions::MAX_EXTENSION_TOOLS
            && descriptor.definition_count <= crate::services::extensions::MAX_EXTENSION_TOOLS
            && descriptor.tool_count <= descriptor.definition_count
            && descriptor_ids.insert(descriptor.id.clone())
    });
    let known = state
        .plugin_descriptors
        .iter()
        .map(|descriptor| descriptor.id.as_str())
        .collect::<HashSet<_>>();
    let mut active_ids = HashSet::with_capacity(known.len());
    state.active_plugin_ids.retain(|id| {
        known.contains(id.as_str()) && active_ids.insert(id.clone())
    });
}

pub async fn is_tool_active(session_id: &str, tool_name: &str) -> Result<bool, String> {
    let Some(plugin_id) = crate::services::extensions::plugin_id_for_tool(tool_name) else {
        return Ok(false);
    };
    let state = super::extension_session_state::read(session_id).await?;
    Ok(state.active_plugin_ids.contains(&plugin_id))
}

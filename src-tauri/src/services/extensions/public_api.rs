use super::ui_types::UiActionPayload;

pub fn discovery_preferences() -> Result<super::DiscoveryPreferences, String> {
    super::discovery_preferences::get()
}

pub fn set_discovery_preferences(
    plugin_ids: Vec<String>,
) -> Result<super::DiscoveryPreferences, String> {
    super::discovery_preferences::set(plugin_ids)
}

pub fn ui_catalog() -> Result<super::UiCatalogSnapshot, String> {
    super::runtime::global()?.ui_catalog.snapshot()
}

pub async fn invoke_ui_action(
    extension_id: String,
    contribution_id: String,
    action_id: String,
    payload: UiActionPayload,
    locale: String,
) -> Result<serde_json::Value, String> {
    super::ui_dispatch::invoke(super::ui_types::UiActionRequest {
        extension_id,
        contribution_id,
        action_id,
        payload,
        locale,
    })
    .await
}

pub fn report_ui_mount_failure(extension_id: &str, contribution_id: &str) -> Result<(), String> {
    super::validation::identifier(extension_id)?;
    super::validation::identifier(contribution_id)?;
    super::runtime::global()?.record_ui_mount_failure(extension_id, contribution_id)
}

pub(crate) fn record_tool_invocation(tool_name: &str) -> Result<(), String> {
    super::discovery_usage::record_invocation(tool_name)
}

pub(crate) async fn revoke_extension(id: &str, deadline: std::time::Instant) -> Result<(), String> {
    let record = super::registry::find(id)?;
    let identity = super::host_identity::HostIdentity::from_record(&record)?;
    super::runtime::revoke_extension(&identity, deadline).await
}

pub(crate) fn close_command_error(operation: &str, error: String) -> String {
    super::operation_error::close(operation, error)
}

pub(crate) const MAX_DISCOVERED_PLUGINS: usize = super::types::MAX_EXTENSIONS;
pub(crate) const MAX_EXTENSION_TOOLS: usize = super::types::MAX_TOOLS;
pub(crate) const MAX_PERMISSION_SUMMARY_CHARS: usize = super::types::MAX_PERMISSION_SUMMARY_CHARS;

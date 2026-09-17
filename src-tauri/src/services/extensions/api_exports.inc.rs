pub use extension_recovery::ExtensionRecoveryState;
pub use types::{ExtensionEffect, ExtensionHostStatus, ExtensionKind};
pub use ui_types::{UiActionPayload, UiCatalogSnapshot};
pub use view::ExtensionView;

pub(crate) use discovery_catalog::CatalogSnapshot;
pub(crate) use discovery_contract::{CONTEXT_THRESHOLD_PERCENT, UNKNOWN_CONTEXT_TOKENS};
pub(crate) use discovery_inspection::inspect as inspect_discoverable;
pub(crate) use discovery_inspection::InspectionStatus;
pub(crate) use discovery_limits::DISCOVERY_STORE_MAX_BYTES;
pub(crate) use discovery_listing::list as list_discoverable;
pub(crate) use discovery_result_serialization::serialize_bounded_result;
pub(crate) use event_api::automation_event;
pub(crate) use event_api::subagent_status_changed as subagent_status_event;
pub(crate) const MAX_INSPECTED_EXTENSIONS: usize = discovery_contract::MAX_INSPECTED_EXTENSIONS;
pub(crate) const MAX_COMPACT_CATALOG_BYTES: usize = discovery_contract::MAX_COMPACT_CATALOG_BYTES;
pub(crate) const LIST_EXTENSIONS_TOOL_NAME: &str = discovery_contract::DISCOVERY_TOOL_NAMES[0];
pub(crate) const INSPECT_EXTENSIONS_TOOL_NAME: &str = discovery_contract::DISCOVERY_TOOL_NAMES[1];
pub use discovery_preferences::DiscoveryPreferences;
pub use public_api::{
    discovery_preferences, invoke_ui_action, report_ui_mount_failure, set_discovery_preferences,
    ui_catalog,
};
include!("registry_exports.inc.rs");
pub use runtime::status;
pub use runtime_dispatch::dispatch_tool;
pub(crate) use runtime_lifecycle::{new_stop_deadline, CHANGED_EVENT};
pub use runtime_lifecycle::{restart, stop_and_wait};
#[cfg(feature = "e2e")]
pub(crate) use startup::initialize;
pub use startup::initialize_on_startup;
pub(crate) use tool_bridge::definitions as extension_tool_definitions;
pub(crate) use tool_bridge::{core_fallback, without_core_fallback};
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};
pub(crate) use tool_interception::{
    before_effect as before_tool_effect, snapshot_for_model_request, InterceptionSnapshot,
};
pub(crate) use tool_result::unavailable as unavailable_tool_result;

pub(crate) use public_api::{
    automation_owner_is_current, close_command_error, record_tool_invocation, revoke_extension,
    MAX_DISCOVERED_PLUGINS, MAX_EXTENSION_TOOLS, MAX_PERMISSION_SUMMARY_CHARS,
};

pub(crate) use extension_internal_exports::*;
pub(crate) use installer::{
    install_git as install_git_source, install_npm as install_npm_source,
    uninstall as uninstall_extension, update as update_managed_extension,
};
pub(crate) use operation_error::{report as report_operation_error, Operation};
pub(crate) use operation_failure::OperationFailure;
pub(crate) use resource_identifier::parse as parse_qualified_contribution_id;
pub(crate) use resource_loader::{
    load_skill_for_session as load_extension_skill_for_session, LoadedResource, ResourceLoadError,
};

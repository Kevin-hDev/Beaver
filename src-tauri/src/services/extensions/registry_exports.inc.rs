pub use registry::{list, set_enabled, set_show_in_chat};
pub(crate) use registry_index::{
    catalog_snapshot, dynamic_tool_names, indexed_plugins, indexed_tool, plugin_id_for_tool,
};
pub use registry_index::{is_dynamic_tool, is_replacement};
pub use registry_recovery::{disable_hosted_extensions, restore_recovery_snapshot};
pub(crate) use registry_index::{registry_availability, registry_catalog};
#[cfg(test)]
pub(crate) use registry_startup::initialize_test_registry;

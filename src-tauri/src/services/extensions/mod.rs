mod builtin;
mod core_bridge;
mod host_channel;
mod host_paths;
mod host_process;
mod host_reader;
mod manifest;
mod message_validation;
mod protocol;
mod registry;
mod registry_index;
mod registry_sync;
mod runtime;
mod runtime_diagnostics;
mod runtime_dispatch;
mod runtime_restart;
mod runtime_sync;
mod runtime_version;
mod startup;
mod storage;
mod tool_bridge;
mod types;
mod validation;

pub use types::{ExtensionHostStatus, ExtensionKind, ExtensionRecord};

pub use registry::{
    add_local, disable_user_extensions, list, remove, set_enabled, set_show_in_chat,
};
pub use registry_index::{is_dynamic_tool, is_replacement};
pub use runtime::{restart, status, stop};
pub use runtime_dispatch::{dispatch_tool, emit_event};
pub use startup::initialize_on_startup;
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};

pub(crate) use manifest::load_local as install_local;
pub(crate) use validation::identifier as validate_identifier;

#[cfg(test)]
mod builtin_tests;
#[cfg(test)]
mod runtime_sync_tests;
#[cfg(test)]
mod tests;

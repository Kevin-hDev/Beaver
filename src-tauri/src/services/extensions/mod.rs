mod builtin;
mod core_bridge;
mod host_paths;
mod host_process;
mod manifest;
mod message_validation;
mod protocol;
mod registry;
mod runtime;
mod runtime_dispatch;
mod runtime_sync;
mod startup;
mod storage;
mod tool_bridge;
mod types;
mod validation;

pub use types::{ExtensionHostStatus, ExtensionKind, ExtensionRecord};

pub use registry::{
    add_local, disable_user_extensions, is_dynamic_tool, list, remove, set_enabled,
    set_show_in_chat,
};
pub use runtime::{restart, start_and_sync, status, stop};
pub use runtime_dispatch::{dispatch_tool, emit_event};
pub use startup::initialize_on_startup;
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};

pub(crate) use manifest::load_local as install_local;
pub(crate) use validation::identifier as validate_identifier;

#[cfg(test)]
mod tests;

mod builtin;
mod core_bridge;
mod error_codes;
mod git_source;
mod host_channel;
mod host_paths;
mod host_process;
mod host_reader;
mod installer;
mod installer_record;
mod managed_cleanup;
mod managed_store;
mod managed_tree;
mod manifest;
mod manifest_source;
mod message_validation;
mod npm_environment;
mod npm_runner;
mod npm_source;
mod operation_error;
mod operation_log;
mod origin_validation;
mod process_runner;
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
mod source_validation;
mod startup;
mod storage;
mod tool_bridge;
mod types;
mod validation;
mod view;

pub use types::{ExtensionHostStatus, ExtensionKind};
pub use view::ExtensionView;

pub use registry::{add_local, disable_hosted_extensions, list, set_enabled, set_show_in_chat};
pub use registry_index::{is_dynamic_tool, is_replacement};
pub use runtime::{restart, status, stop};
pub use runtime_dispatch::{dispatch_tool, emit_event};
pub use startup::initialize_on_startup;
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};

pub(crate) use installer::{
    install_git as install_git_source, install_npm as install_npm_source,
    uninstall as uninstall_extension, update as update_managed_extension,
};
pub(crate) use manifest::load_local as install_local;
pub(crate) use operation_error::{report as report_operation_error, Operation};
pub(crate) use validation::identifier as validate_identifier;

#[cfg(test)]
mod builtin_tests;
#[cfg(test)]
mod git_dependencies_tests;
#[cfg(test)]
mod git_source_tests;
#[cfg(test)]
mod managed_store_tests;
#[cfg(test)]
mod npm_runner_tests;
#[cfg(test)]
mod runtime_sync_tests;
#[cfg(test)]
mod source_validation_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;

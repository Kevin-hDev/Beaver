mod git_checkout;
#[cfg(test)]
mod git_dependencies_tests;
mod git_package;
#[cfg(test)]
mod git_policy_tests;
mod git_reference;
mod git_resolution;
mod git_resolution_history;
mod git_source;
#[cfg(test)]
mod git_source_reference_tests;
#[cfg(test)]
mod git_source_tests;
mod git_transport;
pub(crate) mod install_jobs;
mod install_retry;
mod install_signal;
mod installer;
mod installer_process;
mod installer_record;
mod installer_uninstall;
mod managed_cleanup;
#[cfg(test)]
mod managed_install_error_tests;
mod managed_store;
#[cfg(test)]
mod managed_store_tests;
mod managed_tree;
mod npm_environment;
mod npm_paths;
mod npm_runner;
#[cfg(test)]
mod npm_runner_tests;
mod npm_source;
mod npm_workspace;
mod operation_error;
mod operation_failure;
mod operation_log;
mod source_validation;

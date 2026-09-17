mod ui_action_result;
mod ui_artifact;
mod ui_artifact_manifest;
mod ui_artifact_store;
mod ui_build_api;
mod ui_builder;
mod ui_builder_build;
mod ui_builder_paths;
mod ui_builder_process;
mod ui_catalog;
mod ui_catalog_actions;
mod ui_catalog_lifecycle;
mod ui_catalog_limits;
mod ui_dispatch;
mod ui_normalization;
pub(crate) mod ui_protocol;
mod ui_protocol_proof;
mod ui_protocol_response;
mod ui_types;
mod ui_validation;
mod ui_view_validation;
mod view;
#[allow(dead_code)]
mod ui_contract {
    include!(concat!(env!("OUT_DIR"), "/extension_ui_contract.rs"));
}
#[cfg(test)]
mod ui_artifact_tests;
#[cfg(test)]
mod ui_builder_tests;
#[cfg(test)]
mod ui_contract_tests;
#[cfg(test)]
mod ui_dispatch_tests;
#[cfg(test)]
mod ui_limit_tests;
#[cfg(test)]
mod ui_protocol_tests;
mod ui_startup;
mod ui_startup_ack;
mod ui_startup_platform;
mod ui_startup_state;
#[cfg(test)]
mod ui_startup_tests;
#[cfg(test)]
mod ui_validation_tests;
#[cfg(test)]
mod view_tests;

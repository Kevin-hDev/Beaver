mod builtin;
mod contribution_path;
mod contribution_resources;
#[cfg(test)]
mod contribution_resources_tests;
mod contribution_skills;
#[cfg(test)]
mod contribution_skills_tests;
mod contribution_types;
mod core_secrets;
mod discovery_catalog;
mod discovery_inspection;
mod discovery_limits;
mod discovery_listing;
mod discovery_preferences;
mod discovery_result_serialization;
mod discovery_usage;
pub(crate) mod error_codes;
mod extension_internal_exports;
mod message_validation;
mod origin_validation;
mod protocol;
mod public_api;
pub(crate) mod types;
mod validation;
#[allow(dead_code)]
mod discovery_contract {
    include!(concat!(env!("OUT_DIR"), "/extension_discovery_contract.rs"));
}
#[cfg(test)]
mod builtin_tests;
#[cfg(test)]
mod contract_artifact_tests;
#[cfg(test)]
mod contribution_contract_tests;
mod core_api_contract;
#[cfg(test)]
mod core_api_contract_tests;
mod core_api_dispatch;
mod core_api_permissions;
#[cfg(test)]
mod core_api_permissions_tests;
#[cfg(test)]
mod core_api_test_support;
mod core_automations;
mod core_automations_params;
#[cfg(test)]
mod core_automations_tests;
mod core_bridge;
mod core_call_quota;
mod core_memory;
mod core_memory_content;
mod core_memory_mutations;
mod core_memory_reads;
mod core_mcp;
mod core_model_catalog;
mod core_model_generation;
mod core_model_quota;
mod core_models;
mod core_response_audit;
#[cfg(test)]
mod core_response_audit_tests;
pub(crate) mod core_scope;
#[cfg(test)]
mod core_scope_tests;
mod core_subagents;
#[cfg(test)]
mod core_subagents_tests;
#[cfg(test)]
mod source_validation_tests;
#[cfg(test)]
mod tests;

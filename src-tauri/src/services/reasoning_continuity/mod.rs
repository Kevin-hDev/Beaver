pub(crate) mod bounded_json;
pub mod capture_budget;
mod continuation_target;
pub mod contract;
pub mod diagnostics;
pub mod eligibility;
pub mod envelope;
pub mod fingerprint;
pub mod limits;
pub mod registry;
mod registry_anthropic;
mod registry_inventory;
mod registry_validated_cloud;
mod registry_validated_reasoning;
pub mod tool_links;

#[cfg(test)]
mod diagnostics_tests;
#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod registry_tests;

#![allow(
    dead_code,
    reason = "the closed domain is adopted incrementally by Tasks 4 through 23"
)]

mod bounded_json;
pub mod capture_budget;
mod continuation_target;
pub mod contract;
pub mod eligibility;
pub mod envelope;
pub mod limits;
pub mod registry;
mod registry_inventory;
pub mod tool_links;

#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod registry_tests;

#![allow(
    dead_code,
    reason = "the closed domain is adopted incrementally by Tasks 4 through 23"
)]

pub mod contract;
pub mod eligibility;
pub mod envelope;
pub mod limits;
pub mod registry;
pub mod tool_links;

#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod registry_tests;

pub mod agent_session_contract;
pub mod agent_turn_contract;
mod agent_turn_contract_wire;
// Temporaire : Task 2 branche ces types sur le stockage durable.
#[allow(dead_code)]
pub mod automation;
pub mod compression_profile_contract;
pub mod config;
pub mod file_tree;
pub mod gateway_config;
pub mod mascot;
pub mod provider_contract;

pub use automation::*;
pub use config::*;
pub use gateway_config::*;
pub use mascot::*;

#[cfg(test)]
mod agent_session_contract_tests;
#[cfg(test)]
mod agent_turn_contract_tests;
#[cfg(test)]
mod compression_profile_contract_tests;
#[cfg(test)]
mod file_tree_tests;
#[cfg(test)]
mod provider_contract_tests;

pub mod extension_skill_loader;
#[cfg(test)]
mod extension_skill_loader_tests;
mod extension_tool_authority;
pub mod extension_tool_set;
mod tool_extension_catalog_diagnostics;
pub mod tool_extension_inspect;
pub mod tool_extension_list;
pub mod tool_extension_resource;
#[cfg(test)]
mod tool_extension_resource_tests;
#[allow(dead_code)]
mod extension_discovery_contract {
    include!(concat!(env!("OUT_DIR"), "/extension_discovery_contract.rs"));
}
#[cfg(test)]
mod extension_discovery_contract_tests;

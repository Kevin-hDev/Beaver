#[path = "extension_discovery_contract_budget.rs"]
mod budget;
#[path = "extension_discovery_contract_rust.rs"]
mod rust_renderer;
#[path = "extension_discovery_contract_validation.rs"]
mod validation;
use super::extension_contract_shared as shared;

use serde_json::Value;
use std::path::Path;

const CONTRACT_PATH: &str = "resources/extension-discovery/contract.json";
const CONTRACT_BOOTSTRAP_PATH: &str = "resources/extension-discovery/contract-bootstrap.json";

pub fn generate() {
    println!("cargo:rerun-if-changed={CONTRACT_PATH}");
    println!("cargo:rerun-if-changed={CONTRACT_BOOTSTRAP_PATH}");
    let discovery_directory = Path::new("resources/extension-discovery");
    let host_directory = Path::new("resources/extension-host");
    let discovery = load_contract(discovery_directory).unwrap_or_else(|error| panic!("{error}"));
    let host = shared::load_contract(
        host_directory,
        "Beaver extension contract",
        "Beaver extension contract exceeds its size limit",
    )
    .unwrap_or_else(|error| panic!("{error}"));
    validate_contract(&discovery, &host).unwrap_or_else(|error| panic!("{error}"));
    let output = rust_renderer::render(&discovery, &host).unwrap_or_else(|error| panic!("{error}"));
    let out_dir = std::env::var_os("OUT_DIR").expect("missing Cargo OUT_DIR");
    std::fs::write(
        Path::new(&out_dir).join("extension_discovery_contract.rs"),
        output,
    )
    .expect("cannot generate the Beaver extension discovery contract");
}

pub fn load_contract(directory: &Path) -> Result<Value, String> {
    shared::load_contract(
        directory,
        "Beaver extension discovery contract",
        "Beaver extension discovery contract exceeds its size limit",
    )
}

pub fn validate_contract(discovery: &Value, host: &Value) -> Result<(), String> {
    validation::validate(discovery, host)
}

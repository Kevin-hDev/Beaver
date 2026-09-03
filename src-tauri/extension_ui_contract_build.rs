#[path = "extension_ui_contract_artifacts.rs"]
#[allow(dead_code)]
mod artifacts;
#[path = "extension_ui_contract_document.rs"]
#[allow(dead_code)]
mod document;
#[path = "extension_ui_contract_rust_objects.rs"]
mod rust_objects;
#[path = "extension_ui_contract_rust.rs"]
mod rust_renderer;
#[path = "extension_ui_contract_schema.rs"]
mod schema;
#[path = "extension_ui_contract_validation.rs"]
mod validation;
use super::extension_contract_shared as shared;

use serde_json::Value;
use std::path::Path;

const CONTRACT_DIRECTORY: &str = "resources/extension-ui";
const GENERATED_BEGIN: &str = "<!-- BEGIN GENERATED EXTENSION UI CONTRACT -->";
const GENERATED_END: &str = "<!-- END GENERATED EXTENSION UI CONTRACT -->";

pub fn generate() {
    println!("cargo:rerun-if-changed={CONTRACT_DIRECTORY}/contract-bootstrap.json");
    println!("cargo:rerun-if-changed={CONTRACT_DIRECTORY}/contract.json");
    let contract =
        load_contract(Path::new(CONTRACT_DIRECTORY)).unwrap_or_else(|error| panic!("{error}"));
    validation::validate(&contract).unwrap_or_else(|error| panic!("{error}"));
    let rust = rust_renderer::render(&contract).unwrap_or_else(|error| panic!("{error}"));
    let out_dir = std::env::var_os("OUT_DIR").expect("missing Cargo OUT_DIR");
    std::fs::write(Path::new(&out_dir).join("extension_ui_contract.rs"), rust)
        .expect("cannot generate the Beaver extension UI contract");
}

pub fn load_contract(directory: &Path) -> Result<Value, String> {
    shared::load_contract(
        directory,
        "Beaver extension UI contract",
        "Beaver extension UI contract exceeds its size limit",
    )
}

#[allow(dead_code)]
pub fn validate_contract(contract: &Value) -> Result<(), String> {
    validation::validate(contract)
}

#[allow(dead_code)]
pub fn render_typescript(contract: &Value) -> Result<String, String> {
    artifacts::render_typescript(contract)
}

#[allow(dead_code)]
pub fn render_sdk_contract(contract: &Value) -> Result<String, String> {
    artifacts::render_sdk(contract)
}

#[allow(dead_code)]
pub fn render_node(contract: &Value) -> Result<String, String> {
    artifacts::render_node(contract)
}

#[allow(dead_code)]
pub fn generated_document_section(contract: &Value) -> Result<String, String> {
    document::render(contract)
}

#[allow(dead_code)]
pub fn export_artifacts(manifest_root: &Path) -> Result<(), String> {
    let contract = load_contract(&manifest_root.join(CONTRACT_DIRECTORY))?;
    validation::validate(&contract)?;
    shared::write(
        &manifest_root.join("../src/types/extension-ui-contract.generated.ts"),
        &artifacts::render_typescript(&contract)?,
    )?;
    shared::write(
        &manifest_root.join("resources/extension-host/sdk/ui-contract.d.ts"),
        &artifacts::render_sdk(&contract)?,
    )?;
    shared::write(
        &manifest_root.join("resources/extension-host/ui-contract.mjs"),
        &artifacts::render_node(&contract)?,
    )?;
    let section = document::render(&contract)?;
    shared::update_document(
        &manifest_root.join("resources/extension-host/sdk/README.md"),
        &section,
        GENERATED_BEGIN,
        GENERATED_END,
    )?;
    let private = shared::private_api_path(manifest_root)?;
    if private.exists() {
        shared::update_document(&private, &section, GENERATED_BEGIN, GENERATED_END)?;
    }
    Ok(())
}

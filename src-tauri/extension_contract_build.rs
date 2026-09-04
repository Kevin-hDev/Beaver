#[path = "extension_contract_artifacts.rs"]
#[allow(dead_code)]
mod artifacts;
#[path = "extension_contract_document.rs"]
#[allow(dead_code)]
mod document;
#[path = "extension_contract_effect.rs"]
mod effect_renderer;
#[path = "extension_contract_enum.rs"]
mod enum_renderer;
#[path = "extension_contract_io.rs"]
mod io;
#[path = "extension_contract_rust.rs"]
mod rust_renderer;
#[path = "extension_contract_validation.rs"]
mod validation;
use super::extension_contract_shared as shared;

use serde_json::Value;
use std::path::{Path, PathBuf};

const CONTRACT_PATH: &str = "resources/extension-host/contract.json";
const CONTRACT_BOOTSTRAP_PATH: &str = "resources/extension-host/contract-bootstrap.json";
#[allow(dead_code)]
pub const BOOTSTRAP_FILE_MAX_BYTES: usize = shared::BOOTSTRAP_FILE_MAX_BYTES;
#[allow(dead_code)]
pub const MAX_BOOTSTRAPPED_CONTRACT_BYTES: usize = shared::MAX_BOOTSTRAPPED_CONTRACT_BYTES;
const GENERATED_BEGIN: &str = "<!-- BEGIN GENERATED EXTENSION CONTRACT -->";
const GENERATED_END: &str = "<!-- END GENERATED EXTENSION CONTRACT -->";

#[allow(unused_imports)]
pub use artifacts::{render_sdk_contract, render_typescript};
#[allow(unused_imports)]
pub use document::generated_document_section;

pub fn generate() {
    println!("cargo:rerun-if-changed={CONTRACT_PATH}");
    println!("cargo:rerun-if-changed={CONTRACT_BOOTSTRAP_PATH}");
    println!("cargo:rerun-if-changed=resources/extension-host/builtin-plugins/catalog.json");
    let directory = Path::new("resources/extension-host");
    let contract = load_contract(directory).unwrap_or_else(|error| panic!("{error}"));
    validate_contract(&contract, directory).unwrap_or_else(|error| panic!("{error}"));
    let output = rust_renderer::render(&contract).unwrap_or_else(|error| panic!("{error}"));
    let out_dir = std::env::var_os("OUT_DIR").expect("missing Cargo OUT_DIR");
    std::fs::write(Path::new(&out_dir).join("extension_contract.rs"), output)
        .expect("cannot generate the Beaver extension contract");
}

pub fn load_contract(directory: &Path) -> Result<Value, String> {
    shared::load_contract(
        directory,
        "Beaver extension contract",
        "Beaver extension contract exceeds its size limit",
    )
}

pub fn validate_contract(contract: &Value, directory: &Path) -> Result<(), String> {
    validation::validate(contract, directory)
}

#[allow(dead_code)]
pub fn export_artifacts(manifest_root: &Path) -> Result<(), String> {
    let directory = manifest_root.join("resources/extension-host");
    let contract = load_contract(&directory)?;
    validate_contract(&contract, &directory)?;
    write(
        &manifest_root.join("../src/types/extension-contract.generated.ts"),
        &artifacts::render_typescript(&contract)?,
    )?;
    write(
        &directory.join("sdk/contract.d.ts"),
        &artifacts::render_sdk_contract(&contract)?,
    )?;
    update_document(
        &directory.join("sdk/README.md"),
        &document::generated_document_section(&contract)?,
    )?;
    update_private_document_if_present(
        &private_api_path(manifest_root)?,
        &document::generated_document_section(&contract)?,
    )
}

pub fn update_private_document_if_present(path: &Path, section: &str) -> Result<(), String> {
    // The private planning documentation is intentionally ignored by Git, so
    // contract generation must also work in a clean clone where it is absent.
    if !path.exists() {
        return Ok(());
    }
    update_document(path, section)
}

fn update_document(path: &Path, section: &str) -> Result<(), String> {
    shared::update_document(path, section, GENERATED_BEGIN, GENERATED_END)
}

fn write(path: &Path, content: &str) -> Result<(), String> {
    shared::write(path, content)
}

pub fn private_api_path(manifest_root: &Path) -> Result<PathBuf, String> {
    shared::private_api_path(manifest_root)
}

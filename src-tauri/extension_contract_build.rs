#[path = "extension_contract_artifacts.rs"]
#[allow(dead_code)]
mod artifacts;
#[path = "extension_contract_document.rs"]
#[allow(dead_code)]
mod document;
#[path = "extension_contract_io.rs"]
mod io;
#[path = "extension_contract_rust.rs"]
mod rust_renderer;
#[path = "extension_contract_validation.rs"]
mod validation;

use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

const CONTRACT_PATH: &str = "resources/extension-host/contract.json";
const CONTRACT_BOOTSTRAP_PATH: &str = "resources/extension-host/contract-bootstrap.json";
pub const BOOTSTRAP_FILE_MAX_BYTES: usize = 256;
pub const MAX_BOOTSTRAPPED_CONTRACT_BYTES: usize = 1_048_576;
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
    let bootstrap = io::read_bounded(
        &directory.join("contract-bootstrap.json"),
        BOOTSTRAP_FILE_MAX_BYTES,
        "Beaver extension contract bootstrap exceeds its size limit",
    )?;
    let bootstrap: Map<String, Value> = serde_json::from_slice(&bootstrap)
        .map_err(|_| "invalid Beaver extension contract bootstrap JSON".to_string())?;
    if bootstrap.len() != 1 {
        return Err("invalid Beaver extension contract bootstrap".to_string());
    }
    let max_contract_bytes = bootstrap
        .get("maxContractBytes")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| (1..=MAX_BOOTSTRAPPED_CONTRACT_BYTES).contains(value))
        .ok_or_else(|| "invalid Beaver extension contract bootstrap".to_string())?;
    let raw = io::read_bounded(
        &directory.join("contract.json"),
        max_contract_bytes,
        "Beaver extension contract exceeds its size limit",
    )?;
    serde_json::from_slice(&raw).map_err(|_| "invalid Beaver extension contract JSON".to_string())
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
    let source = io::read_bounded(path, 1_048_576, "generated document exceeds its size limit")?;
    let source = String::from_utf8(source)
        .map_err(|_| "generated document is not valid UTF-8".to_string())?;
    let updated = if let (Some(begin), Some(end)) =
        (source.find(GENERATED_BEGIN), source.find(GENERATED_END))
    {
        let after = end + GENERATED_END.len();
        format!("{}{}{}", &source[..begin], section, &source[after..])
    } else {
        format!("{}\n\n{}\n", source.trim_end(), section)
    };
    write(path, &updated)
}

fn write(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|_| format!("cannot write generated artifact: {}", path.display()))
}

pub fn private_api_path(manifest_root: &Path) -> Result<PathBuf, String> {
    let checkout = manifest_root
        .parent()
        .ok_or_else(|| "invalid manifest root".to_string())?;
    let git = checkout.join(".git");
    let repository = if git.is_dir() {
        checkout.to_path_buf()
    } else {
        let pointer = std::fs::read_to_string(&git)
            .map_err(|_| "cannot resolve private documentation root".to_string())?;
        let git_dir = pointer
            .trim()
            .strip_prefix("gitdir: ")
            .ok_or_else(|| "cannot resolve private documentation root".to_string())?;
        let marker = "/.git/worktrees/";
        let root = git_dir
            .find(marker)
            .map(|index| &git_dir[..index])
            .ok_or_else(|| "cannot resolve private documentation root".to_string())?;
        PathBuf::from(root)
    };
    Ok(repository.join("docs/fonctionnalites/extension/EXTENSION_API_V1.md"))
}

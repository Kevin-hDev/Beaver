use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use subtle::ConstantTimeEq;

use super::types::{
    ExtensionKind, ExtensionRecord, ExtensionStatus, ExtensionUiArtifact,
    ExtensionUiArtifactOutput, ExtensionUiMode,
};

const MAX_INPUTS: usize = 8_192;
pub(super) const MAX_MANIFEST_BYTES: u64 = 1_048_576;

pub(super) use super::ui_artifact_manifest::{manifest_bytes, manifest_hash};

pub(super) fn validate(artifact: &ExtensionUiArtifact) -> Result<(), String> {
    if artifact.version != 1
        || artifact.outputs.is_empty()
        || artifact.outputs.len() > super::ui_contract::MAX_ADVANCED_ARTIFACT_FILES
        || artifact.inputs.is_empty()
        || artifact.inputs.len() > MAX_INPUTS
        || !version(&artifact.builder_version, false)
        || !version(&artifact.node_version, true)
        || super::runtime_version::validate_node(&artifact.node_version).is_err()
        || !valid_name(&artifact.entry)
        || !valid_sha(&artifact.manifest_sha256)
    {
        return Err(invalid());
    }
    validate_inputs(&artifact.inputs)?;
    validate_outputs(artifact)?;
    let expected = manifest_hash(artifact)?;
    constant_sha(&artifact.manifest_sha256, &expected)
        .then_some(())
        .ok_or_else(invalid)
}

pub(super) fn validate_record(record: &ExtensionRecord) -> Result<(), String> {
    match (
        record.manifest.ui.as_ref().map(|ui| &ui.mode),
        &record.ui_artifact,
    ) {
        (Some(ExtensionUiMode::Advanced), Some(artifact)) => validate(artifact),
        (Some(ExtensionUiMode::Advanced), None) if record.kind == ExtensionKind::Builtin => Ok(()),
        (Some(ExtensionUiMode::Advanced), None)
            if !record.enabled
                && !record.trusted
                && record.status == ExtensionStatus::Error
                && record.last_error.as_deref()
                    == Some(super::ui_contract::UI_DIAGNOSTIC_UI_ARTIFACT_INVALID) =>
        {
            Ok(())
        }
        (Some(ExtensionUiMode::Advanced), None) => Err("Artefact UI avancé manquant.".to_string()),
        (_, None) => Ok(()),
        (_, Some(_)) => Err("Artefact UI avancé inattendu.".to_string()),
    }
}

pub(super) fn verify_at(root: &Path, artifact: &ExtensionUiArtifact) -> Result<(), String> {
    validate(artifact)?;
    let root = canonical_directory(root)?;
    let manifest = read_manifest(&root.join("manifest.json"))?;
    validate(&manifest)?;
    if !super::ui_artifact_manifest::same_metadata(&manifest, artifact) {
        return Err(invalid());
    }
    for output in &artifact.outputs {
        verify_output(&root, output)?;
    }
    Ok(())
}

fn validate_inputs(inputs: &[String]) -> Result<(), String> {
    let mut previous: Option<&str> = None;
    for input in inputs {
        super::fingerprint_paths::clean_relative(input)?;
        if previous.is_some_and(|value| value >= input.as_str()) {
            return Err(invalid());
        }
        previous = Some(input);
    }
    Ok(())
}

fn validate_outputs(artifact: &ExtensionUiArtifact) -> Result<(), String> {
    let mut names = HashSet::with_capacity(artifact.outputs.len());
    let mut total = 0_usize;
    let mut javascript = 0_usize;
    let mut entry_is_javascript = false;
    let mut previous: Option<&str> = None;
    for output in &artifact.outputs {
        if !valid_output(output)
            || !names.insert(output.name.as_str())
            || previous.is_some_and(|value| value >= output.name.as_str())
        {
            return Err(invalid());
        }
        previous = Some(&output.name);
        javascript += usize::from(output.kind == "javascript");
        entry_is_javascript |= output.name == artifact.entry && output.kind == "javascript";
        total = total.checked_add(output.bytes).ok_or_else(invalid)?;
    }
    if javascript != 1
        || !entry_is_javascript
        || total != artifact.total_bytes
        || total > super::ui_contract::MAX_ADVANCED_ARTIFACT_BYTES
    {
        return Err(invalid());
    }
    Ok(())
}

fn valid_output(output: &ExtensionUiArtifactOutput) -> bool {
    valid_name(&output.name)
        && valid_sha(&output.sha256)
        && matches!(
            (output.kind.as_str(), extension(&output.name)),
            ("javascript", Some("js" | "mjs"))
                | ("css", Some("css"))
                | ("png", Some("png"))
                | ("jpeg", Some("jpg" | "jpeg"))
                | ("webp", Some("webp"))
                | ("gif", Some("gif"))
                | ("woff2", Some("woff2"))
        )
}

fn verify_output(root: &Path, output: &ExtensionUiArtifactOutput) -> Result<(), String> {
    let path = root.join(&output.name);
    let metadata = super::fingerprint_paths::reject_symlink(&path)?;
    if !metadata.is_file() || metadata.len() != output.bytes as u64 {
        return Err(invalid());
    }
    let canonical = dunce::canonicalize(&path).map_err(|_| invalid())?;
    if !canonical.starts_with(root) {
        return Err(invalid());
    }
    let actual = hash_file(&canonical, output.bytes)?;
    constant_sha(&output.sha256, &actual)
        .then_some(())
        .ok_or_else(invalid)
}

fn hash_file(path: &Path, bytes: usize) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|_| invalid())?;
    let mut reader = file.take(bytes as u64 + 1);
    let mut content = Vec::with_capacity(bytes);
    reader.read_to_end(&mut content).map_err(|_| invalid())?;
    if content.len() != bytes {
        return Err(invalid());
    }
    Ok(hex::encode(Sha256::digest(content)))
}

fn read_manifest(path: &Path) -> Result<ExtensionUiArtifact, String> {
    let bytes = crate::services::private_store::read_bounded_regular(path, MAX_MANIFEST_BYTES)
        .map_err(|_| invalid())?;
    let crate::services::private_store::BoundedFile::Content(bytes) = bytes else {
        return Err(invalid());
    };
    serde_json::from_slice(&bytes).map_err(|_| invalid())
}

fn canonical_directory(path: &Path) -> Result<std::path::PathBuf, String> {
    let metadata = super::fingerprint_paths::reject_symlink(path)?;
    if !metadata.is_dir() {
        return Err(invalid());
    }
    dunce::canonicalize(path).map_err(|_| invalid())
}

fn constant_sha(left: &str, right: &str) -> bool {
    bool::from(left.as_bytes().ct_eq(right.as_bytes()))
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn version(value: &str, allow_v: bool) -> bool {
    let value = if allow_v {
        value.strip_prefix('v').unwrap_or(value)
    } else {
        value
    };
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn extension(name: &str) -> Option<&str> {
    Path::new(name).extension()?.to_str()
}

fn invalid() -> String {
    super::ui_contract::UI_DIAGNOSTIC_UI_ARTIFACT_INVALID.to_string()
}

use serde::Serialize;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::types::{ExtensionUiArtifact, ExtensionUiArtifactOutput};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnsignedArtifact<'a> {
    version: u8,
    builder_version: &'a str,
    node_version: &'a str,
    entry: &'a str,
    total_bytes: usize,
    outputs: &'a [ExtensionUiArtifactOutput],
    inputs: &'a [String],
}

pub(super) fn manifest_bytes(artifact: &ExtensionUiArtifact) -> Result<Vec<u8>, String> {
    super::ui_artifact::validate(artifact)?;
    serde_json::to_vec(artifact)
        .ok()
        .filter(|bytes| bytes.len() <= super::ui_artifact::MAX_MANIFEST_BYTES as usize)
        .ok_or_else(invalid)
}

pub(super) fn manifest_hash(artifact: &ExtensionUiArtifact) -> Result<String, String> {
    let unsigned = UnsignedArtifact {
        version: artifact.version,
        builder_version: &artifact.builder_version,
        node_version: &artifact.node_version,
        entry: &artifact.entry,
        total_bytes: artifact.total_bytes,
        outputs: &artifact.outputs,
        inputs: &artifact.inputs,
    };
    let bytes = serde_json::to_vec(&unsigned).map_err(|_| invalid())?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

pub(super) fn same_metadata(left: &ExtensionUiArtifact, right: &ExtensionUiArtifact) -> bool {
    left.version == right.version
        && left.builder_version == right.builder_version
        && left.node_version == right.node_version
        && left.entry == right.entry
        && left.total_bytes == right.total_bytes
        && left.outputs == right.outputs
        && left.inputs == right.inputs
        && bool::from(
            left.manifest_sha256
                .as_bytes()
                .ct_eq(right.manifest_sha256.as_bytes()),
        )
}

fn invalid() -> String {
    super::ui_contract::UI_DIAGNOSTIC_UI_ARTIFACT_INVALID.to_string()
}

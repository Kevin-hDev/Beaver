use sha2::{Digest, Sha256};

pub(crate) fn extension_resource_artifact(
    resource: super::resource_loader::LoadedResource,
) -> Result<Option<crate::services::agent_local::tool_artifact::EphemeralArtifact>, ()> {
    if resource.signature == crate::services::file_signature::FileSignature::Utf8 {
        return Ok(None);
    }
    let bytes = u64::try_from(resource.bytes.len()).map_err(|_| ())?;
    let sha256 = hex::encode(Sha256::digest(&resource.bytes));
    let metadata = crate::services::agent_local::tool_artifact::ArtifactMetadata {
        name: resource.name,
        mime_type: resource.signature.mime().to_string(),
        bytes,
        sha256,
        purpose: crate::services::agent_local::tool_artifact::ArtifactPurpose::Artifact,
        source: crate::services::agent_local::tool_artifact::ArtifactSource::ExtensionResource {
            resource_id: resource.qualified_resource_id,
            catalog_fingerprint: resource.catalog_fingerprint,
        },
    };
    Ok(Some(
        crate::services::agent_local::tool_artifact::EphemeralArtifact {
            metadata,
            bytes: resource.bytes,
        },
    ))
}

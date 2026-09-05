use serde::Serialize;

pub(crate) use crate::services::extensions::types::ExtensionResultFilePurpose as ArtifactPurpose;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactMetadata {
    pub name: String,
    pub mime_type: String,
    pub bytes: u64,
    pub sha256: String,
    pub purpose: ArtifactPurpose,
    pub source: ArtifactSource,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum ArtifactSource {
    WorkspaceFile {
        path: std::path::PathBuf,
        grant: String,
    },
    ExtensionResource {
        resource_id: String,
        catalog_fingerprint: String,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct EphemeralArtifact {
    pub metadata: ArtifactMetadata,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(crate) struct PendingArtifact {
    pub relative_path: String,
    pub display_name: Option<String>,
    pub purpose: ArtifactPurpose,
}

#[derive(Clone, Debug)]
pub(crate) struct PendingExtensionResource {
    pub session_id: String,
    pub name: String,
    pub extension_id: String,
    pub qualified_resource_id: String,
    pub catalog_fingerprint: String,
    pub root: std::path::PathBuf,
    pub relative_path: String,
}

impl From<crate::services::extensions::PreparedResource> for PendingExtensionResource {
    fn from(resource: crate::services::extensions::PreparedResource) -> Self {
        Self {
            session_id: resource.session_id,
            name: resource.name,
            extension_id: resource.extension_id,
            qualified_resource_id: resource.qualified_resource_id,
            catalog_fingerprint: resource.catalog_fingerprint,
            root: resource.root,
            relative_path: resource.relative_path,
        }
    }
}

impl PendingArtifact {
    pub(crate) fn from_validated(
        relative_path: String,
        display_name: Option<String>,
        purpose: ArtifactPurpose,
    ) -> Self {
        Self {
            relative_path,
            display_name,
            purpose,
        }
    }
}

impl From<&ArtifactMetadata> for super::tool_artifact_record::ToolArtifactRecord {
    fn from(metadata: &ArtifactMetadata) -> Self {
        use super::tool_artifact_record::{ToolArtifactPurpose, ToolArtifactSource};

        let purpose = match metadata.purpose {
            ArtifactPurpose::Artifact => ToolArtifactPurpose::Artifact,
            ArtifactPurpose::Preview => ToolArtifactPurpose::Preview,
        };
        let source = match &metadata.source {
            ArtifactSource::WorkspaceFile { path, grant } => ToolArtifactSource::WorkspaceFile {
                path: path.to_string_lossy().into_owned(),
                grant: grant.clone(),
            },
            ArtifactSource::ExtensionResource {
                resource_id,
                catalog_fingerprint,
            } => ToolArtifactSource::ExtensionResource {
                resource_id: resource_id.clone(),
                catalog_fingerprint: catalog_fingerprint.clone(),
            },
        };
        Self {
            name: metadata.name.clone(),
            mime_type: metadata.mime_type.clone(),
            bytes: metadata.bytes,
            sha256: metadata.sha256.clone(),
            purpose,
            source,
        }
    }
}

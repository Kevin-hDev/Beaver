use serde::{Deserialize, Serialize};

use crate::models::agent_turn_contract::{
    MAX_ATTACHMENT_GRANT_BYTES, MAX_ATTACHMENT_MIME_BYTES, MAX_ATTACHMENT_PATH_BYTES,
};

pub(crate) const MAX_ARTIFACTS_PER_TOOL: usize =
    crate::services::extensions::types::MAX_RESULT_FILES;
const SHA256_HEX_BYTES: usize = 64;
const QUALIFIED_RESOURCE_ID_SEPARATOR_BYTES: usize = 2;
const QUALIFIED_RESOURCE_ID_PREFIX_BYTES: usize = "extension".len();
const MAX_QUALIFIED_RESOURCE_ID_BYTES: usize = QUALIFIED_RESOURCE_ID_PREFIX_BYTES
    + QUALIFIED_RESOURCE_ID_SEPARATOR_BYTES
    + crate::services::extensions::types::MAX_IDENTIFIER_CHARS * 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub(crate) enum ToolArtifactPurpose {
    Artifact,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub(crate) enum ToolArtifactStatus {
    Intact,
    Absent,
    Modified,
    Inaccessible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(test, ts(tag = "kind", rename_all = "snake_case"))]
pub(crate) enum ToolArtifactSource {
    WorkspaceFile { path: String, grant: String },
    ExtensionResource { resource_id: String, catalog_fingerprint: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub(crate) struct ToolArtifactRecord {
    pub name: String,
    pub mime_type: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub bytes: u64,
    pub sha256: String,
    pub purpose: ToolArtifactPurpose,
    pub source: ToolArtifactSource,
}

pub(crate) fn validate(artifacts: &[ToolArtifactRecord]) -> Result<(), ()> {
    if artifacts.len() > MAX_ARTIFACTS_PER_TOOL {
        return Err(());
    }
    for artifact in artifacts {
        if !safe_text_chars(
            &artifact.name,
            crate::services::extensions::types::MAX_EXTENSION_NAME_CHARS,
        )
            || !safe_text_bytes(&artifact.mime_type, MAX_ATTACHMENT_MIME_BYTES)
            || artifact.bytes > crate::services::extensions::types::MAX_RESULT_BYTES as u64
            || artifact.sha256.len() != SHA256_HEX_BYTES
            || !artifact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(());
        }
        match &artifact.source {
            ToolArtifactSource::WorkspaceFile { path, grant } => {
                if !safe_text_bytes(path, MAX_ATTACHMENT_PATH_BYTES)
                    || !safe_text_bytes(grant, MAX_ATTACHMENT_GRANT_BYTES)
                {
                    return Err(());
                }
            }
            ToolArtifactSource::ExtensionResource {
                resource_id,
                catalog_fingerprint,
            } => {
                if !safe_text_bytes(resource_id, MAX_QUALIFIED_RESOURCE_ID_BYTES)
                    || crate::services::extensions::parse_qualified_contribution_id(resource_id)
                        .is_err()
                    || catalog_fingerprint.len() != SHA256_HEX_BYTES
                    || !catalog_fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    return Err(());
                }
            }
        }
    }
    Ok(())
}

fn safe_text_bytes(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
}

fn safe_text_chars(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= maximum
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact() -> ToolArtifactRecord {
        ToolArtifactRecord {
            name: "preview.png".into(),
            mime_type: "image/png".into(),
            bytes: 8,
            sha256: "a".repeat(SHA256_HEX_BYTES),
            purpose: ToolArtifactPurpose::Preview,
            source: ToolArtifactSource::ExtensionResource {
                resource_id: "extension:sample:preview".into(),
                catalog_fingerprint: "b".repeat(SHA256_HEX_BYTES),
            },
        }
    }

    #[test]
    fn accepts_the_exact_limit_and_rejects_unbounded_or_malformed_metadata() {
        assert!(validate(&vec![artifact(); MAX_ARTIFACTS_PER_TOOL]).is_ok());
        assert!(validate(&vec![artifact(); MAX_ARTIFACTS_PER_TOOL + 1]).is_err());

        let mut invalid = artifact();
        invalid.name = "\n".into();
        assert!(validate(&[invalid]).is_err());
    }

    #[test]
    fn serializes_metadata_without_binary_payload_or_extension_path() {
        let json = serde_json::to_string(&artifact()).expect("serialize");

        assert!(!json.contains("base64"));
        assert!(!json.contains("relative_path"));
        assert!(!json.contains("bytes_data"));
        assert!(!json.contains("verification"));
    }

    #[test]
    fn reuses_shared_text_limits_and_counts_visible_names_as_characters() {
        let mut exact = artifact();
        exact.name = "🦫".repeat(crate::services::extensions::types::MAX_EXTENSION_NAME_CHARS);
        assert!(validate(&[exact.clone()]).is_ok());

        exact.name.push('x');
        assert!(validate(&[exact]).is_err());

        let mut grant = artifact();
        grant.source = ToolArtifactSource::WorkspaceFile {
            path: "p".repeat(MAX_ATTACHMENT_PATH_BYTES),
            grant: "g".repeat(MAX_ATTACHMENT_GRANT_BYTES),
        };
        assert!(validate(&[grant]).is_ok());
    }

}

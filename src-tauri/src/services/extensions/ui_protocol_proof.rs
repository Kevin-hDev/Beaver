use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct ProofArtifact {
    root: PathBuf,
    pub(super) extension_id: String,
    pub(super) manifest_sha: String,
    content_sha256: String,
    expected_bytes: Option<usize>,
}

impl ProofArtifact {
    #[cfg(any(test, feature = "e2e"))]
    fn new(root: PathBuf, extension_id: &str, manifest_sha: &str) -> Option<Self> {
        let root = root.canonicalize().ok()?;
        let metadata = std::fs::symlink_metadata(&root).ok()?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || super::validation::identifier(extension_id).is_err()
            || !super::ui_protocol::valid_sha(manifest_sha)
        {
            return None;
        }
        Some(Self {
            root,
            extension_id: extension_id.to_string(),
            manifest_sha: manifest_sha.to_string(),
            // The E2E directory identifier is the content digest, not a manifest digest.
            content_sha256: manifest_sha.to_string(),
            expected_bytes: None,
        })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    pub(super) fn expected_bytes(&self) -> Option<usize> {
        self.expected_bytes
    }

    #[cfg(test)]
    pub(super) fn for_test(root: PathBuf, extension_id: &str, manifest_sha: &str) -> Self {
        let mut artifact =
            Self::new(root, extension_id, manifest_sha).expect("valid UI protocol test artifact");
        artifact.expected_bytes = std::fs::metadata(
            artifact
                .root
                .join(extension_id)
                .join(manifest_sha)
                .join("entry.mjs"),
        )
        .ok()
        .and_then(|metadata| usize::try_from(metadata.len()).ok());
        artifact
    }

    #[cfg(test)]
    pub(super) fn manifest_sha(&self) -> &str {
        &self.manifest_sha
    }
}

#[cfg(feature = "e2e")]
pub(super) fn from_environment() -> Option<ProofArtifact> {
    let profile = std::env::var_os("CL_GO_CEF_TEST_DATA_DIR")?;
    let manifest_sha = std::env::var("BEAVER_E2E_UI_MANIFEST_SHA").ok()?;
    ProofArtifact::new(
        PathBuf::from(profile).join("extensions-ui-proof"),
        "ui-proof",
        &manifest_sha,
    )
}

#[cfg(not(feature = "e2e"))]
pub(super) fn from_environment() -> Option<ProofArtifact> {
    None
}

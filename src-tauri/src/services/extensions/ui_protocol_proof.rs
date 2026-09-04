use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct ProofArtifact {
    root: PathBuf,
    pub(super) extension_id: String,
    pub(super) manifest_sha: String,
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
        })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    #[cfg(test)]
    pub(super) fn for_test(root: PathBuf, extension_id: &str, manifest_sha: &str) -> Self {
        Self::new(root, extension_id, manifest_sha).expect("valid UI protocol test artifact")
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

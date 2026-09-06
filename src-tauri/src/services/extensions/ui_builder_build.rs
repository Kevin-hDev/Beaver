use super::types::{ExtensionRecord, ExtensionUiArtifact};
use super::ui_builder::UiBuildRuntime;
use super::ui_builder_paths::{canonical_directory, canonical_source_file};
use std::path::Path;

pub(super) fn build(
    record: &ExtensionRecord,
    runtime: &UiBuildRuntime,
    token: Option<&str>,
    owner: Option<&super::install_jobs::InstallControl>,
    cancelled: impl Fn() -> bool,
) -> Result<ExtensionUiArtifact, super::OperationFailure> {
    let root = canonical_directory(Path::new(&record.source))?;
    let entry = record
        .manifest
        .ui
        .as_ref()
        .and_then(|ui| ui.entry.as_deref())
        .ok_or(super::OperationFailure::ManifestInvalid)?;
    let entry_path = canonical_source_file(&root.join(entry))?;
    if !entry_path.starts_with(&root) {
        return Err(super::OperationFailure::ManifestInvalid);
    }
    let relative = entry_path
        .strip_prefix(&root)
        .ok()
        .and_then(Path::to_str)
        .ok_or(super::OperationFailure::ManifestInvalid)?;
    let staging = match token {
        Some(token) => super::ui_artifact_store::prepare_owned(token),
        None => super::ui_artifact_store::prepare(),
    }
    .map_err(|_| super::OperationFailure::StorageFailed)?;
    let arguments = vec![
        runtime.builder.as_os_str().to_owned(),
        "--input-root".into(),
        root.as_os_str().to_owned(),
        "--output-root".into(),
        staging.output().as_os_str().to_owned(),
        "--entry".into(),
        relative.into(),
    ];
    let stdout = super::ui_builder_process::run(
        runtime,
        &arguments,
        staging.temporary(),
        cancelled,
        owner,
        || staging.reset_for_retry(),
    )?;
    let artifact: ExtensionUiArtifact =
        serde_json::from_slice(&stdout).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    super::ui_artifact::validate(&artifact)
        .map_err(|_| super::OperationFailure::ManifestInvalid)?;
    commit_artifact(staging, record, &artifact, owner)?;
    Ok(artifact)
}

pub(super) fn commit_artifact(
    staging: super::ui_artifact_store::StagingArtifact,
    record: &ExtensionRecord,
    artifact: &ExtensionUiArtifact,
    owner: Option<&super::install_jobs::InstallControl>,
) -> Result<(), super::OperationFailure> {
    if let Some(owner) = owner {
        let mut checkpoint = owner
            .saved()
            .map_err(|_| super::OperationFailure::StorageFailed)?
            .ok_or(super::OperationFailure::StorageFailed)?;
        let mut candidate = record.clone();
        candidate.ui_artifact = Some(artifact.clone());
        checkpoint.record = Some(candidate);
        // Record the content-addressed destination before rename: a crash must
        // leave the job enough evidence to remove its unpublished UI precisely.
        owner
            .save(checkpoint)
            .map_err(|_| super::OperationFailure::StorageFailed)?;
    }
    staging
        .commit(&record.manifest.id, artifact)
        .map_err(|_| super::OperationFailure::StorageFailed)?;
    Ok(())
}

use super::source_validation::{GitSource, NpmSource};
use super::types::{ExtensionOrigin, ExtensionOriginKind, ExtensionRecord};
use super::OperationFailure;
use std::path::Path;

use crate::services::work_registry::ServiceWorkCancellation;

pub struct PreparedInstall {
    pub record: ExtensionRecord,
}

pub fn git(
    source: GitSource,
    npm: super::npm_runner::NpmRunner,
    ui_runtime: super::ui_builder::UiBuildRuntime,
    cancellation: &ServiceWorkCancellation,
) -> Result<PreparedInstall, OperationFailure> {
    let staging = super::managed_store::prepare().map_err(|_| OperationFailure::StorageFailed)?;
    let staging_path = staging.path().to_path_buf();
    let materialized = super::git_source::materialize(&source, &staging_path, &npm, cancellation)?;
    record(
        staging,
        &materialized.root,
        ExtensionOrigin {
            kind: ExtensionOriginKind::Git,
            locator: source.locator,
            revision: Some(materialized.revision),
        },
        &ui_runtime,
        cancellation,
    )
}

pub fn npm(
    source: NpmSource,
    npm: super::npm_runner::NpmRunner,
    ui_runtime: super::ui_builder::UiBuildRuntime,
    cancellation: &ServiceWorkCancellation,
) -> Result<PreparedInstall, OperationFailure> {
    let staging = super::managed_store::prepare().map_err(|_| OperationFailure::StorageFailed)?;
    let staging_path = staging.path().to_path_buf();
    let package = super::npm_source::materialize(&source, &staging_path, &npm, cancellation)?;
    record(
        staging,
        &package,
        ExtensionOrigin {
            kind: ExtensionOriginKind::Npm,
            locator: source.locator,
            revision: None,
        },
        &ui_runtime,
        cancellation,
    )
}

fn record(
    staging: super::managed_store::StagingDirectory,
    source: &Path,
    origin: ExtensionOrigin,
    ui_runtime: &super::ui_builder::UiBuildRuntime,
    cancellation: &ServiceWorkCancellation,
) -> Result<PreparedInstall, OperationFailure> {
    let staging_path = staging.path().to_path_buf();
    let source_text = source.to_str().ok_or(OperationFailure::ManifestInvalid)?;
    let mut record = super::manifest::load_managed(source_text)?.record;
    record.origin = Some(origin);
    let installed = staging
        .commit(&record.manifest.id)
        .map_err(|_| OperationFailure::StorageFailed)?;
    super::managed_store::rewrite_source(&mut record, &staging_path, &installed)
        .map_err(|_| OperationFailure::StorageFailed)?;
    if let Err(error) =
        super::ui_builder::prepare_record(&mut record, ui_runtime, || cancellation.is_cancelled())
    {
        let _ = super::ui_artifact_store::unreferenced_from_registry();
        let _ = super::managed_store::remove_record(&record);
        return Err(error);
    }
    if super::validation::records(std::slice::from_ref(&record)).is_err() {
        let _ = super::ui_artifact_store::unreferenced_from_registry();
        let _ = super::managed_store::remove_record(&record);
        return Err(OperationFailure::ManifestInvalid);
    }
    Ok(PreparedInstall { record })
}

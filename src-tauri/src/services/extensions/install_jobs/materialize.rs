//! Source production happens only inside the admitted blocking job owner.
use super::super::{
    npm_runner::NpmRunner,
    types::{ExtensionOrigin, ExtensionOriginKind, ExtensionRecord},
    ui_builder::UiBuildRuntime,
};
use super::{
    checkpoint::InstallCheckpoint, InstallControl, InstallInterruption, InstallPhase,
    InstallRequest,
};

pub(super) fn prepare(
    request: &InstallRequest,
    control: &InstallControl,
    checkpoint: &mut InstallCheckpoint,
    npm: &NpmRunner,
    ui: &UiBuildRuntime,
) -> Result<ExtensionRecord, InstallInterruption> {
    if checkpoint.safe_phase == Some(InstallPhase::BuildingUi) {
        control.after_producer_stopped()?;
        return checkpoint
            .record
            .clone()
            .filter(|record| super::super::fingerprint::is_current(record).unwrap_or(false))
            .ok_or(InstallInterruption::Failed);
    }
    control.checkpoint(InstallPhase::Resolving)?;
    let resolved = resolve(request, checkpoint)?;
    checkpoint.resolved_source = Some(resolved.clone());
    // The staging identities are deterministic from this private random token and
    // journaled before creation. No Drop on this path may erase recovery evidence.
    checkpoint.producer_active = true;
    control.save(checkpoint.clone())?;
    let mut record = match resolved {
        InstallRequest::Local { path } => super::super::manifest::load_local(&path)
            .map(|loaded| loaded.record)
            .map_err(|_| InstallInterruption::Failed)?,
        InstallRequest::Git { locator } => {
            let source = super::super::source_validation::git(&locator)
                .map_err(|_| InstallInterruption::Failed)?;
            #[cfg(feature = "e2e")]
            let source =
                super::e2e_fixture::git_source(source).map_err(|_| InstallInterruption::Failed)?;
            let staging = super::super::managed_store::prepare_owned(&checkpoint.token)
                .map_err(|_| InstallInterruption::Failed)?;
            let materialized =
                super::super::git_source::materialize(&source, staging.path(), npm, control)
                    .map_err(|_| InstallInterruption::Failed)?;
            managed_record(
                staging,
                &materialized.root,
                ExtensionOrigin {
                    kind: ExtensionOriginKind::Git,
                    locator: source.locator,
                    revision: Some(materialized.revision),
                },
                checkpoint,
                control,
            )?
        }
        InstallRequest::Npm { locator } => {
            control.checkpoint(InstallPhase::Dependencies)?;
            let source = super::super::source_validation::npm(&locator)
                .map_err(|_| InstallInterruption::Failed)?;
            let staging = super::super::managed_store::prepare_owned(&checkpoint.token)
                .map_err(|_| InstallInterruption::Failed)?;
            let package =
                super::super::npm_source::materialize(&source, staging.path(), npm, control)
                    .map_err(|_| InstallInterruption::Failed)?;
            managed_record(
                staging,
                &package,
                ExtensionOrigin {
                    kind: ExtensionOriginKind::Npm,
                    locator: source.locator,
                    revision: None,
                },
                checkpoint,
                control,
            )?
        }
        InstallRequest::Update { .. } => return Err(InstallInterruption::Failed),
    };
    control.checkpoint(InstallPhase::Validating)?;
    if checkpoint
        .previous
        .as_ref()
        .is_some_and(|old| old.manifest.id != record.manifest.id)
    {
        return Err(InstallInterruption::Failed);
    }
    // A pre-existing local record is not evidence that this job published after a crash.
    if checkpoint.previous.is_none() && super::super::registry::find(&record.manifest.id).is_ok() {
        return Err(InstallInterruption::Failed);
    }
    checkpoint.record = Some(record.clone());
    control.save(checkpoint.clone())?;
    control.checkpoint(InstallPhase::BuildingUi)?;
    super::super::ui_builder::prepare_owned_record(
        &mut record,
        ui,
        &checkpoint.token,
        control,
        || control.is_cancelled(),
    )
    .map_err(|_| InstallInterruption::Failed)?;
    super::super::validation::records(std::slice::from_ref(&record))
        .map_err(|_| InstallInterruption::Failed)?;
    let occupied_bytes = if super::super::installer::is_managed(&record) {
        let root = super::super::managed_store::install_root(&record)
            .map_err(|_| InstallInterruption::Failed)?;
        super::super::managed_tree::measure_with_budget(&root, control.storage_budget())
            .map_err(|_| InstallInterruption::Failed)?
    } else {
        0
    };
    let occupied_bytes = occupied_bytes.saturating_add(
        record
            .ui_artifact
            .as_ref()
            .map_or(0, |artifact| artifact.total_bytes as u64),
    );
    control.progress(super::InstallProgress {
        phase: InstallPhase::BuildingUi,
        downloaded_bytes: None,
        download_total_bytes: None,
        occupied_bytes,
        free_bytes: super::disk_policy::free_bytes(&crate::services::paths::data_dir()).ok(),
    })?;
    checkpoint.producer_active = false;
    checkpoint.safe_phase = Some(InstallPhase::BuildingUi);
    checkpoint.record = Some(record.clone());
    control.save(checkpoint.clone())?;
    control.after_producer_stopped()?;
    Ok(record)
}

fn resolve(
    request: &InstallRequest,
    checkpoint: &mut InstallCheckpoint,
) -> Result<InstallRequest, InstallInterruption> {
    let InstallRequest::Update { extension_id } = request else {
        return Ok(request.clone());
    };
    let previous =
        super::super::registry::find(extension_id).map_err(|_| InstallInterruption::Failed)?;
    let origin = previous
        .origin
        .as_ref()
        .ok_or(InstallInterruption::Failed)?;
    let resolved = match origin.kind {
        ExtensionOriginKind::Git => InstallRequest::Git {
            locator: origin.locator.clone(),
        },
        ExtensionOriginKind::Npm => InstallRequest::Npm {
            locator: origin.locator.clone(),
        },
        ExtensionOriginKind::Local => return Err(InstallInterruption::Failed),
    };
    checkpoint.previous = Some(previous);
    Ok(resolved)
}

fn managed_record(
    staging: super::super::managed_store::StagingDirectory,
    source: &std::path::Path,
    origin: ExtensionOrigin,
    checkpoint: &mut InstallCheckpoint,
    control: &InstallControl,
) -> Result<ExtensionRecord, InstallInterruption> {
    control.checkpoint(InstallPhase::Validating)?;
    let mut record =
        super::super::manifest::load_managed(source.to_str().ok_or(InstallInterruption::Failed)?)
            .map_err(|_| InstallInterruption::Failed)?
            .record;
    record.origin = Some(origin);
    let staging_path = staging.path().to_path_buf();
    let destination = super::super::managed_store::root()
        .join(&record.manifest.id)
        .join(&checkpoint.token);
    super::super::managed_store::rewrite_source(&mut record, &staging_path, &destination)
        .map_err(|_| InstallInterruption::Failed)?;
    // Both sides of the rename can now be derived during recovery.
    checkpoint.record = Some(record.clone());
    control.save(checkpoint.clone())?;
    staging
        .commit(&record.manifest.id)
        .map_err(|_| InstallInterruption::Failed)?;
    Ok(record)
}

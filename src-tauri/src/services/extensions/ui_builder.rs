use std::path::{Path, PathBuf};

use super::types::{ExtensionRecord, ExtensionUiArtifact, ExtensionUiMode};

#[derive(Clone)]
pub(crate) struct UiBuildRuntime {
    pub(super) node: PathBuf,
    pub(super) builder: PathBuf,
    pub(super) directory: PathBuf,
}

impl UiBuildRuntime {
    pub(super) fn resolve(app: &tauri::AppHandle) -> Result<Self, super::OperationFailure> {
        let paths = super::host_paths::resolve(app)
            .map_err(|_| super::OperationFailure::RuntimeUnavailable)?;
        Ok(Self {
            node: canonical_file(&paths.node)?,
            builder: canonical_file(&paths.ui_builder())?,
            directory: canonical_directory(&paths.directory)?,
        })
    }
}

pub(crate) async fn refresh_all(app: &tauri::AppHandle) -> Result<Vec<String>, String> {
    let runtime = UiBuildRuntime::resolve(app).map_err(|error| error.code().to_string())?;
    let records = super::registry::list()?;
    let originals = records
        .into_iter()
        .filter(|record| record.kind == super::types::ExtensionKind::Local)
        .collect::<Vec<_>>();
    let cleanup_records = originals.clone();
    let prepared = tokio::task::spawn_blocking(move || prepare_reloads(originals, &runtime))
        .await
        .map_err(|_| super::error_codes::INSTALL_FAILED.to_string())?;
    let replacements = match prepared {
        Ok(replacements) => replacements,
        Err(error) => {
            let _ = super::ui_artifact_store::unreferenced(&cleanup_records);
            return Err(error);
        }
    };
    let revoked = replacements
        .iter()
        .filter(|(old, next)| old.trusted && !next.trusted)
        .map(|(old, _)| old.manifest.id.clone())
        .collect::<Vec<_>>();
    if let Err(error) = super::registry::replace_ui_records(replacements) {
        let _ = super::ui_artifact_store::unreferenced_from_registry();
        return Err(error);
    }
    let current = super::registry::list()?;
    super::ui_artifact_store::unreferenced(&current)?;
    for id in &revoked {
        crate::services::agent_local::permission_gate::clear_extension(id).await;
    }
    Ok(revoked)
}

fn prepare_reloads(
    originals: Vec<ExtensionRecord>,
    runtime: &UiBuildRuntime,
) -> Result<Vec<(ExtensionRecord, ExtensionRecord)>, String> {
    let mut replacements = Vec::with_capacity(originals.len());
    for original in originals {
        let mut candidate = reload_record(&original)?;
        prepare_record(&mut candidate, runtime, || false)
            .map_err(|error| error.code().to_string())?;
        let unchanged = super::fingerprint::same_encoded(
            original.fingerprint.as_deref(),
            candidate.fingerprint.as_deref(),
        );
        candidate.enabled = original.enabled && unchanged;
        candidate.trusted = original.trusted && unchanged;
        candidate.trusted_at = unchanged.then(|| original.trusted_at.clone()).flatten();
        candidate.show_in_chat = original.show_in_chat;
        candidate.last_activated_at = original.last_activated_at.clone();
        candidate.sensitive_access_granted = original.sensitive_access_granted;
        if unchanged {
            candidate.status = original.status.clone();
            candidate.last_error = original.last_error.clone();
        } else {
            candidate.status = super::types::ExtensionStatus::Error;
            candidate.last_error = Some(super::error_codes::FINGERPRINT_CHANGED.to_string());
        }
        replacements.push((original, candidate));
    }
    Ok(replacements)
}

fn reload_record(original: &ExtensionRecord) -> Result<ExtensionRecord, String> {
    let root = Path::new(&original.source);
    let mut candidate = if super::manifest_source::manifest_path(root).is_some() {
        super::manifest::load_local(&original.source)?.record
    } else {
        original.clone()
    };
    if candidate.manifest.id != original.manifest.id {
        return Err(super::error_codes::UPDATE_IDENTITY_CHANGED.to_string());
    }
    candidate.origin = original.origin.clone();
    candidate.contributions = super::types::ExtensionContributions::default();
    Ok(candidate)
}

pub(super) fn prepare_record(
    record: &mut ExtensionRecord,
    runtime: &UiBuildRuntime,
    cancelled: impl Fn() -> bool,
) -> Result<(), super::OperationFailure> {
    let advanced = record
        .manifest
        .ui
        .as_ref()
        .is_some_and(|ui| ui.mode == ExtensionUiMode::Advanced);
    if !advanced {
        record.ui_artifact = None;
        record.fingerprint = Some(
            super::fingerprint::calculate(record)
                .map_err(|_| super::OperationFailure::ManifestInvalid)?,
        );
        return Ok(());
    }
    let artifact = build(record, runtime, cancelled)?;
    record.ui_artifact = Some(artifact);
    record.fingerprint = Some(
        super::fingerprint::calculate(record)
            .map_err(|_| super::OperationFailure::ManifestInvalid)?,
    );
    Ok(())
}

fn build(
    record: &ExtensionRecord,
    runtime: &UiBuildRuntime,
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
    let staging =
        super::ui_artifact_store::prepare().map_err(|_| super::OperationFailure::StorageFailed)?;
    let arguments = vec![
        runtime.builder.as_os_str().to_owned(),
        "--input-root".into(),
        root.as_os_str().to_owned(),
        "--output-root".into(),
        staging.output().as_os_str().to_owned(),
        "--entry".into(),
        relative.into(),
    ];
    let stdout =
        super::ui_builder_process::run(runtime, &arguments, staging.temporary(), cancelled)?;
    let artifact: ExtensionUiArtifact =
        serde_json::from_slice(&stdout).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    super::ui_artifact::validate(&artifact)
        .map_err(|_| super::OperationFailure::ManifestInvalid)?;
    super::ui_artifact::verify_at(staging.output(), &artifact)
        .map_err(|_| super::OperationFailure::ManifestInvalid)?;
    staging
        .commit(&record.manifest.id, &artifact)
        .map_err(|_| super::OperationFailure::StorageFailed)?;
    Ok(artifact)
}

fn canonical_file(path: &Path) -> Result<PathBuf, super::OperationFailure> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| super::OperationFailure::RuntimeUnavailable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(super::OperationFailure::RuntimeUnavailable);
    }
    dunce::canonicalize(path).map_err(|_| super::OperationFailure::RuntimeUnavailable)
}

fn canonical_source_file(path: &Path) -> Result<PathBuf, super::OperationFailure> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(super::OperationFailure::ManifestInvalid);
    }
    dunce::canonicalize(path).map_err(|_| super::OperationFailure::ManifestInvalid)
}

fn canonical_directory(path: &Path) -> Result<PathBuf, super::OperationFailure> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(super::OperationFailure::ManifestInvalid);
    }
    dunce::canonicalize(path).map_err(|_| super::OperationFailure::ManifestInvalid)
}

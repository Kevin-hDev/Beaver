use super::source_validation::{GitSource, NpmSource};
use super::types::{ExtensionOrigin, ExtensionOriginKind, ExtensionRecord};
use std::path::Path;

struct PreparedInstall {
    record: ExtensionRecord,
}

pub async fn install_git(app: &tauri::AppHandle, locator: &str) -> Result<ExtensionRecord, String> {
    let source = super::source_validation::git(locator)?;
    let npm = super::npm_runner::NpmRunner::resolve(app)?;
    let prepared = blocking(move || prepare_git(source, npm)).await?;
    register_new(prepared)
}

pub async fn install_npm(app: &tauri::AppHandle, locator: &str) -> Result<ExtensionRecord, String> {
    let source = super::source_validation::npm(locator)?;
    let npm = super::npm_runner::NpmRunner::resolve(app)?;
    let prepared = blocking(move || prepare_npm(source, npm)).await?;
    register_new(prepared)
}

pub async fn update(app: &tauri::AppHandle, id: &str) -> Result<ExtensionRecord, String> {
    let current = super::registry::find(id)?;
    let origin = current
        .origin
        .clone()
        .filter(|origin| is_managed_kind(&origin.kind))
        .ok_or_else(|| "Cette extension n'est pas gérée par Beaver.".to_string())?;
    let npm = super::npm_runner::NpmRunner::resolve(app)?;
    let prepared = match origin.kind {
        ExtensionOriginKind::Git => {
            let source = super::source_validation::git(&origin.locator)?;
            blocking(move || prepare_git(source, npm)).await?
        }
        ExtensionOriginKind::Npm => {
            let source = super::source_validation::npm(&origin.locator)?;
            blocking(move || prepare_npm(source, npm)).await?
        }
        ExtensionOriginKind::Local => {
            return Err("Cette extension n'est pas gérée par Beaver.".to_string())
        }
    };
    if prepared.record.manifest.id != current.manifest.id {
        cleanup(&prepared.record).await;
        return Err("L'identité de l'extension mise à jour a changé.".to_string());
    }
    replace_current(current, prepared).await
}

pub async fn uninstall(id: &str) -> Result<(), String> {
    let current = super::registry::find(id)?;
    super::runtime::stop().await;
    if let Err(error) = super::registry::remove(id) {
        let _ = super::runtime::start_and_sync().await;
        return Err(error);
    }
    let cleanup_result = if is_managed(&current) {
        let record = current.clone();
        blocking(move || super::managed_store::remove_record(&record)).await
    } else {
        Ok(())
    };
    let _ = super::runtime::start_and_sync().await;
    cleanup_result
}

fn prepare_git(
    source: GitSource,
    npm: super::npm_runner::NpmRunner,
) -> Result<PreparedInstall, String> {
    let staging = super::managed_store::prepare()?;
    let staging_path = staging.path().to_path_buf();
    let materialized = super::git_source::materialize(&source, &staging_path, &npm)?;
    prepare_record(
        staging,
        &materialized.root,
        ExtensionOrigin {
            kind: ExtensionOriginKind::Git,
            locator: source.locator,
            revision: Some(materialized.revision),
        },
    )
}

fn prepare_npm(
    source: NpmSource,
    npm: super::npm_runner::NpmRunner,
) -> Result<PreparedInstall, String> {
    let staging = super::managed_store::prepare()?;
    let staging_path = staging.path().to_path_buf();
    let package = super::npm_source::materialize(&source, &staging_path, &npm)?;
    prepare_record(
        staging,
        &package,
        ExtensionOrigin {
            kind: ExtensionOriginKind::Npm,
            locator: source.locator,
            revision: None,
        },
    )
}

fn prepare_record(
    staging: super::managed_store::StagingDirectory,
    source: &Path,
    origin: ExtensionOrigin,
) -> Result<PreparedInstall, String> {
    let staging_path = staging.path().to_path_buf();
    let source_text = source
        .to_str()
        .ok_or_else(|| "Source d'extension invalide.".to_string())?;
    let mut record = super::manifest::load_local(source_text)?.record;
    record.origin = Some(origin);
    super::validation::records(std::slice::from_ref(&record))?;
    let installed = staging.commit(&record.manifest.id)?;
    super::managed_store::rewrite_source(&mut record, &staging_path, &installed)?;
    if let Err(error) = super::validation::records(std::slice::from_ref(&record)) {
        let _ = super::managed_store::remove_record(&record);
        return Err(error);
    }
    Ok(PreparedInstall { record })
}

fn register_new(prepared: PreparedInstall) -> Result<ExtensionRecord, String> {
    let record = prepared.record;
    if let Err(error) = super::registry::add_local(record.clone()) {
        let _ = super::managed_store::remove_record(&record);
        return Err(error);
    }
    Ok(record)
}

async fn replace_current(
    current: ExtensionRecord,
    prepared: PreparedInstall,
) -> Result<ExtensionRecord, String> {
    let replacement = super::installer_record::for_update(&current, prepared.record);
    super::runtime::stop().await;
    if let Err(error) = super::registry::replace_user(&current, replacement.clone()) {
        cleanup(&replacement).await;
        let _ = super::runtime::start_and_sync().await;
        return Err(error);
    }
    let old = current.clone();
    let _ = blocking(move || super::managed_store::remove_record(&old)).await;
    let _ = super::runtime::start_and_sync().await;
    Ok(replacement)
}

async fn cleanup(record: &ExtensionRecord) {
    let record = record.clone();
    let _ = blocking(move || super::managed_store::remove_record(&record)).await;
}

fn is_managed(record: &ExtensionRecord) -> bool {
    record
        .origin
        .as_ref()
        .is_some_and(|origin| is_managed_kind(&origin.kind))
}

fn is_managed_kind(kind: &ExtensionOriginKind) -> bool {
    matches!(kind, ExtensionOriginKind::Git | ExtensionOriginKind::Npm)
}

async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| "Installation d'extension interrompue.".to_string())?
}

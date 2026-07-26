use super::{family_has_other_installed_model, fs_safety, models_dir, sidecar_dir};
use crate::services::forecast::{catalog, sidecar_runtime, validation};
use std::path::Path;

pub async fn uninstall(model_id: &str) -> Result<(), String> {
    uninstall_from_roots(model_id, &models_dir(), &sidecar_dir()).await
}

pub(super) async fn uninstall_from_roots(
    model_id: &str,
    models: &Path,
    sidecar: &Path,
) -> Result<(), String> {
    validation::validate_model_id(model_id)?;
    let spec =
        catalog::find_model(model_id).ok_or_else(|| "Modèle Forecast inconnu".to_string())?;
    let staging = models.join(format!(".{model_id}.staging"));
    fs_safety::remove_path(&staging)
        .await
        .map_err(|_| "Suppression du modèle Forecast impossible".to_string())?;
    if !family_has_other_installed_model(models, spec.family_id, Some(model_id)) {
        remove_runtime(sidecar, spec.family_id).await?;
    }
    fs_safety::remove_path(&models.join(model_id))
        .await
        .map_err(|_| "Suppression du modèle Forecast impossible".to_string())
}

async fn remove_runtime(sidecar: &Path, family_id: &str) -> Result<(), String> {
    let directory = sidecar.to_path_buf();
    let family = family_id.to_string();
    tokio::task::spawn_blocking(move || sidecar_runtime::remove_family_runtime(&directory, &family))
        .await
        .map_err(|_| "Suppression du runtime Forecast impossible".to_string())?
}

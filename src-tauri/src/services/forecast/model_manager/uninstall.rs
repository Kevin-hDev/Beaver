use super::{family_has_other_installed_model, fs_safety, models_dir, sidecar_dir};
use crate::services::forecast::{catalog, sidecar_runtime, validation};
use std::path::Path;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UninstallBoundary {
    Staging,
    Model,
    Runtime,
}

pub async fn uninstall(model_id: &str) -> Result<(), String> {
    uninstall_from_roots(model_id, &models_dir(), &sidecar_dir()).await
}

pub(super) async fn uninstall_from_roots(
    model_id: &str,
    models: &Path,
    sidecar: &Path,
) -> Result<(), String> {
    let transaction = UninstallTransaction::prepare(model_id, models, sidecar)?;
    transaction.remove_staging().await?;
    transaction.remove_model().await?;
    transaction.remove_unused_runtime().await
}

struct UninstallTransaction<'a> {
    model_id: &'a str,
    models: &'a Path,
    sidecar: &'a Path,
    family_id: &'static str,
}

impl<'a> UninstallTransaction<'a> {
    fn prepare(model_id: &'a str, models: &'a Path, sidecar: &'a Path) -> Result<Self, String> {
        validation::validate_model_id(model_id)?;
        let spec =
            catalog::find_model(model_id).ok_or_else(|| "Modèle Forecast inconnu".to_string())?;
        Ok(Self {
            model_id,
            models,
            sidecar,
            family_id: spec.family_id,
        })
    }

    async fn remove_staging(&self) -> Result<(), String> {
        fs_safety::remove_path(&self.models.join(format!(".{}.staging", self.model_id)))
            .await
            .map_err(|_| "Suppression du modèle Forecast impossible".to_string())
    }

    async fn remove_model(&self) -> Result<(), String> {
        fs_safety::remove_path(&self.models.join(self.model_id))
            .await
            .map_err(|_| "Suppression du modèle Forecast impossible".to_string())
    }

    async fn remove_unused_runtime(&self) -> Result<(), String> {
        if !family_has_other_installed_model(self.models, self.family_id, None) {
            remove_runtime(self.sidecar, self.family_id).await?;
        }
        Ok(())
    }
}

async fn remove_runtime(sidecar: &Path, family_id: &str) -> Result<(), String> {
    let directory = sidecar.to_path_buf();
    let family = family_id.to_string();
    tokio::task::spawn_blocking(move || sidecar_runtime::remove_family_runtime(&directory, &family))
        .await
        .map_err(|_| "Suppression du runtime Forecast impossible".to_string())?
}

#[cfg(test)]
fn fail_if_requested(
    requested: UninstallBoundary,
    reached: UninstallBoundary,
) -> Result<(), String> {
    if requested == reached {
        Err("Échec de désinstallation Forecast injecté".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
pub(super) async fn uninstall_with_failure_after(
    model_id: &str,
    models: &Path,
    sidecar: &Path,
    boundary: UninstallBoundary,
) -> Result<(), String> {
    let transaction = UninstallTransaction::prepare(model_id, models, sidecar)?;
    transaction.remove_staging().await?;
    fail_if_requested(boundary, UninstallBoundary::Staging)?;
    transaction.remove_model().await?;
    fail_if_requested(boundary, UninstallBoundary::Model)?;
    transaction.remove_unused_runtime().await?;
    fail_if_requested(boundary, UninstallBoundary::Runtime)
}

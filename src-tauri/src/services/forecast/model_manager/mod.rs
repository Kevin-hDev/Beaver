use crate::services::forecast::validation;
use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};

pub mod download;
mod fs_safety;
mod install;
mod install_plan;
mod model_artifacts;
mod model_receipt;
mod runtime_install;
mod smoke;
mod smoke_proof;
mod uninstall;

pub use install::install_with_callback;
pub use uninstall::uninstall;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelReadiness {
    NotInstalled,
    UpdateRequired,
    Invalid,
    Ready,
}

impl ModelReadiness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotInstalled => "not_installed",
            Self::UpdateRequired => "update_required",
            Self::Invalid => "invalid",
            Self::Ready => "ready",
        }
    }
}

fn models_dir() -> PathBuf {
    data_dir().join("forecast-models")
}

fn sidecar_dir() -> PathBuf {
    data_dir().join("forecast-sidecar")
}

pub fn model_path(model_id: &str) -> PathBuf {
    models_dir().join(model_id)
}

pub fn is_installed(model_id: &str) -> bool {
    validation::validate_model_id(model_id).is_ok() && is_installed_in(&models_dir(), model_id)
}

pub fn is_ready(model_id: &str) -> bool {
    readiness(model_id) == ModelReadiness::Ready
}

pub fn readiness(model_id: &str) -> ModelReadiness {
    let Some(spec) = crate::services::forecast::catalog::find_model(model_id) else {
        return ModelReadiness::Invalid;
    };
    let model_dir = model_path(model_id);
    resolve_readiness(
        is_installed(model_id),
        model_receipt::has_expected_sizes(&model_dir, model_id),
        model_receipt::is_current(&model_dir, model_id),
        crate::services::forecast::sidecar_runtime::family_runtime_ready(
            &sidecar_dir(),
            spec.family_id,
        ),
        smoke::is_validated(&model_dir, &sidecar_dir(), spec.family_id),
    )
}

fn resolve_readiness(
    installed: bool,
    artifacts_valid: bool,
    receipt_current: bool,
    runtime_current: bool,
    smoke_current: bool,
) -> ModelReadiness {
    if !installed {
        return ModelReadiness::NotInstalled;
    }
    if !artifacts_valid {
        return ModelReadiness::Invalid;
    }
    if !receipt_current || !runtime_current || !smoke_current {
        return ModelReadiness::UpdateRequired;
    }
    ModelReadiness::Ready
}

pub fn get_model_size(model_id: &str) -> u64 {
    if validation::validate_model_id(model_id).is_err() {
        return 0;
    }
    fs_safety::bounded_directory_size(&model_path(model_id)).unwrap_or(0)
}

fn is_installed_in(root: &Path, model_id: &str) -> bool {
    has_downloaded_weights(root, model_id)
}

fn has_downloaded_weights(root: &Path, model_id: &str) -> bool {
    let model = root.join(model_id);
    fs_safety::is_real_directory(&model) && fs_safety::is_regular_file(&model.join(".complete"))
}

fn family_has_installed_model(models: &Path, family_id: &str) -> bool {
    family_has_other_installed_model(models, family_id, None)
}

fn family_has_other_installed_model(
    models: &Path,
    family_id: &str,
    excluded_model_id: Option<&str>,
) -> bool {
    let Ok(runtime) = crate::services::forecast::sidecar_runtime::runtime_id(family_id) else {
        return false;
    };
    crate::services::forecast::catalog::FORECAST_MODELS
        .iter()
        .filter(|model| {
            Some(model.id) != excluded_model_id
                && !model.is_cloud
                && crate::services::forecast::sidecar_runtime::runtime_id(model.family_id)
                    .is_ok_and(|candidate| candidate == runtime)
        })
        .any(|model| is_installed_in(models, model.id))
}

#[cfg(test)]
mod tests;

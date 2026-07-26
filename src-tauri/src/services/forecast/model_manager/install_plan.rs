use super::{family_has_installed_model, has_downloaded_weights, model_receipt, smoke};
use crate::services::forecast::{catalog, sidecar_runtime};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InstallPlan {
    Full,
    RuntimeOnly,
    Validate,
    Ready,
}

pub(super) fn should_remove_prepared_runtime(
    runtime_was_ready: bool,
    models: &Path,
    family_id: &str,
) -> bool {
    !runtime_was_ready && !family_has_installed_model(models, family_id)
}

pub(super) fn install_plan(
    models: &Path,
    sidecar: &Path,
    model_id: &str,
) -> Result<InstallPlan, String> {
    let spec =
        catalog::find_model(model_id).ok_or_else(|| "Modèle Forecast inconnu".to_string())?;
    let model_dir = models.join(model_id);
    let weights = has_downloaded_weights(models, model_id)
        && model_receipt::has_expected_sizes(&model_dir, model_id);
    let runtime = sidecar_runtime::family_runtime_ready(sidecar, spec.family_id);
    let smoke = model_receipt::is_current(&model_dir, model_id)
        && smoke::is_validated(&model_dir, sidecar, spec.family_id);
    Ok(plan_for_state(weights, runtime, smoke))
}

pub(super) fn plan_for_state(weights: bool, runtime: bool, smoke: bool) -> InstallPlan {
    match (weights, runtime, smoke) {
        (true, true, true) => InstallPlan::Ready,
        (true, true, false) => InstallPlan::Validate,
        (true, false, _) => InstallPlan::RuntimeOnly,
        (false, _, _) => InstallPlan::Full,
    }
}

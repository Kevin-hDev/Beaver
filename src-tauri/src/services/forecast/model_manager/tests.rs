use super::install_plan::{
    install_plan, plan_for_state, should_remove_prepared_runtime, InstallPlan,
};
use super::is_installed_in;
use super::uninstall::uninstall_from_roots;
use super::{resolve_readiness, ModelReadiness};
use std::fs;

fn mark_installed(models: &std::path::Path, model_id: &str) {
    let model = models.join(model_id);
    fs::create_dir_all(&model).unwrap();
    fs::write(model.join(".complete"), "ok").unwrap();
    fs::write(model.join(".smoke-v1"), "ok").unwrap();
}

fn mark_downloaded_artifacts(models: &std::path::Path, model_id: &str) {
    mark_installed(models, model_id);
    let model_dir = models.join(model_id);
    for artifact in super::model_artifacts::model(model_id)
        .unwrap()
        .artifacts
        .iter()
    {
        let path = model_dir.join(&artifact.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::File::create(path)
            .unwrap()
            .set_len(artifact.size)
            .unwrap();
    }
}

#[test]
fn ready_plan_requires_a_model_smoke_proof() {
    assert_eq!(plan_for_state(true, true, false), InstallPlan::Validate);
    assert_eq!(plan_for_state(true, true, true), InstallPlan::Ready);
    assert_eq!(plan_for_state(true, false, true), InstallPlan::RuntimeOnly);
}

#[test]
fn readiness_keeps_legacy_models_visible_as_update_required() {
    assert_eq!(
        resolve_readiness(true, true, false, false, false),
        ModelReadiness::UpdateRequired
    );
    assert_eq!(
        resolve_readiness(true, false, false, false, false),
        ModelReadiness::Invalid
    );
    assert_eq!(
        resolve_readiness(true, true, true, true, true),
        ModelReadiness::Ready
    );
}

#[test]
fn downloaded_legacy_model_remains_installed_before_smoke_validation() {
    let temp = tempfile::tempdir().unwrap();
    let model = temp.path().join("chronos-bolt-tiny");
    fs::create_dir_all(&model).unwrap();
    fs::write(model.join(".complete"), "ok").unwrap();

    assert!(is_installed_in(temp.path(), "chronos-bolt-tiny"));
}

#[test]
fn installed_weights_only_require_the_missing_runtime() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");
    let sidecar = temp.path().join("sidecar");
    fs::create_dir_all(&sidecar).unwrap();
    fs::write(sidecar.join("requirements.txt"), "numpy\n").unwrap();
    mark_downloaded_artifacts(&models, "chronos-bolt-tiny");

    assert_eq!(
        install_plan(&models, &sidecar, "chronos-bolt-tiny").unwrap(),
        InstallPlan::RuntimeOnly
    );
}

#[test]
fn incomplete_weights_require_a_full_reinstall() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");
    let sidecar = temp.path().join("sidecar");
    mark_installed(&models, "chronos-bolt-tiny");

    assert_eq!(
        install_plan(&models, &sidecar, "chronos-bolt-tiny").unwrap(),
        InstallPlan::Full
    );
}

#[test]
fn orphan_cleanup_preserves_preexisting_and_shared_runtimes() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");

    assert!(should_remove_prepared_runtime(
        false,
        &models,
        "chronos-bolt"
    ));
    assert!(!should_remove_prepared_runtime(
        true,
        &models,
        "chronos-bolt"
    ));

    mark_installed(&models, "chronos-bolt-mini");
    assert!(!should_remove_prepared_runtime(
        false,
        &models,
        "chronos-bolt"
    ));
}

#[tokio::test]
async fn shared_runtime_is_removed_only_after_the_last_model() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");
    let sidecar = temp.path().join("sidecar");
    let runtime = sidecar.join(".venvs").join("chronos");
    mark_installed(&models, "chronos-bolt-tiny");
    mark_installed(&models, "chronos-bolt-mini");
    fs::create_dir_all(&runtime).unwrap();

    uninstall_from_roots("chronos-bolt-tiny", &models, &sidecar)
        .await
        .unwrap();
    assert!(runtime.exists());

    uninstall_from_roots("chronos-bolt-mini", &models, &sidecar)
        .await
        .unwrap();
    assert!(!runtime.exists());
}

#[tokio::test]
async fn runtime_cleanup_failure_preserves_the_last_model() {
    let temp = tempfile::tempdir().unwrap();
    let models = temp.path().join("models");
    let sidecar = temp.path().join("sidecar");
    let runtime = sidecar.join(".venvs").join("chronos");
    mark_installed(&models, "chronos-bolt-tiny");
    fs::create_dir_all(runtime.parent().unwrap()).unwrap();
    fs::write(&runtime, "invalid runtime").unwrap();

    assert!(uninstall_from_roots("chronos-bolt-tiny", &models, &sidecar)
        .await
        .is_err());
    assert!(is_installed_in(&models, "chronos-bolt-tiny"));
}

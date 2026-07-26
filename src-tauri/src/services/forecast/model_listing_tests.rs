use super::{readiness_state, runnable_state};
use crate::services::forecast::model_manager::ModelReadiness;
use crate::services::forecast::{catalog, registry};

#[test]
fn local_model_is_not_runnable_until_its_runtime_is_ready() {
    let model = catalog::find_model("chronos-bolt-tiny").unwrap();
    let runtime = registry::find_runtime(model.id);

    assert!(!runnable_state(model, runtime, true, false, false));
    assert!(runnable_state(model, runtime, true, false, true));
}

#[test]
fn cloud_model_still_depends_on_provider_configuration() {
    let model = catalog::find_model("timegpt-2-mini").unwrap();
    let runtime = registry::find_runtime(model.id);

    assert!(!runnable_state(model, runtime, false, false, false));
    assert!(runnable_state(model, runtime, false, true, true));
}

#[test]
fn readiness_state_distinguishes_updates_from_missing_cloud_credentials() {
    let local = catalog::find_model("chronos-bolt-tiny").unwrap();
    let local_runtime = registry::find_runtime(local.id);
    assert_eq!(
        readiness_state(local, local_runtime, false, ModelReadiness::UpdateRequired),
        "update_required"
    );

    let cloud = catalog::find_model("timegpt-2-mini").unwrap();
    let cloud_runtime = registry::find_runtime(cloud.id);
    assert_eq!(
        readiness_state(cloud, cloud_runtime, false, ModelReadiness::NotInstalled),
        "provider_required"
    );
    assert_eq!(
        readiness_state(cloud, cloud_runtime, true, ModelReadiness::NotInstalled),
        "ready"
    );
}

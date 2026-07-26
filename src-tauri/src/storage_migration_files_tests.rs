use super::{copy_items, resolve_forecast_resource_base, sync_forecast_sidecar_from};

fn write_required_assets(root: &std::path::Path) {
    std::fs::create_dir_all(root.join("forecast_runtime")).unwrap();
    std::fs::write(root.join("server.py"), "server").unwrap();
    std::fs::write(root.join("test_model_smoke.py"), "smoke").unwrap();
    std::fs::write(root.join("requirements.txt"), "runtime").unwrap();
    std::fs::write(root.join("forecast_runtime/adapters.py"), "adapters").unwrap();
}

#[test]
fn forecast_resources_fall_back_to_manifest_dir_for_cef_dev() {
    let temp = tempfile::tempdir().unwrap();
    let empty_bundle = temp.path().join("bundle-resources");
    let manifest = temp.path().join("manifest");
    let source = manifest.join("resources/forecast-sidecar");
    std::fs::create_dir_all(&empty_bundle).unwrap();
    write_required_assets(&source);

    let resolved = resolve_forecast_resource_base(&empty_bundle, &manifest).unwrap();

    assert_eq!(resolved, source);
}

#[test]
fn forecast_sync_adds_new_assets_and_preserves_existing_runtimes() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    write_required_assets(&source);
    std::fs::create_dir_all(source.join("runtime-locks")).unwrap();
    std::fs::write(source.join("runtime-locks/internal.lock"), "private").unwrap();
    std::fs::write(source.join("test_contracts.py"), "dev only").unwrap();
    std::fs::create_dir_all(target.join(".venvs/chronos-bolt")).unwrap();
    std::fs::write(target.join(".venvs/chronos-bolt/keep"), "runtime").unwrap();

    sync_forecast_sidecar_from(&source, &target).unwrap();

    assert!(target.join("test_model_smoke.py").is_file());
    assert!(target.join(".venvs/chronos-bolt/keep").is_file());
    assert!(!target.join("runtime-locks").exists());
    assert!(!target.join("test_contracts.py").exists());
}

#[test]
fn forecast_sync_rejects_an_incomplete_source_without_deleting_target() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    std::fs::create_dir_all(source.join("forecast_runtime")).unwrap();
    std::fs::write(source.join("forecast_runtime/adapters.py"), "adapters").unwrap();
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("keep"), "runtime").unwrap();

    assert!(sync_forecast_sidecar_from(&source, &target).is_err());
    assert!(target.join("keep").is_file());
}

#[test]
fn legacy_copy_preserves_existing_files_and_copies_missing_items() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    std::fs::create_dir_all(source.join("agent-sessions")).unwrap();
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(source.join("config.json"), "legacy").unwrap();
    std::fs::write(source.join("agent-sessions/one.json"), "session").unwrap();
    std::fs::write(target.join("config.json"), "current").unwrap();

    copy_items(&source, &target).unwrap();

    assert_eq!(
        std::fs::read_to_string(target.join("config.json")).unwrap(),
        "current"
    );
    assert_eq!(
        std::fs::read_to_string(target.join("agent-sessions/one.json")).unwrap(),
        "session"
    );
}

#[cfg(unix)]
#[test]
fn legacy_copy_fails_closed_on_a_symbolic_link() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(&target).unwrap();
    symlink(outside.path(), source.join("agent-sessions")).unwrap();

    assert!(copy_items(&source, &target).is_err());
    assert!(!target.join("agent-sessions").exists());
}

#[cfg(unix)]
#[test]
fn failed_legacy_copy_removes_every_partial_item() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let sessions = source.join("agent-sessions");
    let target = temp.path().join("target");
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(sessions.join("valid.json"), "session").unwrap();
    symlink(outside.path(), sessions.join("external")).unwrap();

    assert!(copy_items(&source, &target).is_err());
    assert!(!target.join("agent-sessions").exists());
    assert!(!target.join(".migration-agent-sessions.tmp").exists());
    assert!(copy_items(&source, &target).is_err());
    assert!(!target.join("agent-sessions").exists());
}

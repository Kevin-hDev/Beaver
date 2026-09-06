use super::types::{ExtensionOrigin, ExtensionOriginKind};
use std::sync::Mutex;

static MANAGED_STORE_TESTS: Mutex<()> = Mutex::new(());

fn staged_record(
    id: &str,
) -> (
    super::managed_store::StagingDirectory,
    super::types::ExtensionRecord,
) {
    let staging = super::managed_store::prepare().unwrap();
    std::fs::write(staging.path().join("index.ts"), "export default () => {}").unwrap();
    std::fs::write(
        staging.path().join("beaver-extension.json"),
        serde_json::json!({
            "id": id,
            "name": "Managed test",
            "version": "1.0.0",
            "beaverApi": "1",
            "runtime": "node",
            "main": "index.ts",
            "access": "full"
        })
        .to_string(),
    )
    .unwrap();
    let source = staging.path().to_str().unwrap();
    let mut record = super::manifest::load_local(source).unwrap().record;
    record.origin = Some(ExtensionOrigin {
        kind: ExtensionOriginKind::Git,
        locator: "https://github.com/example/extension.git".to_string(),
        revision: Some("a".repeat(40)),
    });
    (staging, record)
}

#[test]
fn managed_install_is_versioned_and_removed_without_touching_other_sources() {
    let _guard = MANAGED_STORE_TESTS.lock().unwrap();
    let id = format!(
        "test.managed.{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    );
    let (staging, mut record) = staged_record(&id);
    let staging_path = staging.path().to_path_buf();
    let installed = staging.commit(&id).unwrap();
    super::managed_store::rewrite_source(&mut record, &staging_path, &installed).unwrap();

    assert!(installed.is_dir());
    assert!(super::validation::records(std::slice::from_ref(&record)).is_ok());
    super::managed_store::remove_record(&record).unwrap();
    assert!(!installed.exists());

    let local = tempfile::tempdir().unwrap();
    let entry = local.path().join("local.ts");
    std::fs::write(&entry, "export default () => {}").unwrap();
    let local_record = super::manifest::load_local(entry.to_str().unwrap())
        .unwrap()
        .record;
    assert!(super::managed_store::remove_record(&local_record).is_err());
    assert!(entry.is_file());
}

#[test]
fn successive_managed_installs_never_overwrite_each_other() {
    let _guard = MANAGED_STORE_TESTS.lock().unwrap();
    let id = format!(
        "test.versions.{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    );
    let (first_stage, mut first) = staged_record(&id);
    let first_path = first_stage.path().to_path_buf();
    let first_install = first_stage.commit(&id).unwrap();
    super::managed_store::rewrite_source(&mut first, &first_path, &first_install).unwrap();

    let (second_stage, mut second) = staged_record(&id);
    let second_path = second_stage.path().to_path_buf();
    let second_install = second_stage.commit(&id).unwrap();
    super::managed_store::rewrite_source(&mut second, &second_path, &second_install).unwrap();

    assert_ne!(first_install, second_install);
    assert!(first_install.is_dir());
    assert!(second_install.is_dir());
    // This shared store may contain producers from other tests. Exercise global
    // orphan collection in managed_cleanup's isolated directory fixtures instead.
    super::managed_store::remove_record(&first).unwrap();
    assert!(!first_install.exists());
    assert!(second_install.is_dir());
    super::managed_store::remove_record(&second).unwrap();
}

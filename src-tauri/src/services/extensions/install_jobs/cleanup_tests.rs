use super::super::types::{ExtensionOrigin, ExtensionOriginKind, ExtensionRecord};
use super::checkpoint::InstallCheckpoint;

fn prepared(id: &str, version: &str) -> (ExtensionRecord, String) {
    let staging = super::super::managed_store::prepare().unwrap();
    std::fs::write(
        staging.path().join("index.mjs"),
        format!("export default {{version:'{version}'}};"),
    )
    .unwrap();
    std::fs::write(staging.path().join("beaver-extension.json"), serde_json::to_vec(&serde_json::json!({
        "id":id,"name":"Update fixture","version":version,"beaverApi":"1","runtime":"node","main":"index.mjs","access":"full"
    })).unwrap()).unwrap();
    let mut record = super::super::manifest::load_local(staging.path().to_str().unwrap())
        .unwrap()
        .record;
    record.origin = Some(ExtensionOrigin {
        kind: ExtensionOriginKind::Git,
        locator: "https://example.invalid/fixture.git".into(),
        revision: Some("ab".repeat(20)),
    });
    let original = staging.path().to_owned();
    let installed = staging.commit(id).unwrap();
    super::super::managed_store::rewrite_source(&mut record, &original, &installed).unwrap();
    let token = installed.file_name().unwrap().to_str().unwrap().to_owned();
    (record, token)
}

#[test]
fn cancellation_preserves_old_version_and_success_retires_it_only_after_publication() {
    let id = format!("test-{}", uuid::Uuid::new_v4().simple());
    let (old, _) = prepared(&id, "1.0.0");
    super::super::registry_managed::add(old.clone()).unwrap();
    let (cancelled, token) = prepared(&id, "2.0.0");
    let checkpoint = InstallCheckpoint {
        version: 1,
        token,
        record: Some(cancelled.clone()),
        previous: Some(old.clone()),
        ..Default::default()
    };
    super::cleanup::run(&checkpoint).unwrap();
    assert!(std::path::Path::new(&old.source).exists());
    assert!(!std::path::Path::new(&cancelled.source).exists());
    assert_eq!(
        super::super::registry::find(&id).unwrap().manifest.version,
        "1.0.0"
    );

    let (next, token) = prepared(&id, "3.0.0");
    let next = super::super::installer_record::for_update(&old, next);
    super::super::registry::replace_user(&old, next.clone()).unwrap();
    let checkpoint = InstallCheckpoint {
        version: 1,
        token,
        record: Some(next.clone()),
        previous: Some(old.clone()),
        ..Default::default()
    };
    super::cleanup::run(&checkpoint).unwrap();
    assert!(!std::path::Path::new(&old.source).exists());
    assert!(std::path::Path::new(&next.source).exists());
    assert_eq!(
        super::super::registry::find(&id).unwrap().manifest.version,
        "3.0.0"
    );
    super::super::registry::remove(&id).unwrap();
    super::super::managed_store::remove_record(&next).unwrap();
}

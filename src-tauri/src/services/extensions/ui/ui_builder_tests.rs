use std::path::PathBuf;

#[test]
fn advanced_install_commits_then_verifies_the_complete_artifact() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root")
        .to_path_buf();
    let fixture = repository.join("scripts/extensions/fixtures/ui/advanced-valid");
    let mut record = super::manifest::load_local(fixture.to_str().expect("fixture path"))
        .expect("load advanced fixture")
        .record;
    let runtime = super::ui_builder::UiBuildRuntime {
        node: which::which(if cfg!(windows) { "node.exe" } else { "node" })
            .expect("Node.js for the extension UI builder"),
        builder: repository.join("scripts/extensions/ui-build.mjs"),
        directory: repository,
    };

    super::ui_builder::prepare_record(&mut record, &runtime, || false)
        .expect("build and commit advanced artifact");

    assert!(record.ui_artifact.is_some());
    super::ui_artifact::validate_record(&record).expect("verify committed artifact");
    super::ui_artifact_store::remove(&record).expect("remove test artifact");
}

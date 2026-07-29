use super::OperationFailure;

fn write_entry(root: &std::path::Path) {
    std::fs::write(root.join("index.js"), "export default () => {}").unwrap();
}

#[test]
fn npm_package_without_beaver_configuration_has_a_specific_failure() {
    let directory = tempfile::tempdir().unwrap();
    write_entry(directory.path());
    std::fs::write(
        directory.path().join("package.json"),
        r#"{"name":"ordinary-package","version":"1.0.0","main":"index.js"}"#,
    )
    .unwrap();

    let result = super::manifest::load_managed(directory.path().to_str().unwrap());

    assert!(matches!(result, Err(OperationFailure::NotBeaverExtension)));
}

#[test]
fn incompatible_beaver_api_has_a_specific_failure() {
    let directory = tempfile::tempdir().unwrap();
    write_entry(directory.path());
    std::fs::write(
        directory.path().join("beaver-extension.json"),
        r#"{
            "id":"test.incompatible",
            "name":"Incompatible",
            "version":"1.0.0",
            "beaverApi":"999",
            "runtime":"node",
            "main":"index.js",
            "access":"full"
        }"#,
    )
    .unwrap();

    let result = super::manifest::load_managed(directory.path().to_str().unwrap());

    assert!(matches!(result, Err(OperationFailure::ApiIncompatible)));
}

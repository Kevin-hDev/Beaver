use super::model_downloads_types::typescript_bindings;

#[test]
fn checked_in_model_download_types_match_rust() {
    let checked_in =
        include_str!("../../../src/types/model-download.generated.ts").replace("\r\n", "\n");
    assert_eq!(checked_in, typescript_bindings());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_model_download_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/model-download.generated.ts");
    std::fs::write(path, typescript_bindings()).expect("write model download contract");
}

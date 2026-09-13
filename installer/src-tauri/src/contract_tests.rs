use super::contract::typescript_bindings;

#[test]
fn checked_in_installer_types_match_rust() {
    let checked_in =
        include_str!("../../src/installer-contract.generated.ts").replace("\r\n", "\n");
    assert_eq!(checked_in, typescript_bindings());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_installer_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/installer-contract.generated.ts");
    std::fs::write(path, typescript_bindings()).unwrap();
}

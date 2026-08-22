use super::types_diagnostics::typescript_bindings;

#[test]
fn checked_in_typescript_matches_the_rust_diagnostics_contract() {
    let checked_in = include_str!("../../../../src/types/agent-diagnostics.ts").replace("\r\n", "\n");

    assert_eq!(checked_in, typescript_bindings());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_diagnostics_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/agent-diagnostics.ts");

    std::fs::write(path, typescript_bindings()).unwrap();
}

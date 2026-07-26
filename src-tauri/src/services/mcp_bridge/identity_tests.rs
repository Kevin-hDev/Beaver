use super::identity::{client_info, registration_name};

#[test]
fn mcp_client_info_uses_beaver_without_legacy_names() {
    let info = client_info();

    assert_eq!(info["name"], "Beaver");
    assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));
    assert!(!info.to_string().contains(&["CL", "GO"].join("-")));
    assert!(!info.to_string().contains(&["cl", "go", "dash"].join("-")));
}

#[test]
fn dynamic_registration_uses_beaver() {
    assert_eq!(registration_name("connector"), "Beaver (connector)");
}

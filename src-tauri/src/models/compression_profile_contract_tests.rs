use serde_json::json;

use super::compression_profile_contract::{typescript_bindings, CompressionProfileInput};

#[test]
fn input_rejects_frontend_capabilities_and_unknown_fields() {
    let mut value =
        serde_json::to_value(crate::services::compress::profile_defaults::beaver_profile())
            .expect("profile json");
    value
        .as_object_mut()
        .expect("profile object")
        .insert("capabilities".into(), json!(["tools"]));
    assert!(serde_json::from_value::<CompressionProfileInput>(value).is_err());
}

#[test]
fn checked_in_compression_profile_types_match_rust() {
    let checked_in =
        include_str!("../../../src/types/compression-profile.generated.ts").replace("\r\n", "\n");
    assert_eq!(checked_in, typescript_bindings());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_compression_profile_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/compression-profile.generated.ts");
    std::fs::write(path, typescript_bindings()).expect("write compression profile contract");
}

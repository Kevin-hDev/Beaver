use super::*;

#[test]
fn platform_extensions_stay_stable() {
    assert_eq!(temp_extension(UpdatePlatform::Macos), "dmg");
    assert_eq!(temp_extension(UpdatePlatform::Windows), "exe");
    assert_eq!(temp_extension(UpdatePlatform::Linux), "deb");
}

#[test]
fn names_include_platform_and_architecture() {
    assert_eq!(
        expected_asset_name(
            &super::super::app_update_source::UPDATE_SOURCE,
            "1.1.0",
            UpdatePlatform::Linux,
            UpdateArchitecture::X86_64,
        )
        .as_deref(),
        Some("Beaver_1.1.0_amd64.deb")
    );
}

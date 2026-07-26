use super::*;
use crate::commands::app_update_assets::{expected_asset_name, UpdateArchitecture, UpdatePlatform};
use crate::commands::app_update_source::UPDATE_SOURCE;

const NEXT_VERSION: &str = "1.1.1";

fn asset_url(version: &str, platform: UpdatePlatform, architecture: UpdateArchitecture) -> String {
    let name = expected_asset_name(&UPDATE_SOURCE, version, platform, architecture).unwrap();
    format!("https://github.com/Kevin-hDev/Beaver/releases/download/v{version}/{name}")
}

#[test]
fn accepts_exact_assets_for_every_supported_target() {
    for (platform, architecture) in [
        (UpdatePlatform::Macos, UpdateArchitecture::Aarch64),
        (UpdatePlatform::Macos, UpdateArchitecture::X86_64),
        (UpdatePlatform::Windows, UpdateArchitecture::Aarch64),
        (UpdatePlatform::Windows, UpdateArchitecture::X86_64),
        (UpdatePlatform::Linux, UpdateArchitecture::Aarch64),
        (UpdatePlatform::Linux, UpdateArchitecture::X86_64),
    ] {
        let raw = asset_url(NEXT_VERSION, platform, architecture);
        assert!(validate_update_url_for(&raw, platform, architecture).is_ok());
    }
}

#[test]
fn rejects_old_repository_wrong_target_and_mismatched_version() {
    let valid = asset_url(
        NEXT_VERSION,
        UpdatePlatform::Linux,
        UpdateArchitecture::X86_64,
    );
    let invalid = [
        valid.replace("Kevin-hDev/Beaver", "Kevin-hDev/CL-GO-DASH"),
        valid.replace("_amd64.deb", "_arm64.deb"),
        valid.replace("Beaver_1.1.1", "Beaver_1.1.2"),
        format!("{valid}.sha256"),
        asset_url(
            env!("CARGO_PKG_VERSION"),
            UpdatePlatform::Linux,
            UpdateArchitecture::X86_64,
        ),
        asset_url("1.0.0", UpdatePlatform::Linux, UpdateArchitecture::X86_64),
    ];

    for raw in invalid {
        assert!(
            validate_update_url_for(&raw, UpdatePlatform::Linux, UpdateArchitecture::X86_64)
                .is_err(),
            "{raw}"
        );
    }
}

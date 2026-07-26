use std::path::Path;

const HELPERS: &[&str] = &[
    "Beaver Helper",
    "Beaver Helper (GPU)",
    "Beaver Helper (Renderer)",
    "Beaver Helper (Plugin)",
    "Beaver Helper (Alerts)",
];

#[test]
fn every_cef_helper_has_a_bounded_valid_bundle_manifest() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/cef/macos/helpers");
    let mut directories = std::fs::read_dir(&root)
        .expect("helper directory")
        .map(|entry| entry.expect("helper entry").file_name())
        .collect::<Vec<_>>();
    directories.sort();
    let mut expected = HELPERS
        .iter()
        .map(|helper| format!("{helper}.app").into())
        .collect::<Vec<std::ffi::OsString>>();
    expected.sort();
    assert_eq!(directories, expected);

    for helper in HELPERS {
        let manifest = root.join(format!("{helper}.app/Contents/Info.plist"));
        let content = std::fs::read_to_string(manifest).expect("helper Info.plist");
        assert!(content.len() < 8_192);
        assert!(content.contains("<key>CFBundleExecutable</key>"));
        assert!(content.contains(&format!("<string>{helper}</string>")));
        assert!(content.contains("<string>com.clgo.dash.helper"));
        assert!(content.contains("<key>LSUIElement</key>"));
        assert!(!content.contains("PLACEHOLDER"));
    }
}

#[test]
fn macos_ad_hoc_release_allows_loading_the_bundled_cef_framework() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let content = std::fs::read_to_string(path).expect("tauri config");
    assert!(content.len() < 65_536);
    let config: serde_json::Value = serde_json::from_str(&content).expect("valid tauri config");
    let macos = config
        .pointer("/bundle/macOS")
        .expect("macOS bundle config");

    assert_eq!(
        macos
            .get("signingIdentity")
            .and_then(|value| value.as_str()),
        Some("-")
    );
    assert_eq!(
        macos
            .get("hardenedRuntime")
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        macos.get("entitlements").and_then(|value| value.as_str()),
        Some("Entitlements.plist")
    );

    let entitlements =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Entitlements.plist"))
            .expect("CEF entitlements");
    assert!(entitlements.contains("com.apple.security.cs.allow-jit"));
    assert!(entitlements.contains("com.apple.security.cs.disable-library-validation"));

    let dev_entitlements = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("Entitlements.dev.plist"),
    )
    .expect("development CEF entitlements");
    assert!(dev_entitlements.contains("com.apple.security.cs.disable-library-validation"));

    let cargo = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo manifest");
    assert!(cargo.contains("vendored-openssl"));
}

#[test]
fn development_command_verifies_cef_before_starting_vite() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let content = std::fs::read_to_string(path).expect("tauri config");
    let config: serde_json::Value = serde_json::from_str(&content).expect("valid tauri config");
    let command = config
        .pointer("/build/beforeDevCommand")
        .and_then(|value| value.as_str())
        .expect("beforeDevCommand");

    let preparation = command
        .find("prepare-cef-source.mjs")
        .expect("verified CEF preparation");
    let vite = command.find("npm run dev").expect("Vite command");
    assert!(preparation < vite);
    assert!(!command.contains("prepare-updater-helper.sh"));
    let release_command = config
        .pointer("/build/beforeBuildCommand")
        .and_then(|value| value.as_str())
        .expect("beforeBuildCommand");
    assert!(release_command.contains("prepare-updater-helper.sh"));

    let helper_script = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/prepare-updater-helper.sh"),
    )
    .expect("updater helper preparation");
    assert!(helper_script.contains("cmp -s \"$source_path\" \"$destination\""));
}

#[test]
fn cargo_is_forced_to_use_the_preverified_cef_directory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_config =
        std::fs::read_to_string(root.join(".cargo/config.toml")).expect("Cargo configuration");

    assert!(cargo_config.contains("CEF_PATH"));
    assert!(cargo_config.contains(".cef-verified/current"));
    assert!(cargo_config.contains("force = true"));
    assert!(cargo_config.contains("cef-download-disabled.invalid"));
}

#[test]
fn cef_manifest_pins_sha256_for_supported_desktops() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest =
        std::fs::read_to_string(root.join("cef-artifacts.json")).expect("CEF artifact manifest");

    assert!(manifest.contains("ef5fe464184e2e00381a2cc73e911bb4b8cc219f0e6f9fd610af0bc89d0ea58d"));
    assert!(manifest.contains("ff10d09944e976e281b2eaed17a20eaecb60ae5142ee2bd06fe2f7b38a23bf73"));
    assert!(manifest.contains("https://cef-builds.spotifycdn.com/"));
    assert!(!manifest.to_ascii_lowercase().contains("sha1"));

    let tools = std::fs::read_to_string(root.join("cef-build-tools.json"))
        .expect("CEF build tools manifest");
    assert!(tools.contains("89a287444b5b3e98f88a945afa50ce937b8ffd1dcc59c555ad9b1baf855298c9"));
    assert!(tools.contains("f550fec705b6d6ff58f2db3c374c2277a37691678d6aba463adcbb129108467a"));
}

#[test]
fn macos_development_runner_uses_a_real_application_bundle() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_config = std::fs::read_to_string(root.join(".cargo/config.toml"))
        .expect("macOS development runner config");
    let runner = std::fs::read_to_string(root.join("scripts/run-cef-dev-app.sh"))
        .expect("macOS development runner");

    assert!(cargo_config.len() < 8_192);
    assert!(cargo_config.contains("runner = \"bash scripts/run-cef-dev-app.sh\""));
    assert!(runner.len() < 16_384);
    let preparation = runner
        .find("bash scripts/ensure-cef-dev-runtime.sh")
        .expect("cached CEF preparation");
    let bundle = runner
        .find("Beaver Dev.app/Contents/MacOS")
        .expect("development application bundle");
    assert!(preparation < bundle);
    assert!(runner.contains("Beaver Dev.app/Contents/MacOS"));
    assert!(runner.contains("$DEBUG_ROOT/default-skills"));
    assert!(runner.contains("APP_RESOURCES=\"$APP_ROOT/Contents/Resources\""));
    assert!(runner.contains("\"$APP_RESOURCES/default-skills\""));
    assert!(runner.contains("rm -rf -- \"$APP_FRAMEWORKS/$helper.app\""));
    assert!(runner.contains("exec \"$APP_EXECUTABLE\""));
    assert!(runner.contains("exec \"$BINARY\" \"$@\""));
    assert!(runner.contains("--options runtime --entitlements Entitlements.dev.plist"));
    assert!(!runner.contains("--no-sandbox"));

    let preparation = std::fs::read_to_string(root.join("scripts/prepare-cef.sh"))
        .expect("CEF runtime preparation");
    assert!(preparation.contains("Release/Chromium Embedded Framework.framework"));
    assert!(preparation.contains("CLGO_CEF_ALLOW_ADHOC_SIGNING"));
    assert!(preparation.contains("CEF ad hoc release signing must be explicitly allowed"));
    assert!(preparation.contains("CARGO_BUILD_TARGET"));
    assert!(preparation.contains("target/$BUILD_TARGET/release"));
    assert!(preparation.contains("$TARGET_RELEASE_DIR/cl-go-dash-helper"));
    assert!(preparation.contains("Contents/MacOS/$helper"));

    let workflow = std::fs::read_to_string(root.join("../.github/workflows/release.yml"))
        .expect("release workflow");
    assert!(workflow
        .contains("CLGO_CEF_ALLOW_ADHOC_SIGNING: ${{ runner.os == 'macOS' && '1' || '0' }}"));
}

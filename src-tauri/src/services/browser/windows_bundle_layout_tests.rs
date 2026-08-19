use serde_json::Value;

#[test]
fn windows_bundle_stages_the_sandboxed_cef_runtime_at_the_app_root() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("tauri.windows.conf.json"))
            .expect("windows bundle config"),
    )
    .expect("valid windows bundle config");

    let hook = config
        .pointer("/build/beforeBundleCommand")
        .and_then(Value::as_str)
        .expect("Windows CEF bundle hook");
    assert!(hook.contains("prepare-cef-windows.ps1"));
    assert_eq!(
        config
            .pointer("/bundle/resources/target~1cef-runtime~1windows~1")
            .and_then(Value::as_str),
        Some("")
    );
    assert_eq!(
        config.pointer("/bundle/targets/0").and_then(Value::as_str),
        Some("nsis")
    );
}

#[test]
fn windows_bundle_hook_pins_and_verifies_the_cef_bootstrap() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = std::fs::read_to_string(root.join("scripts/prepare-cef-windows.ps1"))
        .expect("Windows CEF bundle hook");
    let manifest =
        std::fs::read_to_string(root.join("cef-artifacts.json")).expect("CEF artifact manifest");

    assert!(manifest.contains("150.0.10+g8042e43"));
    assert!(!script.contains("ExpectedArchiveSha1"));
    assert!(!script.contains("Algorithm SHA1"));
    assert!(script.contains("eab5d939293a666b210b8f5faec191324a017d6105485cfc45150863607bd367"));
    assert!(!script.contains("Join-Path $CefRoot \"Release\""));
    assert!(!script.contains("Join-Path $CefRoot \"Resources\""));
    assert!(script.contains("Join-Path $CefRoot \"locales"));
    assert!(script.contains("cl-go-dash.dll"));
    assert!(script.contains("BEAVER_TAURI_BUNDLE_TYPE"));
    assert!(script.contains("tauri-bundle-marker.mjs"));
    assert!(script.contains("$BundleMarkerScript patch-module"));
    assert!(script.contains("$BundleMarkerScript prepare-bootstrap"));
    assert!(script.contains("LICENSE.txt"));
    assert!(script.contains("CREDITS.html"));
    assert!(script.contains("$env:CARGO_BUILD_TARGET"));
    assert!(script.contains("cargo-target-dir.mjs"));
    assert!(script.contains("$env:CLGO_CEF_CARGO_FEATURES"));
    assert!(script.contains("$CargoProfile = \"debug\""));
    assert!(script.contains("tauri/custom-protocol,e2e"));
    assert!(!script.contains("Join-Path $TauriDir \"target\\release\""));
    let library_build = script
        .find("& cargo @CargoArguments")
        .expect("profile-aware Windows application DLL build");
    let library_staging = script
        .find("$ApplicationDll")
        .expect("Windows application DLL staging");
    assert!(library_build < library_staging);
    let application_source = script
        .find("$ApplicationExecutable")
        .expect("branded Tauri executable source");
    let temporary_bootstrap = script
        .find("$BrandedBootstrap")
        .expect("temporary branded CEF bootstrap");
    let branding = script
        .find("copy-windows-brand-resources.ps1")
        .expect("bounded Windows resource copier");
    let atomic_replace = script
        .find("[IO.File]::Replace")
        .expect("atomic executable replacement");
    assert!(application_source < temporary_bootstrap);
    assert!(temporary_bootstrap < branding);
    assert!(branding < atomic_replace);
}

#[test]
fn windows_e2e_overlay_preserves_object_resource_merging() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("tauri.e2e.conf.json")).expect("E2E config"),
    )
    .expect("valid E2E config");

    assert_eq!(
        config
            .pointer("/bundle/resources/default-skills~1")
            .and_then(Value::as_str),
        Some("default-skills/")
    );
    assert!(config.pointer("/bundle/resources/0").is_none());
}

#[test]
fn windows_release_exposes_the_explicit_cargo_target_to_the_bundle_hook() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow = std::fs::read_to_string(root.join("../.github/workflows/release.yml"))
        .expect("release workflow");

    assert!(workflow.contains("CARGO_BUILD_TARGET: ${{ matrix.target }}"));
    assert!(workflow.contains("BEAVER_TAURI_BUNDLE_TYPE: ${{ matrix.bundles }}"));
    assert!(workflow.contains("--bundles=${{ matrix.bundles }}"));
    assert!(workflow.contains("tauri-bundle-marker.mjs verify $env:BEAVER_TAURI_BUNDLE_TYPE"));
    assert!(workflow.contains("CARGO_TARGET_DIR=$target"));
    assert!(workflow.contains("Out-File -FilePath $env:GITHUB_ENV"));
    assert!(workflow.contains("- os: windows-latest"));
}

#[test]
fn windows_private_storage_ci_does_not_load_the_cef_runtime() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow =
        std::fs::read_to_string(root.join("../.github/workflows/ci.yml")).expect("CI workflow");

    assert!(workflow.contains("cargo test --manifest-path tests/windows-private-store/Cargo.toml"));
    assert!(!workflow.contains("prepare-cef-windows-test-runtime.ps1"));
}

#[test]
fn windows_backend_ci_checks_native_cef_and_isolates_it_from_unit_tests() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow =
        std::fs::read_to_string(root.join("../.github/workflows/ci.yml")).expect("CI workflow");
    let native_job = workflow
        .split_once("backend-windows-native:")
        .map(|(_, job)| job)
        .and_then(|job| job.split_once("backend-windows:").map(|(job, _)| job))
        .expect("Windows native backend job");
    let test_job = workflow
        .split_once("backend-windows:")
        .map(|(_, job)| job)
        .and_then(|job| job.split_once("frontend:").map(|(job, _)| job))
        .expect("Windows test backend job");

    let preparation = native_job
        .find("node scripts/cef/prepare-cef-source.mjs")
        .expect("verified CEF preparation");
    let clippy = native_job
        .find("cargo clippy --all-targets -- -D warnings")
        .expect("Windows Clippy check");
    assert!(native_job.contains("runs-on: windows-latest"));
    assert!(native_job.contains("src-tauri/.cef-cache"));
    assert!(native_job.contains("src-tauri/.cef-tool-cache"));
    assert!(preparation < clippy);
    assert!(test_job.contains("runs-on: windows-2022"));
    assert_eq!(test_job.matches("--features windows-tests").count(), 4);
    // --lib et non --all : configure_windows_test_manifest pose /MANIFESTINPUT
    // par cargo:rustc-link-arg, qui atteint toutes les cibles liées. Avec --all
    // les binaires reçoivent un second manifeste et l'éditeur de liens s'arrête
    // sur CVT1100. Y passer demande de restreindre le manifeste aux cibles de
    // test (cargo:rustc-link-arg-tests).
    assert!(test_job.contains("cargo test --lib --features windows-tests -- --test-threads=1"));
    assert!(test_job.contains("Windows AppContainer test inventory"));
    assert!(!test_job.contains("prepare-cef-source.mjs"));
    assert!(!test_job.contains(".cef-cache"));
    assert!(!workflow.contains("/DELAYLOAD:libcef.dll"));
    assert!(!workflow.contains("delayimp.lib"));
}

#[test]
fn windows_unit_tests_link_the_common_controls_v6_manifest() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let build_script = std::fs::read_to_string(root.join("build.rs")).expect("build script");
    let manifest =
        std::fs::read_to_string(root.join("windows-test.manifest")).expect("test manifest");

    assert!(build_script.contains("CARGO_FEATURE_WINDOWS_TESTS"));
    assert!(build_script.contains("cargo:rustc-link-arg="));
    assert!(build_script.contains("/MANIFEST:EMBED"));
    assert!(build_script.contains("/MANIFESTINPUT:"));
    assert!(manifest.contains("Microsoft.Windows.Common-Controls"));
    assert!(manifest.contains("version=\"6.0.0.0\""));
}

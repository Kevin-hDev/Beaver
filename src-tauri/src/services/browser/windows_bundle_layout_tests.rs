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
    assert!(script.contains("LICENSE.txt"));
    assert!(script.contains("CREDITS.html"));
    assert!(script.contains("$env:CARGO_BUILD_TARGET"));
    assert!(script.contains("target\\$BuildTarget\\release"));
    let library_build = script
        .find("cargo build --release --lib")
        .expect("explicit Windows application DLL build");
    assert!(script.contains("--features tauri/custom-protocol"));
    let library_staging = script
        .find("$ApplicationDll")
        .expect("Windows application DLL staging");
    assert!(library_build < library_staging);
}

#[test]
fn windows_release_exposes_the_explicit_cargo_target_to_the_bundle_hook() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow = std::fs::read_to_string(root.join("../.github/workflows/release.yml"))
        .expect("release workflow");

    assert!(workflow.contains("CARGO_BUILD_TARGET: ${{ matrix.target }}"));
}

#[test]
fn windows_private_storage_ci_does_not_load_the_cef_runtime() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflow =
        std::fs::read_to_string(root.join("../.github/workflows/ci.yml")).expect("CI workflow");

    assert!(workflow.contains("cargo test --manifest-path tests/windows-private-store/Cargo.toml"));
    assert!(!workflow.contains("prepare-cef-windows-test-runtime.ps1"));
}

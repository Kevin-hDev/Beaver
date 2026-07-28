use super::npm_runner::{package_path, resolve_cli, NpmRunner};
use super::source_validation::NpmSource;
use std::ffi::OsString;
use std::path::PathBuf;

#[test]
fn scoped_package_names_map_to_bounded_relative_paths() {
    assert_eq!(
        package_path("@beaver/search"),
        PathBuf::from("@beaver").join("search")
    );
    assert_eq!(package_path("search"), PathBuf::from("search"));
}

#[test]
fn every_npm_install_disables_lifecycle_scripts_and_side_effects() {
    let runner = NpmRunner::for_test(PathBuf::from("/node"), PathBuf::from("/npm-cli.js"));
    let arguments = runner.common_arguments(
        "install",
        PathBuf::from("/extension").as_path(),
        PathBuf::from("/cache").as_path(),
    );

    for required in [
        "--ignore-scripts",
        "--omit=dev",
        "--no-audit",
        "--no-fund",
        "--no-bin-links",
        "--workspaces=false",
        "--strict-ssl=true",
        "--replace-registry-host=always",
        "https://registry.npmjs.org/",
    ] {
        assert!(arguments.contains(&OsString::from(required)));
    }
}

#[test]
fn local_fixture_install_never_runs_its_lifecycle_script() {
    let node = which::which("node").unwrap().canonicalize().unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let cli = resolve_cli(temporary.path(), &node).unwrap();
    let runner = NpmRunner::for_test(node, cli);
    let package = temporary.path().join("package");
    let prefix = temporary.path().join("install");
    std::fs::create_dir(&package).unwrap();
    std::fs::create_dir(&prefix).unwrap();
    std::fs::write(
        package.join("package.json"),
        serde_json::json!({
            "name": "beaver-test-extension",
            "version": "1.0.0",
            "main": "index.js",
            "scripts": { "postinstall": "node postinstall.js" },
            "beaver": {
                "id": "test.npm.lifecycle",
                "beaverApi": "1",
                "runtime": "node",
                "access": "full"
            }
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(package.join("index.js"), "export default () => {}").unwrap();
    std::fs::write(
        package.join("postinstall.js"),
        "require('fs').writeFileSync('postinstall-ran', 'unsafe')",
    )
    .unwrap();
    let source = NpmSource {
        locator: package.to_str().unwrap().to_string(),
        package_name: "beaver-test-extension".to_string(),
    };

    let installed = runner.install_package(&prefix, &source).unwrap();

    assert!(installed.join("index.js").is_file());
    assert!(!installed.join("postinstall-ran").exists());
    assert!(!prefix.join(".npm-cache").exists());
}

#[test]
fn npm_resolution_never_falls_back_to_an_unrelated_system_cli() {
    let temporary = tempfile::tempdir().unwrap();
    let node = temporary.path().join("node");
    std::fs::write(&node, "").unwrap();

    assert!(resolve_cli(temporary.path(), &node).is_err());
}

#[test]
fn an_uncleanable_cache_blocks_installation_before_validation() {
    let temporary = tempfile::tempdir().unwrap();
    let prefix = temporary.path().join("install");
    std::fs::create_dir(&prefix).unwrap();
    std::fs::write(prefix.join(".npm-cache"), "not a directory").unwrap();
    let runner = NpmRunner::for_test(PathBuf::from("/node"), PathBuf::from("/npm-cli.js"));
    let source = NpmSource {
        locator: "beaver-test-extension".to_string(),
        package_name: "beaver-test-extension".to_string(),
    };

    assert!(runner.install_package(&prefix, &source).is_err());
    assert!(prefix.join(".npm-cache").is_file());
}

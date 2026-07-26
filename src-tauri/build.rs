fn main() {
    prepare_cef_bundle_placeholders();
    prepare_updater_helper_placeholder();
    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-lib=framework=CoreServices");
    }
}

fn prepare_updater_helper_placeholder() {
    let directory = std::path::Path::new("target/updater-helper");
    std::fs::create_dir_all(directory).expect("cannot prepare updater helper directory");
    let name = if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        "cl-go-dash-updater.exe"
    } else {
        "cl-go-dash-updater"
    };
    let path = directory.join(name);
    if !path.exists() {
        std::fs::File::create(path).expect("cannot prepare updater helper placeholder");
    }
}

fn prepare_cef_bundle_placeholders() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    if target_os.as_deref() == Ok("windows") {
        std::fs::create_dir_all("target/cef-runtime/windows")
            .expect("cannot prepare Windows CEF bundle directory");
        return;
    }
    if target_os.as_deref() != Ok("macos") {
        return;
    }
    let root = std::path::Path::new("target/cef-runtime/macos");
    let framework = root.join("Chromium Embedded Framework.framework");
    let helpers = root.join("helpers");
    if let Err(error) = std::fs::create_dir_all(framework) {
        panic!("cannot prepare CEF bundle directory: {error}");
    }
    if let Err(error) = std::fs::create_dir_all(helpers) {
        panic!("cannot prepare CEF helper directory: {error}");
    }
    let license = root.join("LICENSE.txt");
    if !license.exists() {
        std::fs::File::create(license).expect("cannot prepare CEF license placeholder");
    }
}

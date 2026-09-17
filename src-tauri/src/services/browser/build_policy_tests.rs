fn normalized_source(path: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .expect("Rust source")
        .replace("\r\n", "\n")
}

#[test]
fn build_script_never_embeds_dotenv_values_in_the_binary() {
    let build = normalized_source("build.rs");

    assert!(!build.contains("load_env"));
    assert!(!build.contains("cargo:rustc-env"));
}

#[test]
fn build_script_names_browser_capabilities_separately() {
    let build = normalized_source("build.rs");

    assert!(build.contains("cargo:rustc-check-cfg=cfg(browser_native_api)"));
    assert!(build.contains("cargo:rustc-check-cfg=cfg(native_browser)"));
    assert!(build.contains(
        r#"if target == "macos" || target == "windows" {
        println!("cargo:rustc-cfg=browser_native_api");"#
    ));
    assert!(build.contains(
        r#"if target == "macos" || (target == "windows" && !windows_tests) {
        println!("cargo:rustc-cfg=native_browser");"#
    ));
}

#[test]
fn native_runtime_modules_are_not_built_in_linux_library() {
    let module = normalized_source("src/services/browser/mod.rs");

    for runtime_module in [
        "lifecycle",
        "native_paths",
        "navigation_target",
        "runtime_revision",
        "session_model_runtime",
        "view_recency",
        "view_state",
    ] {
        let guarded = format!("#[cfg(any(test, browser_native_api))]\nmod {runtime_module};");
        assert!(
            module.contains(&guarded),
            "{runtime_module} must be excluded from the Linux library build"
        );
    }

    assert!(module.contains("#[cfg(any(test, target_os = \"macos\"))]\nmod cookie_store_probe;"));
}

#[test]
fn native_favicon_modules_are_not_built_in_linux_library() {
    let module = normalized_source("src/services/browser/mod.rs");

    // Linux exposes the portable empty snapshot through IPC, while the CEF
    // download engine remains native-only so strict production builds stay clean.
    assert!(module.contains("#[cfg(any(test, native_browser))]\nmod browser_view_key;"));
    for runtime_module in [
        "favicon_png",
        "favicon_policy",
        "favicon_runtime",
        "favicon_state",
        "favicon_store",
        "favicon_task_gate",
        "favicon_types",
        "favicon_watchdog",
    ] {
        let guarded = format!("#[cfg(any(test, native_browser))]\nmod {runtime_module};");
        assert!(
            module.contains(&guarded),
            "{runtime_module} must be excluded from the Linux library build"
        );
    }
}

#[test]
fn native_runtime_entrypoints_stay_out_of_linux_tests() {
    let runtime = normalized_source("src/services/browser/runtime_handle.rs");
    let module = normalized_source("src/services/browser/mod.rs");
    let sessions = normalized_source("src/services/browser/session_service_runtime.rs");
    let native = "#[cfg(browser_native_api)]";

    for signature in [
        "pub(super) fn mark_failed",
        "pub(super) fn begin_stopping",
        "pub(super) fn mark_stopped",
    ] {
        assert!(runtime.contains(&format!("{native}\n    {signature}")));
    }
    assert!(module.contains(&format!("{native}\nmod session_service_runtime;")));
    for signature in [
        "pub(super) fn update_runtime",
        "pub(super) fn mark_released",
    ] {
        assert!(sessions.contains(signature));
    }
}

#[test]
fn native_view_release_paths_share_one_boundary() {
    let bridge = normalized_source("src/services/browser/cef_state_bridge.rs");
    let view = normalized_source("src/services/browser/cef_surface_view.rs");
    let renderer = normalized_source("src/services/browser/cef_request_handler.rs");
    let lifecycle = normalized_source("src/services/browser/cef_life_span_handler.rs");

    assert!(bridge.contains("state.release_view(key, epoch)"));
    assert!(bridge.contains("mark_view_released(app, key.clone(), stamp)"));
    assert!(view.contains("release_view(app, &self.key, &self.slot)"));
    assert!(renderer.contains("release_view(Some(&app), &key, &self.slot)"));
    assert!(lifecycle.contains("release_view(Some(&self.app), &self.key, &self.slot)"));
    for source in [&view, &renderer, &lifecycle] {
        assert!(!source.contains("state.release_view"));
        assert!(!source.contains("mark_view_released"));
    }
}

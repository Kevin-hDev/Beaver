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
fn native_runtime_modules_are_not_built_in_linux_library() {
    let module = normalized_source("src/services/browser/mod.rs");

    for runtime_module in [
        "browser_view_key",
        "lifecycle",
        "native_paths",
        "navigation_target",
        "runtime_revision",
        "session_model_runtime",
        "view_recency",
        "view_state",
    ] {
        let guarded = format!(
            "#[cfg(any(test, target_os = \"macos\", target_os = \"windows\"))]\nmod {runtime_module};"
        );
        assert!(
            module.contains(&guarded),
            "{runtime_module} must be excluded from the Linux library build"
        );
    }

    assert!(module.contains("#[cfg(any(test, target_os = \"macos\"))]\nmod cookie_store_probe;"));
}

#[test]
fn native_runtime_entrypoints_stay_out_of_linux_tests() {
    let runtime = normalized_source("src/services/browser/runtime_handle.rs");
    let sessions = normalized_source("src/services/browser/session_service.rs");
    let native = "#[cfg(any(target_os = \"macos\", target_os = \"windows\"))]";

    for signature in [
        "pub(super) fn mark_failed",
        "pub(super) fn begin_stopping",
        "pub(super) fn mark_stopped",
    ] {
        assert!(runtime.contains(&format!("{native}\n    {signature}")));
    }
    for signature in [
        "pub(super) fn update_runtime",
        "pub(super) fn mark_released",
    ] {
        assert!(sessions.contains(&format!("{native}\n    {signature}")));
    }
}

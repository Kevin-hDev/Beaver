use super::*;

#[cfg(all(test, native_browser))]
#[path = "browser_slot_tests.rs"]
mod browser_slot_tests;
#[cfg(test)]
#[path = "build_policy_tests.rs"]
mod build_policy_tests;
#[cfg(all(test, target_os = "macos"))]
#[path = "bundle_layout_tests.rs"]
mod bundle_layout_tests;
#[cfg(test)]
#[path = "cef_cookie_gate_policy_tests.rs"]
mod cef_cookie_gate_policy_tests;
#[cfg(all(test, native_browser))]
#[path = "cef_diagnostics_tests.rs"]
mod cef_diagnostics_tests;
#[cfg(test)]
#[path = "cef_preflight_tests.rs"]
mod cef_preflight_tests;
#[cfg(test)]
#[path = "cookie_store_probe_tests.rs"]
mod cookie_store_probe_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
#[path = "ffi_guard_tests.rs"]
mod ffi_guard_tests;
#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "live_session_registry_tests.rs"]
mod live_session_registry_tests;
#[cfg(test)]
#[path = "local_site_candidates_tests.rs"]
mod local_site_candidates_tests;
#[cfg(test)]
#[path = "local_site_policy_tests.rs"]
mod local_site_policy_tests;
#[cfg(test)]
#[path = "local_site_probe_tests.rs"]
mod local_site_probe_tests;
#[cfg(test)]
#[path = "local_site_scan_state_tests.rs"]
mod local_site_scan_state_tests;
#[cfg(test)]
#[path = "local_site_scan_throttle_tests.rs"]
mod local_site_scan_throttle_tests;
#[cfg(test)]
#[path = "native_paths_tests.rs"]
mod native_paths_tests;
#[cfg(all(test, target_os = "macos"))]
#[path = "native_pump_policy_tests.rs"]
mod native_pump_policy_tests;
#[cfg(test)]
#[path = "navigation_target_tests.rs"]
mod navigation_target_tests;
#[cfg(test)]
#[path = "process_role_tests.rs"]
mod process_role_tests;
#[cfg(all(test, target_os = "macos"))]
#[path = "pump_gate_tests.rs"]
mod pump_gate_tests;
#[cfg(test)]
#[path = "runtime_handle_tests.rs"]
mod runtime_handle_tests;
#[cfg(test)]
#[path = "runtime_revision_tests.rs"]
mod runtime_revision_tests;
#[cfg(test)]
#[path = "session_model_tests.rs"]
mod session_model_tests;
#[cfg(test)]
#[path = "session_order_tests.rs"]
mod session_order_tests;
#[cfg(test)]
#[path = "session_store_tests.rs"]
mod session_store_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
#[path = "settings_tests.rs"]
mod settings_tests;
#[cfg(test)]
#[path = "surface_bounds_tests.rs"]
mod surface_bounds_tests;
#[cfg(test)]
#[path = "url_policy_tests.rs"]
mod url_policy_tests;
#[cfg(test)]
#[path = "view_recency_tests.rs"]
mod view_recency_tests;
#[cfg(test)]
#[path = "view_state_tests.rs"]
mod view_state_tests;
#[cfg(test)]
#[path = "windows_bundle_layout_tests.rs"]
mod windows_bundle_layout_tests;
#[cfg(all(test, target_os = "windows"))]
#[path = "windows_surface_order_tests.rs"]
mod windows_surface_order_tests;

#[cfg(test)]
#[path = "favicon_contract_tests.rs"]
mod favicon_contract_tests;
#[cfg(test)]
#[path = "favicon_png_tests.rs"]
mod favicon_png_tests;
